use std::io::Cursor;

use axum::{
    Json, Router,
    extract::{Multipart, Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use tactica_api_types::v1::files::{FileSummary, ListFilesResponse};
use tactica_db_model::{File, ListPagination, NewFile};
use tactica_files::{Bytes, FileKey, FileStorage};
use tactica_uuid_kinds::{FileId, UnitId};
use tower_http::limit::RequestBodyLimitLayer;

use super::common::{require_unit_member, require_user, validate_pagination};
use crate::{
    error::{Error, Result},
    state::{ApiState, Files, Principal, Storage},
};

const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
const MAX_BANNER_BYTES: usize = 5 * 1024 * 1024;
const MAX_ICON_BYTES: usize = 2 * 1024 * 1024;

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/api/v1/units/{unit_id}/files", get(list).post(upload))
        .route(
            "/api/v1/units/{unit_id}/files/{file_id}",
            get(download).delete(remove),
        )
        .layer(RequestBodyLimitLayer::new(MAX_FILE_BYTES + 64 * 1024))
        .merge(
            Router::new()
                .route("/api/v1/units/{unit_id}/icon", post(upload_icon))
                .route("/api/v1/units/{unit_id}/icon/{file_id}", get(icon))
                .layer(RequestBodyLimitLayer::new(MAX_ICON_BYTES + 64 * 1024)),
        )
        .merge(
            Router::new()
                .route("/api/v1/units/{unit_id}/banner", post(upload_banner))
                .route("/api/v1/units/{unit_id}/banner/{file_id}", get(banner))
                .layer(RequestBodyLimitLayer::new(MAX_BANNER_BYTES + 64 * 1024)),
        )
}

#[derive(Clone, Copy)]
enum UploadKind {
    Attachment,
    Icon,
    Banner,
}

fn summary(file: File) -> FileSummary {
    let kind = if file.is_icon {
        "icon"
    } else if file.is_banner {
        "banner"
    } else {
        "files"
    };
    FileSummary {
        url: format!("/api/v1/units/{}/{kind}/{}", file.unit_id, file.id),
        id: file.id,
        unit_id: file.unit_id,
        uploaded_by: file.uploaded_by,
        filename: file.filename,
        content_type: file.content_type,
        size: file.size,
        created_at: file.created_at,
    }
}
const fn key(id: FileId) -> FileKey {
    FileKey::from_uuid(*id.as_uuid())
}

async fn list(
    Storage(db): Storage,
    Principal(actor): Principal,
    Path(unit): Path<UnitId>,
    Query(pagination): Query<ListPagination>,
) -> Result<Json<ListFilesResponse>> {
    validate_pagination(&pagination)?;
    require_unit_member(db.as_ref(), actor, unit).await?;
    Ok(Json(ListFilesResponse {
        files: db
            .list_files(unit, &pagination)
            .await?
            .into_iter()
            .map(summary)
            .collect(),
    }))
}

async fn upload(
    storage: Storage,
    files: Files,
    principal: Principal,
    Path(unit): Path<UnitId>,
    multipart: Multipart,
) -> Result<impl IntoResponse> {
    save(
        storage,
        files,
        principal,
        unit,
        multipart,
        UploadKind::Attachment,
    )
    .await
}
async fn upload_icon(
    storage: Storage,
    files: Files,
    principal: Principal,
    Path(unit): Path<UnitId>,
    multipart: Multipart,
) -> Result<impl IntoResponse> {
    save(storage, files, principal, unit, multipart, UploadKind::Icon).await
}

async fn upload_banner(
    storage: Storage,
    files: Files,
    principal: Principal,
    Path(unit): Path<UnitId>,
    multipart: Multipart,
) -> Result<impl IntoResponse> {
    save(
        storage,
        files,
        principal,
        unit,
        multipart,
        UploadKind::Banner,
    )
    .await
}

async fn read_upload(mut multipart: Multipart, limit: usize) -> Result<(String, Bytes)> {
    let mut field = multipart
        .next_field()
        .await
        .map_err(|error| multipart_error(&error))?
        .ok_or_else(|| Error::Validation("A file field is required".to_owned()))?;
    if field.name() != Some("file") {
        return Err(Error::Validation(
            "Expected a multipart field named file".to_owned(),
        ));
    }
    let filename = field
        .file_name()
        .ok_or_else(|| Error::Validation("A filename is required".to_owned()))?;
    let filename = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or_default()
        .trim();
    if filename.is_empty() || filename.len() > 255 || filename.chars().any(char::is_control) {
        return Err(Error::Validation("Invalid filename".to_owned()));
    }
    let filename = filename.to_owned();
    let mut data = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|error| multipart_error(&error))?
    {
        if data.len().saturating_add(chunk.len()) > limit {
            return Err(Error::PayloadTooLarge);
        }
        data.extend_from_slice(&chunk);
    }
    drop(field);
    if data.is_empty() {
        return Err(Error::Validation(
            "Empty files are not supported".to_owned(),
        ));
    }
    if multipart
        .next_field()
        .await
        .map_err(|error| multipart_error(&error))?
        .is_some()
    {
        return Err(Error::Validation("Send exactly one file field".to_owned()));
    }
    Ok((filename, data.into()))
}
fn multipart_error(error: &axum::extract::multipart::MultipartError) -> Error {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        Error::PayloadTooLarge
    } else {
        Error::Validation("Invalid multipart upload".to_owned())
    }
}

/// Decode and re-encode raster images so stored profile artwork contains only image data.
fn normalize_image(bytes: &[u8], banner: bool) -> Result<Bytes> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_error| Error::Validation("Invalid image".to_owned()))?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::WebP)
    ) {
        return Err(Error::Validation(
            "Images must be PNG, JPEG or WebP images".to_owned(),
        ));
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(if banner { 4096 } else { 2048 });
    limits.max_image_height = Some(if banner { 4096 } else { 2048 });
    limits.max_alloc = Some(if banner { 128 } else { 32 } * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(|_error| {
        Error::Validation(format!(
            "Invalid image or dimensions exceed {}",
            if banner { "4096×4096" } else { "2048×2048" }
        ))
    })?;
    let mut output = Cursor::new(Vec::new());
    image
        .thumbnail(
            if banner { 1920 } else { 512 },
            if banner { 1080 } else { 512 },
        )
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|error| Error::Other(error.into()))?;
    Ok(output.into_inner().into())
}

async fn save(
    Storage(db): Storage,
    Files(files): Files,
    Principal(actor): Principal,
    unit: UnitId,
    multipart: Multipart,
    kind: UploadKind,
) -> Result<impl IntoResponse> {
    let actor_id = require_user(actor)?;
    let is_icon = matches!(kind, UploadKind::Icon);
    let is_banner = matches!(kind, UploadKind::Banner);
    db.authorize_file_upload(actor_id, unit, is_icon || is_banner)
        .await?;
    let (mut filename, mut bytes) = read_upload(
        multipart,
        if is_banner {
            MAX_BANNER_BYTES
        } else if is_icon {
            MAX_ICON_BYTES
        } else {
            MAX_FILE_BYTES
        },
    )
    .await?;
    let content_type = if is_icon || is_banner {
        bytes = tokio::task::spawn_blocking(move || normalize_image(&bytes, is_banner))
            .await
            .map_err(|error| Error::Other(error.into()))??;
        filename = if is_banner { "banner.png" } else { "icon.png" }.to_owned();
        "image/png"
    } else {
        "application/octet-stream"
    };
    let id = FileId::new();
    let size = i64::try_from(bytes.len()).map_err(|_error| Error::PayloadTooLarge)?;
    files.put(key(id), bytes).await?;
    let result = db
        .create_file(NewFile {
            id,
            unit_id: unit,
            uploaded_by: actor_id,
            filename,
            content_type: content_type.to_owned(),
            size,
            is_icon,
            is_banner,
        })
        .await;
    match result {
        Ok(file) => Ok((StatusCode::CREATED, Json(summary(file)))),
        Err(error) => {
            cleanup(files.as_ref(), id).await;
            Err(error.into())
        }
    }
}

async fn download(
    Storage(db): Storage,
    Files(files): Files,
    Principal(actor): Principal,
    Path((unit, id)): Path<(UnitId, FileId)>,
) -> Result<Response> {
    require_unit_member(db.as_ref(), actor, unit).await?;
    let file = db.get_file(unit, id).await?.ok_or(Error::NotFound)?;
    let bytes = files.get(key(id)).await?;
    // ASCII fallback keeps arbitrary user filenames out of response headers.
    let filename: String = file
        .filename
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " ._-".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
            (header::CACHE_CONTROL, "private, no-store".to_owned()),
        ],
        bytes,
    )
        .into_response())
}

async fn icon(
    Storage(db): Storage,
    Files(files): Files,
    Path((unit, id)): Path<(UnitId, FileId)>,
) -> Result<Response> {
    let file = db
        .get_file(unit, id)
        .await?
        .filter(|file| file.is_icon)
        .ok_or(Error::NotFound)?;
    Ok((
        [
            (header::CONTENT_TYPE, file.content_type),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
            (header::CACHE_CONTROL, "public, max-age=300".to_owned()),
        ],
        files.get(key(id)).await?,
    )
        .into_response())
}

async fn banner(
    Storage(db): Storage,
    Files(files): Files,
    Path((unit, id)): Path<(UnitId, FileId)>,
) -> Result<Response> {
    let file = db
        .get_file(unit, id)
        .await?
        .filter(|file| file.is_banner)
        .ok_or(Error::NotFound)?;
    Ok((
        [
            (header::CONTENT_TYPE, file.content_type),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
            (header::CACHE_CONTROL, "public, max-age=300".to_owned()),
        ],
        files.get(key(id)).await?,
    )
        .into_response())
}

async fn remove(
    Storage(db): Storage,
    Files(files): Files,
    Principal(actor): Principal,
    Path((unit, id)): Path<(UnitId, FileId)>,
) -> Result<StatusCode> {
    db.delete_file(require_user(actor)?, unit, id).await?;
    cleanup(files.as_ref(), id).await;
    Ok(StatusCode::NO_CONTENT)
}

async fn cleanup(files: &dyn FileStorage, id: FileId) {
    if let Err(error) = files.delete(key(id)).await {
        tracing::error!(%id, ?error, "Orphaned file requires storage cleanup");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icons_reject_active_content_and_invalid_images() {
        normalize_image(b"<svg xmlns='http://www.w3.org/2000/svg'></svg>", false)
            .expect_err("SVG is rejected");
        normalize_image(b"\x89PNG\r\n\x1a\ninvalid", false).expect_err("Malformed PNG is rejected");
    }
    #[test]
    fn banners_allow_larger_images_and_preserve_landscape_ratio() {
        let image = image::DynamicImage::new_rgb8(2400, 1200);
        let mut input = Cursor::new(Vec::new());
        image
            .write_to(&mut input, image::ImageFormat::Png)
            .expect("fixture");
        normalize_image(input.get_ref(), false).expect_err("too large for icon");
        let output = normalize_image(input.get_ref(), true).expect("banner");
        let decoded = image::load_from_memory(&output).expect("decode");
        assert_eq!((decoded.width(), decoded.height()), (1920, 960));
    }
    #[test]
    fn icons_are_resized_and_reencoded_as_png() {
        let image = image::DynamicImage::new_rgb8(1024, 512);
        let mut input = Cursor::new(Vec::new());
        image
            .write_to(&mut input, image::ImageFormat::Jpeg)
            .expect("encode fixture");
        let output = normalize_image(input.get_ref(), false).expect("normalize");
        assert_eq!(
            image::guess_format(&output).expect("format"),
            image::ImageFormat::Png
        );
        let decoded = image::load_from_memory(&output).expect("decode");
        assert_eq!((decoded.width(), decoded.height()), (512, 256));
    }
}
