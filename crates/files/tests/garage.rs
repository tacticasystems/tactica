mod support;

use object_store::aws::AmazonS3Builder;
use tactica_files::{Bytes, FileKey, FileStorage, FileStorageError, S3Storage};
use testcontainers::{
    CopyTargetOptions, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor, wait::HttpWaitStrategy},
    runners::AsyncRunner,
};
use uuid::Uuid;

// Disposable test credentials, never read from the developer's AWS environment.
const ACCESS_KEY: &str = "GK00000000000000000000000000000001";
const SECRET_KEY: &str = "0000000000000000000000000000000000000000000000000000000000000001";
const BUCKET: &str = "tactica-test";
const CONFIG: &str = r#"
metadata_dir = "/tmp/garage/meta"
data_dir = "/tmp/garage/data"
db_engine = "lmdb"
replication_factor = 1
rpc_bind_addr = "0.0.0.0:3901"
rpc_public_addr = "127.0.0.1:3901"
rpc_secret = "1111111111111111111111111111111111111111111111111111111111111111"
[s3_api]
api_bind_addr = "0.0.0.0:3900"
s3_region = "garage"
[admin]
api_bind_addr = "0.0.0.0:3903"
"#;

fn storage(endpoint: &str, secret: &str) -> S3Storage {
    S3Storage::new(
        AmazonS3Builder::new()
            .with_bucket_name(BUCKET)
            .with_region("garage")
            .with_endpoint(endpoint)
            .with_allow_http(true) // Only the ephemeral local test container.
            .with_access_key_id(ACCESS_KEY)
            .with_secret_access_key(secret),
    )
    .expect("Garage S3 client")
}

#[tokio::test(flavor = "multi_thread")]
async fn garage_s3_storage_contract() {
    // Garage 2.3 initializes the single-node layout, bucket and key before
    // becoming healthy. No fixed host ports, shared buckets or external services.
    let container = GenericImage::new("dxflrs/garage", "v2.3.0")
        .with_exposed_port(3900.tcp())
        .with_exposed_port(3903.tcp())
        .with_wait_for(WaitFor::http(
            HttpWaitStrategy::new("/health")
                .with_port(3903.tcp())
                .with_expected_status_code(200_u16),
        ))
        .with_copy_to(
            CopyTargetOptions::new("/etc/garage.toml").with_mode(0o600),
            CONFIG.as_bytes().to_vec(),
        )
        .with_env_var("GARAGE_DEFAULT_ACCESS_KEY", ACCESS_KEY)
        .with_env_var("GARAGE_DEFAULT_SECRET_KEY", SECRET_KEY)
        .with_env_var("GARAGE_DEFAULT_BUCKET", BUCKET)
        .with_cmd(["/garage", "server", "--single-node", "--default-bucket"])
        .with_startup_timeout(std::time::Duration::from_secs(60))
        .start()
        .await
        .expect("start Garage");
    let host = container.get_host().await.expect("Garage host");
    let port = container
        .get_host_port_ipv4(3900)
        .await
        .expect("Garage S3 port");
    let endpoint = format!("http://{host}:{port}");
    let writer = storage(&endpoint, SECRET_KEY);
    let reader = storage(&endpoint, SECRET_KEY);
    support::storage_contract(&writer, &reader).await;

    let invalid = storage(&endpoint, "invalid-test-secret");
    let key = FileKey::from_uuid(Uuid::now_v7());
    assert!(matches!(
        invalid.put(key, Bytes::from_static(b"denied")).await,
        Err(FileStorageError::Backend(_))
    ));
    assert!(matches!(
        reader.get(key).await,
        Err(FileStorageError::NotFound)
    ));
    // Dropping the container tears down its temporary data as in the PostgreSQL tests.
}
