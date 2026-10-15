use super::*;
use ic_host_fs::durable::NamedWriteError;

#[test]
fn host_publication_failures_keep_exact_phase_source_and_cleanup_evidence() {
    let producer = PublicationError::from_host(NamedWriteError::Producer {
        source: io::Error::new(io::ErrorKind::BrokenPipe, "producer failed"),
        cleanup_error: Some(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "stage held",
        )),
    });
    assert!(matches!(
        &producer,
        PublicationError::Producer { source, cleanup_error: Some(cleanup) }
            if source.kind() == io::ErrorKind::BrokenPipe
                && cleanup.kind() == io::ErrorKind::PermissionDenied
    ));
    assert!(
        producer
            .to_string()
            .contains("staging cleanup failed: stage held")
    );
    assert_eq!(producer.source().unwrap().to_string(), "producer failed");

    let before = PublicationError::from_host(NamedWriteError::<io::Error>::BeforePublication {
        source: io::Error::new(io::ErrorKind::AlreadyExists, "original remains"),
        cleanup_error: Some(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "stage held",
        )),
    });
    assert!(matches!(
        &before,
        PublicationError::BeforePublication { source, cleanup_error: Some(cleanup) }
            if source.kind() == io::ErrorKind::AlreadyExists
                && cleanup.kind() == io::ErrorKind::PermissionDenied
    ));
    assert!(
        before
            .to_string()
            .contains("staging cleanup failed: stage held")
    );
    assert_eq!(before.source().unwrap().to_string(), "original remains");

    let after = PublicationError::from_host(NamedWriteError::<io::Error>::AfterPublication {
        source: io::Error::other("sync uncertain"),
    });
    assert!(matches!(
        &after,
        PublicationError::AfterPublication { source } if source.to_string() == "sync uncertain"
    ));
    assert!(
        after
            .to_string()
            .contains("output published but completion failed")
    );
    assert_eq!(after.source().unwrap().to_string(), "sync uncertain");
}
