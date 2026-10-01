use tactica_files::{Bytes, FileKey, FileStorage, FileStorageError};
use uuid::Uuid;

/// Both real backends must provide the same observable behavior, including when
/// read through a newly constructed client pointing at the same storage.
pub async fn storage_contract(writer: &dyn FileStorage, reader: &dyn FileStorage) {
    let first = FileKey::from_uuid(Uuid::now_v7());
    let second = FileKey::from_uuid(Uuid::now_v7());
    assert!(matches!(
        reader.get(first).await,
        Err(FileStorageError::NotFound)
    ));
    writer.delete(first).await.expect("delete absent object");

    // Cross Garage's block boundary and cover arbitrary binary data, not just UTF-8.
    let payload = Bytes::from(
        (0..1024 * 1024)
            .map(|n| u8::try_from(n % 256).expect("byte"))
            .collect::<Vec<_>>(),
    );
    writer
        .put(first, payload.clone())
        .await
        .expect("upload binary file");
    assert_eq!(
        reader.get(first).await.expect("read from separate client"),
        payload
    );
    writer
        .put(second, Bytes::from_static(b"independent"))
        .await
        .expect("second upload");

    writer
        .put(first, Bytes::from_static(b"replacement"))
        .await
        .expect("replace object");
    assert_eq!(reader.get(first).await.expect("replacement"), "replacement");
    assert_eq!(
        reader.get(second).await.expect("other object unchanged"),
        "independent"
    );

    writer.delete(first).await.expect("delete existing object");
    writer.delete(first).await.expect("repeat delete");
    assert!(matches!(
        reader.get(first).await,
        Err(FileStorageError::NotFound)
    ));
    assert_eq!(
        reader
            .get(second)
            .await
            .expect("other object survives deletion"),
        "independent"
    );
    writer.delete(second).await.expect("cleanup second object");

    // Empty payloads are valid at the storage layer, even though the API rejects them.
    writer.put(first, Bytes::new()).await.expect("empty object");
    assert!(reader.get(first).await.expect("empty bytes").is_empty());
    writer.delete(first).await.expect("cleanup empty object");
}
