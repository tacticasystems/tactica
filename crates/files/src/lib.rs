//! Backend-independent storage for bounded file uploads.
//! Authorization, filenames and MIME types belong to the application/database.
use async_trait::async_trait;
pub use bytes::Bytes;
use object_store::{ObjectStoreExt, aws::AmazonS3Builder, local::LocalFileSystem, path::Path};
use uuid::Uuid;

/// An opaque, server-generated key. User filenames are never storage paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileKey(Uuid);

impl FileKey {
    #[must_use]
    pub const fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    fn path(self) -> Path {
        Path::from(format!("files/{}", self.0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileStorageError {
    #[error("File not found")]
    NotFound,
    #[error("File storage failed: {0}")]
    Backend(#[source] object_store::Error),
}
impl From<object_store::Error> for FileStorageError {
    fn from(error: object_store::Error) -> Self {
        match error {
            object_store::Error::NotFound { .. } => Self::NotFound,
            error => Self::Backend(error),
        }
    }
}

/// Implementations atomically replace complete objects. Delete is idempotent.
/// Callers must bound upload sizes before allocating the body.
#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn put(&self, key: FileKey, bytes: Bytes) -> Result<(), FileStorageError>;
    async fn get(&self, key: FileKey) -> Result<Bytes, FileStorageError>;
    async fn delete(&self, key: FileKey) -> Result<(), FileStorageError>;
}

#[derive(Debug)]
pub struct FilesystemStorage(LocalFileSystem);
impl FilesystemStorage {
    /// The root must already exist and be writable only by the service account.
    pub fn new(root: impl AsRef<std::path::Path>) -> Result<Self, FileStorageError> {
        Ok(Self(LocalFileSystem::new_with_prefix(root)?))
    }
}

pub struct S3Storage(object_store::aws::AmazonS3);
impl S3Storage {
    /// Configure credentials and endpoints explicitly, including Garage endpoints.
    pub fn new(builder: AmazonS3Builder) -> Result<Self, FileStorageError> {
        // Single-object DELETE preserves idempotence on Garage, whose bulk
        // delete response reports NoSuchKey as an error for missing objects.
        Ok(Self(
            builder
                .with_disable_bulk_delete(true)
                .with_virtual_hosted_style_request(false)
                .build()?,
        ))
    }

    /// Uses the AWS environment/credential provider chain. The bucket must exist.
    pub fn from_env(bucket: &str, region: &str) -> Result<Self, FileStorageError> {
        Self::new(
            AmazonS3Builder::from_env()
                .with_bucket_name(bucket)
                .with_region(region),
        )
    }
}

macro_rules! implement_storage {
    ($backend:ty) => {
        #[async_trait]
        impl FileStorage for $backend {
            async fn put(&self, key: FileKey, bytes: Bytes) -> Result<(), FileStorageError> {
                self.0.put(&key.path(), bytes.into()).await?;
                Ok(())
            }
            async fn get(&self, key: FileKey) -> Result<Bytes, FileStorageError> {
                Ok(self.0.get(&key.path()).await?.bytes().await?)
            }
            async fn delete(&self, key: FileKey) -> Result<(), FileStorageError> {
                match self.0.delete(&key.path()).await {
                    Ok(()) | Err(object_store::Error::NotFound { .. }) => Ok(()),
                    Err(error) => Err(error.into()),
                }
            }
        }
    };
}
implement_storage!(FilesystemStorage);
implement_storage!(S3Storage);
