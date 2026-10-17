use crate::{
    model::{
        ic_mutation::{
            IcMutationAcknowledgement, IcMutationAcknowledgementError, IcMutationRequest,
            MAX_IC_MUTATION_REPLY_BYTES,
        },
        ic_request::IcManagementMethodRecord,
        ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_REPLY_BYTES,
        ic_snapshot_transfer_read::{IcSnapshotTransferReadError, IcSnapshotTransferReadResponse},
    },
    test_support::ic_mutation::{input, original},
};

#[test]
fn shared_fields_keep_distinct_lane_limits_order_and_redaction() {
    let original = original(IcManagementMethodRecord::StopCanister);
    let request =
        IcMutationRequest::new(&original.plan, 7, &original.journal, &original.payload).unwrap();
    let mut fields = input(&request);
    fields.target = fields.target.to_ascii_uppercase();
    fields.reply = vec![b'X'; MAX_IC_MUTATION_REPLY_BYTES + 1];
    assert_eq!(
        IcMutationAcknowledgement::new(fields.clone()).unwrap_err(),
        IcMutationAcknowledgementError::ReplyTooLarge
    );
    let transfer = IcSnapshotTransferReadResponse::new(fields.clone()).unwrap();
    assert_eq!(transfer.input().target, original.payload.target());
    assert_eq!(transfer.input().reply, fields.reply);
    assert!(!format!("{transfer:?}").contains("XXXX"));
    fields.reply.resize(MAX_IC_SNAPSHOT_DATA_REPLY_BYTES, b'X');
    assert!(IcSnapshotTransferReadResponse::new(fields.clone()).is_ok());
    fields.reply.push(b'X');
    fields.target = "invalid principal".into();
    fields.mutation_attempt = 0;
    assert_eq!(
        IcMutationAcknowledgement::new(fields.clone()).unwrap_err(),
        IcMutationAcknowledgementError::InvalidAttempt
    );
    assert!(matches!(
        IcSnapshotTransferReadResponse::new(fields.clone()),
        Err(IcSnapshotTransferReadError::InvalidAttempt)
    ));
    fields.mutation_attempt = 1;
    assert_eq!(
        IcMutationAcknowledgement::new(fields.clone()).unwrap_err(),
        IcMutationAcknowledgementError::ReplyTooLarge
    );
    assert!(matches!(
        IcSnapshotTransferReadResponse::new(fields),
        Err(IcSnapshotTransferReadError::ReplyTooLarge)
    ));
}
