//! Original binding identity and raw-size admission before workflow continuation.
use super::*;
use crate::tests::support;
use ic_backup::model::{
    artifacts::ArtifactChecksumRecord,
    execution_workflow::{
        ExecutionStageBindingRecord, ExecutionWorkflowRecord, MAX_EXECUTION_STAGE_BYTES,
    },
};
use ic_host_artifacts::artifact::ArtifactError;
use std::fs;

fn original() -> ExecutionStageBindingRecord {
    let plan = support::plan(
        "renrk-eyaaa-aaaaa-aaada-cai",
        &ArtifactChecksumRecord::from_bytes(b"original"),
        0,
    );
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    ExecutionStageBindingRecord::new(&workflow, 0, &plan, vec![]).unwrap()
}

#[test]
fn binding_reader_admits_exact_limit_and_refuses_larger_original_bytes() {
    let root = support::root("stage-binding-limit");
    let path = root.join("stage-binding.json");
    let binding = original();
    let mut bytes = serde_json::to_vec(&binding).unwrap();
    bytes.resize(usize::try_from(MAX_EXECUTION_STAGE_BYTES).unwrap(), b' ');
    fs::write(&path, &bytes).unwrap();
    assert_eq!(read_binding(&root, 0).unwrap(), binding);
    bytes.push(b' ');
    fs::write(&path, &bytes).unwrap();
    let error = read_binding(&root, 0).unwrap_err();
    assert!(matches!(
        error.get_ref().unwrap().downcast_ref::<ArtifactError>(),
        Some(ArtifactError::LimitExceeded { .. })
    ));
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn binding_reader_refuses_wrong_sequence_and_malformed_evidence_without_rewriting() {
    let root = support::root("stage-binding-identity");
    let path = root.join("stage-binding.json");
    let original = serde_json::to_vec(&original()).unwrap();
    fs::write(&path, &original).unwrap();
    assert_eq!(
        read_binding(&root, 1).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(fs::read(&path).unwrap(), original);
    let malformed = br#"{"version":1}"#;
    fs::write(&path, malformed).unwrap();
    let error = read_binding(&root, 0).unwrap_err();
    assert!(
        error
            .get_ref()
            .unwrap()
            .downcast_ref::<serde_json::Error>()
            .is_some()
    );
    assert_eq!(fs::read(path).unwrap(), malformed);
}
