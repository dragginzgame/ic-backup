//! Deterministic host IO and single-step local effects.

pub mod artifacts;
pub mod persistence;

pub(crate) fn directory_creation_io(
    error: ic_host_fs::durable::DirectoryCreationError,
) -> std::io::Error {
    use ic_host_fs::durable::DirectoryCreationError;
    let kind = match &error {
        DirectoryCreationError::BeforeCreation(source)
        | DirectoryCreationError::AfterCreation(source) => source.kind(),
    };
    std::io::Error::new(kind, error)
}
