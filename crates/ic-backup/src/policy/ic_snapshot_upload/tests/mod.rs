use super::*;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_mutation::IcMutationAcknowledgementInput,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::{IcSnapshotUploadReplyKind, IcSnapshotUploadRequest},
    },
    test_support::ic_snapshot_upload::{
        DESTINATION_ID, SOURCE_ID, TARGET, raw, source_plan, upload_plan,
    },
};
use ic_management_canister_types::{SnapshotDataKind, UploadCanisterSnapshotMetadataResult};

fn input(
    request: &IcSnapshotUploadAttempt<'_, '_>,
    reply: Vec<u8>,
) -> IcMutationAcknowledgementInput {
    IcMutationAcknowledgementInput {
        authority: request.authority().digest(),
        mutation_attempt: request.mutation_attempt(),
        context: request.plan().context().clone(),
        target: request.payload().target().into(),
        reply,
        evidence: ArtifactChecksumRecord::from_bytes(b"passive provider declaration"),
    }
}

#[test]
fn metadata_and_data_association_preserve_pending_spending_and_original_evidence() {
    let source_plan = source_plan();
    let metadata_request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let metadata =
        IcSnapshotMetadataReply::decode(&metadata_request, &raw("upload-source")).unwrap();
    let checksum = ArtifactChecksumRecord::from_bytes(b"declared tree");
    let upload = IcSnapshotUploadRequest::metadata(&source_plan, &metadata, &checksum).unwrap();
    let data = IcSnapshotUploadRequest::data(
        &upload,
        DESTINATION_ID,
        SnapshotDataKind::WasmModule { offset: 5, size: 3 },
        SOURCE_ID,
    )
    .unwrap();
    for payload in [&upload, &data] {
        let plan = upload_plan(payload);
        let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
        journal.reserve_mutation().unwrap();
        let before = journal.clone();
        let request = IcSnapshotUploadAttempt::new(&plan, 7, &journal, payload).unwrap();
        let bytes = if matches!(
            payload.kind(),
            crate::model::ic_snapshot_upload::IcSnapshotUploadKind::Metadata
        ) {
            candid::encode_one(UploadCanisterSnapshotMetadataResult {
                snapshot_id: DESTINATION_ID.to_vec(),
            })
            .unwrap()
        } else {
            b"DIDL\0\0".to_vec()
        };
        let acknowledgement = IcMutationAcknowledgement::new(input(&request, bytes)).unwrap();
        let view = validate_acknowledgement(&request, &journal, &acknowledgement).unwrap();
        assert_eq!(
            view.acknowledgement().input().authority,
            acknowledgement.input().authority
        );
        assert_eq!(
            view.acknowledgement().input().evidence,
            acknowledgement.input().evidence
        );
        assert_eq!(
            view.acknowledgement().input().reply,
            acknowledgement.input().reply
        );
        assert_eq!(
            view.reply().payload_checksum(),
            &ArtifactChecksumRecord::from_bytes(&acknowledgement.input().reply)
        );
        let expected = if matches!(
            payload.kind(),
            crate::model::ic_snapshot_upload::IcSnapshotUploadKind::Metadata
        ) {
            IcSnapshotUploadReplyKind::Metadata {
                snapshot_id: DESTINATION_ID.to_vec(),
            }
        } else {
            IcSnapshotUploadReplyKind::DataAcknowledgement
        };
        assert_eq!(view.reply().kind(), &expected);
        assert_eq!(journal, before);
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().mutations_remaining, 0);
        assert!(!journal.view().applied);
        assert!(journal.reserve_mutation().is_err());
        assert_eq!(journal, before);
    }
}

#[test]
fn actual_claim_mismatches_invalid_wire_and_lost_recovery_reply_stay_pending() {
    let source_plan = source_plan();
    let metadata_request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let metadata =
        IcSnapshotMetadataReply::decode(&metadata_request, &raw("upload-source-absent")).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &source_plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"tree"),
    )
    .unwrap();
    let plan = upload_plan(&upload);
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(7).unwrap());
    journal.reserve_mutation().unwrap();
    let request = IcSnapshotUploadAttempt::new(&plan, 7, &journal, &upload).unwrap();
    let reply = candid::encode_one(UploadCanisterSnapshotMetadataResult {
        snapshot_id: DESTINATION_ID.to_vec(),
    })
    .unwrap();
    for change in 0..7 {
        let before = journal.clone();
        let mut fields = input(&request, reply.clone());
        match change {
            0 => fields.authority = ArtifactChecksumRecord::from_bytes(b"wrong"),
            1 => fields.mutation_attempt = 2,
            2..=4 => {
                let mut context = serde_json::to_value(&fields.context).unwrap();
                let (field, value) = match change {
                    2 => ("network", "12".repeat(32)),
                    3 => ("caller", "aaaaa-aa".into()),
                    _ => ("release", "12".repeat(32)),
                };
                context[field] = serde_json::json!(value);
                fields.context = serde_json::from_value(context).unwrap();
            }
            5 => fields.target = "aaaaa-aa".into(),
            _ => fields.reply = b"DIDL\0\0".to_vec(),
        }
        let acknowledgement = IcMutationAcknowledgement::new(fields).unwrap();
        let error = validate_acknowledgement(&request, &journal, &acknowledgement).unwrap_err();
        assert!(match change {
            0 => matches!(error, IcSnapshotUploadAssociationError::AuthorityMismatch),
            1 => matches!(error, IcSnapshotUploadAssociationError::AttemptMismatch),
            2..=4 => matches!(error, IcSnapshotUploadAssociationError::ContextMismatch),
            5 => matches!(error, IcSnapshotUploadAssociationError::TargetMismatch),
            _ => matches!(error, IcSnapshotUploadAssociationError::Reply(_)),
        });
        assert_eq!(journal, before);
    }
    let acknowledgement = IcMutationAcknowledgement::new(input(&request, reply)).unwrap();
    journal
        .reserve_observation(
            1,
            ArtifactChecksumRecord::from_bytes(b"original observation").hash(),
        )
        .unwrap();
    let before = journal.clone();
    assert!(matches!(
        validate_acknowledgement(&request, &journal, &acknowledgement),
        Err(IcSnapshotUploadAssociationError::Reservation(
            IcSnapshotUploadAttemptError::ObservationPending
        ))
    ));
    assert_eq!(journal, before);
    assert_eq!(journal.view().pending_observation, Some(2));
    assert_eq!(journal.view().observations_remaining, 0);
}
