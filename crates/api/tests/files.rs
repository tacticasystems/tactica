mod support;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::Response,
};
use serde_json::Value;
use support::ApiFixture;
use tactica_db_model::{
    CreateUnit, CreatedUnit, NewUnit, NewUnitMembership, NewUser, UnitMembershipStore, UnitStore,
    UserStore,
};
use tactica_uuid_kinds::{MemberId, UnitId, UserId};
use tower::ServiceExt;

async fn user(api: &ApiFixture, name: &str) -> UserId {
    let id = UserId::new();
    UserStore::create(
        &api.storage,
        NewUser {
            id,
            username: name.to_owned(),
            email: format!("{name}@example.test"),
            display_name: None,
            icon_url: None,
            banner_url: None,
            biography: None,
            is_active: true,
            is_superuser: false,
            password_hash: None,
            totp_secret: None,
        },
    )
    .await
    .expect("user");
    id
}
async fn create_unit(api: &ApiFixture, owner: UserId, slug: &str) -> CreatedUnit {
    api.storage
        .create_with_defaults(CreateUnit {
            owner_id: owner,
            unit: NewUnit {
                id: UnitId::new(),
                slug: slug.to_owned(),
                display_name: Some(slug.to_owned()),
                icon_url: None,
                banner_url: None,
                biography: None,
            },
        })
        .await
        .expect("unit")
}
async fn request(
    api: &ApiFixture,
    method: &str,
    path: &str,
    actor: Option<UserId>,
    data: Option<&[u8]>,
) -> Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(actor) = actor {
        let token = api.jwt.generate_jwt_for_user(actor).expect("token");
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let body = if let Some(data) = data {
        request = request.header(
            "Content-Type",
            "multipart/form-data; boundary=test-boundary",
        );
        let mut body = b"--test-boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"report.txt\"\r\nContent-Type: text/plain\r\n\r\n".to_vec();
        body.extend_from_slice(data);
        body.extend_from_slice(b"\r\n--test-boundary--\r\n");
        Body::from(body)
    } else {
        Body::empty()
    };
    api.router
        .clone()
        .oneshot(request.body(body).expect("request"))
        .await
        .expect("response")
}
async fn json(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("JSON")
}

#[tokio::test(flavor = "multi_thread")]
async fn attachments_are_scoped_to_members_and_deletion_requires_ownership_or_management() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let member = user(&api, "member").await;
    let outsider = user(&api, "outsider").await;
    let unit = create_unit(&api, owner, "unit").await;
    let member_id = MemberId::new();
    UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: member_id,
            user_id: member,
            unit_id: unit.unit.id,
            rank_id: unit.owner_rank_id,
        },
    )
    .await
    .expect("membership");
    let path = format!("/api/v1/units/{}/files", unit.unit.id);
    assert_eq!(
        request(&api, "POST", &path, None, Some(b"hello"))
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&api, "POST", &path, Some(outsider), Some(b"hello"))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let uploaded = request(&api, "POST", &path, Some(owner), Some(b"hello")).await;
    assert_eq!(uploaded.status(), StatusCode::CREATED);
    let file = json(uploaded).await;
    let url = file["url"].as_str().expect("URL");
    assert_eq!(file["size"], 5);
    let downloaded = request(&api, "GET", url, Some(member), None).await;
    assert_eq!(downloaded.status(), StatusCode::OK);
    assert_eq!(downloaded.headers()["cache-control"], "private, no-store");
    assert_eq!(
        downloaded.headers()["content-type"],
        "application/octet-stream"
    );
    assert_eq!(
        to_bytes(downloaded.into_body(), 100).await.expect("bytes"),
        "hello"
    );
    assert_eq!(
        request(&api, "GET", url, Some(outsider), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    let foreign = create_unit(&api, owner, "foreign").await.unit.id;
    let foreign_url = format!(
        "/api/v1/units/{foreign}/files/{}",
        file["id"].as_str().expect("id")
    );
    assert_eq!(
        request(&api, "GET", &foreign_url, Some(owner), None)
            .await
            .status(),
        StatusCode::NOT_FOUND
    );
    let public_url = url.replace("/files/", "/icon/");
    assert_eq!(
        request(&api, "GET", &public_url, None, None).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(&api, "DELETE", url, Some(member), None)
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        json(request(&api, "GET", &path, Some(member), None).await).await["files"]
            .as_array()
            .expect("list")
            .len(),
        1
    );
    // The upload route must accept files larger than the ordinary 1 MiB API cap.
    let member_file = request(
        &api,
        "POST",
        &path,
        Some(member),
        Some(&vec![7; 1024 * 1024 + 1]),
    )
    .await;
    assert_eq!(member_file.status(), StatusCode::CREATED);
    let member_file = json(member_file).await;
    assert_eq!(
        request(
            &api,
            "DELETE",
            member_file["url"].as_str().expect("URL"),
            Some(owner),
            None
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );
    let token = api.jwt.generate_jwt_for_user(member).expect("token");
    assert_eq!(
        api.get(&format!("{path}?limit=0"), Some(&token)).await.0,
        StatusCode::BAD_REQUEST
    );
    UnitMembershipStore::delete(&api.storage, member_id)
        .await
        .expect("remove member");
    assert_eq!(
        request(&api, "GET", url, Some(member), None).await.status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&api, "DELETE", url, Some(owner), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&api, "GET", url, Some(owner), None).await.status(),
        StatusCode::NOT_FOUND
    );
}
#[tokio::test(flavor = "multi_thread")]
async fn icons_are_public_validated_permission_checked_and_update_the_unit() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let member = user(&api, "member").await;
    let unit = create_unit(&api, owner, "unit").await;
    UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: member,
            unit_id: unit.unit.id,
            rank_id: unit.owner_rank_id,
        },
    )
    .await
    .expect("membership");
    let path = format!("/api/v1/units/{}/icon", unit.unit.id);
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("PNG");
    assert_eq!(
        request(&api, "POST", &path, Some(member), Some(png.get_ref()))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&api, "POST", &path, Some(owner), Some(b"<svg/>"))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &api,
            "POST",
            &path,
            Some(owner),
            Some(&vec![0; 2 * 1024 * 1024 + 1])
        )
        .await
        .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let response = request(&api, "POST", &path, Some(owner), Some(png.get_ref())).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let first = json(response).await;
    let url = first["url"].as_str().expect("URL");
    assert_eq!(
        UnitStore::get(&api.storage, unit.unit.id)
            .await
            .expect("unit")
            .expect("exists")
            .icon_url
            .as_deref(),
        Some(url)
    );
    let icon = request(&api, "GET", url, None, None).await;
    assert_eq!(icon.status(), StatusCode::OK);
    assert_eq!(icon.headers()["content-type"], "image/png");
    let second = json(request(&api, "POST", &path, Some(owner), Some(png.get_ref())).await).await;
    let delete_url = url.replace("/icon/", "/files/");
    assert_eq!(
        request(&api, "DELETE", &delete_url, Some(owner), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        UnitStore::get(&api.storage, unit.unit.id)
            .await
            .expect("unit")
            .expect("exists")
            .icon_url
            .as_deref(),
        second["url"].as_str()
    );
    let attachments = format!("/api/v1/units/{}/files", unit.unit.id);
    assert!(
        json(request(&api, "GET", &attachments, Some(owner), None).await).await["files"]
            .as_array()
            .expect("list")
            .is_empty()
    );
    assert_eq!(
        request(&api, "POST", &attachments, Some(owner), Some(b""))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &api,
            "POST",
            &attachments,
            Some(owner),
            Some(&vec![0; 10 * 1024 * 1024 + 1])
        )
        .await
        .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}
#[tokio::test(flavor = "multi_thread")]
async fn banners_are_public_validated_permission_checked_and_update_the_unit() {
    let api = ApiFixture::new().await;
    let owner = user(&api, "owner").await;
    let member = user(&api, "member").await;
    let unit = create_unit(&api, owner, "unit").await;
    UnitMembershipStore::create(
        &api.storage,
        NewUnitMembership {
            id: MemberId::new(),
            user_id: member,
            unit_id: unit.unit.id,
            rank_id: unit.owner_rank_id,
        },
    )
    .await
    .expect("membership");
    let path = format!("/api/v1/units/{}/banner", unit.unit.id);
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::new_rgb8(2, 2)
        .write_to(&mut png, image::ImageFormat::Png)
        .expect("PNG");
    assert_eq!(
        request(&api, "POST", &path, Some(member), Some(png.get_ref()))
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(&api, "POST", &path, Some(owner), Some(b"<svg/>"))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &api,
            "POST",
            &path,
            Some(owner),
            Some(&vec![0; 5 * 1024 * 1024 + 1])
        )
        .await
        .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let response = request(&api, "POST", &path, Some(owner), Some(png.get_ref())).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let first = json(response).await;
    let url = first["url"].as_str().expect("URL");
    assert_eq!(
        UnitStore::get(&api.storage, unit.unit.id)
            .await
            .expect("unit")
            .expect("exists")
            .banner_url
            .as_deref(),
        Some(url)
    );
    let wrong_kind = url.replace("/banner/", "/icon/");
    assert_eq!(
        request(&api, "GET", &wrong_kind, None, None).await.status(),
        StatusCode::NOT_FOUND
    );
    let other = create_unit(&api, owner, "other").await;
    let wrong_unit = url.replace(&unit.unit.id.to_string(), &other.unit.id.to_string());
    assert_eq!(
        request(&api, "GET", &wrong_unit, None, None).await.status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &api,
            "DELETE",
            &url.replace("/banner/", "/files/"),
            Some(member),
            None
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let icon = request(&api, "GET", url, None, None).await;
    assert_eq!(icon.status(), StatusCode::OK);
    assert_eq!(icon.headers()["content-type"], "image/png");
    let second = json(request(&api, "POST", &path, Some(owner), Some(png.get_ref())).await).await;
    let delete_url = url.replace("/banner/", "/files/");
    assert_eq!(
        request(&api, "DELETE", &delete_url, Some(owner), None)
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        UnitStore::get(&api.storage, unit.unit.id)
            .await
            .expect("unit")
            .expect("exists")
            .banner_url
            .as_deref(),
        second["url"].as_str()
    );
    let current = second["url"].as_str().expect("URL");
    assert_eq!(
        request(
            &api,
            "DELETE",
            &current.replace("/banner/", "/files/"),
            Some(owner),
            None
        )
        .await
        .status(),
        StatusCode::NO_CONTENT
    );
    assert!(
        UnitStore::get(&api.storage, unit.unit.id)
            .await
            .expect("unit")
            .expect("exists")
            .banner_url
            .is_none()
    );
    let attachments = format!("/api/v1/units/{}/files", unit.unit.id);
    assert!(
        json(request(&api, "GET", &attachments, Some(owner), None).await).await["files"]
            .as_array()
            .expect("list")
            .is_empty()
    );
    assert_eq!(
        request(&api, "POST", &attachments, Some(owner), Some(b""))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(
            &api,
            "POST",
            &attachments,
            Some(owner),
            Some(&vec![0; 10 * 1024 * 1024 + 1])
        )
        .await
        .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );
}
