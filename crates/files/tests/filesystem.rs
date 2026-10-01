mod support;
use tactica_files::FilesystemStorage;

#[tokio::test]
async fn filesystem_storage_contract() {
    let root = tempfile::tempdir().expect("temporary storage directory");
    let writer = FilesystemStorage::new(root.path()).expect("filesystem writer");
    let reader = FilesystemStorage::new(root.path()).expect("independent filesystem reader");
    support::storage_contract(&writer, &reader).await;
}

#[test]
fn filesystem_requires_an_existing_root() {
    let root = tempfile::tempdir().expect("temporary directory");
    FilesystemStorage::new(root.path().join("missing")).expect_err("missing root is rejected");
}
