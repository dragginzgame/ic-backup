//! Fresh exact accounting cases adapted from Canic's pending and receipt regressions.

use super::*;

fn binding_request() -> OperationBindingRequest {
    OperationBindingRequest {
        intent: "ab".repeat(32),
        operation_sequence: 7,
        network: "cd".repeat(32),
        caller: "2vxsx-fae".into(),
        target: "aaaaa-aa".into(),
        release: "ef".repeat(32),
        request: "01".repeat(32),
    }
}
fn authority(mutations: u32, observations: u32) -> AttemptAuthorityRecord {
    AttemptAuthorityRecord::new(
        OperationBindingRecord::new(&binding_request()).expect("binding"),
        AttemptBudgetRecord::new(mutations, observations).expect("budget"),
    )
}
fn direct(attempt: u32, outcome: MutationOutcomeRecord) -> MutationReceiptRequest {
    MutationReceiptRequest {
        attempt,
        request: "01".repeat(32),
        outcome,
        evidence: "23".repeat(32),
    }
}
fn observation(attempt: u32, outcome: ObservationOutcomeRecord) -> ObservationReceiptRequest {
    ObservationReceiptRequest {
        attempt,
        request: "45".repeat(32),
        outcome,
        evidence: "67".repeat(32),
    }
}

#[test]
fn canonical_authority_hash_binds_every_identity_and_original_budget() {
    let base = authority(2, 2);
    assert_eq!(base.binding().intent(), "ab".repeat(32));
    assert_eq!(base.binding().operation_sequence(), 7);
    assert_eq!(base.binding().network(), "cd".repeat(32));
    assert_eq!(base.binding().caller(), "2vxsx-fae");
    assert_eq!(base.binding().target(), "aaaaa-aa");
    assert_eq!(base.binding().release(), "ef".repeat(32));
    assert_eq!(base.binding().request(), "01".repeat(32));
    assert_eq!(
        base.digest().hash(),
        "83bfc98d1310e756e20500d80220a08f2e15ab93ef064fb15d9e7b9674dd3bfb"
    );
    let mut upper = binding_request();
    upper.intent.make_ascii_uppercase();
    upper.network.make_ascii_uppercase();
    upper.release.make_ascii_uppercase();
    upper.caller.make_ascii_uppercase();
    upper.target.make_ascii_uppercase();
    let upper = AttemptAuthorityRecord::new(
        OperationBindingRecord::new(&upper).expect("normalize"),
        base.budget().clone(),
    );
    assert_eq!(upper, base);
    assert_eq!(upper.digest(), base.digest());
    for field in [
        "intent", "sequence", "network", "caller", "target", "release", "request",
    ] {
        let mut request = binding_request();
        match field {
            "intent" => request.intent = "89".repeat(32),
            "sequence" => request.operation_sequence = u64::MAX,
            "network" => request.network = "89".repeat(32),
            "caller" => request.caller = "aaaaa-aa".into(),
            "target" => request.target = "2vxsx-fae".into(),
            "release" => request.release = "89".repeat(32),
            "request" => request.request = "89".repeat(32),
            _ => unreachable!(),
        }
        assert_ne!(
            AttemptAuthorityRecord::new(
                OperationBindingRecord::new(&request).expect("changed binding"),
                base.budget().clone()
            )
            .digest(),
            base.digest(),
            "{field}"
        );
    }
    assert_ne!(authority(3, 2).digest(), base.digest());
    assert_ne!(authority(2, 3).digest(), base.digest());
    let record = AttemptJournalRecord::new(base.clone());
    assert_eq!(
        serde_json::from_slice::<AttemptJournalRecord>(
            &serde_json::to_vec(&record).expect("encode")
        )
        .expect("decode"),
        record
    );
}

#[test]
fn pending_mutations_block_blind_retry_and_uncertain_observations_spend_allowance() {
    let mut record = AttemptJournalRecord::new(authority(2, 2));
    assert_eq!(record.reserve_mutation().expect("claim before effect"), 1);
    let original = record.clone();
    assert!(matches!(
        record.reserve_mutation(),
        Err(AttemptJournalRecordError::MutationPending { attempt: 1 })
    ));
    assert_eq!(record, original);
    assert_eq!(
        record
            .reserve_observation(1, &"45".repeat(32))
            .expect("bounded reconciliation"),
        2
    );
    let original = record.clone();
    assert!(matches!(
        record.record_mutation(direct(1, MutationOutcomeRecord::Applied)),
        Err(AttemptJournalRecordError::ObservationPending { attempt: 2 })
    ));
    assert_eq!(record, original);
    record
        .record_observation(observation(2, ObservationOutcomeRecord::Uncertain))
        .expect("unresolved evidence");
    assert_eq!(record.view().pending_mutation, Some(1));
    assert_eq!(record.view().observations_used, 1);
    assert_eq!(record.view().mutations_remaining, 1);
    assert!(matches!(
        record.reserve_mutation(),
        Err(AttemptJournalRecordError::MutationPending { .. })
    ));
    assert_eq!(
        record
            .reserve_observation(1, &"45".repeat(32))
            .expect("next separately bounded observation"),
        3
    );
    record
        .record_observation(observation(3, ObservationOutcomeRecord::NotApplied))
        .expect("qualified nonapplication");
    assert_eq!(record.reserve_mutation().expect("new consumed attempt"), 4);
    record
        .record_mutation(direct(4, MutationOutcomeRecord::NotApplied))
        .expect("no refund");
    assert_eq!(record.view().mutations_used, 2);
    assert_eq!(record.view().observations_used, 2);
    assert_eq!(record.view().mutations_remaining, 0);
    assert!(matches!(
        record.reserve_mutation(),
        Err(AttemptJournalRecordError::MutationBudgetExhausted)
    ));
    assert!(!record.view().applied);
    let bytes = serde_json::to_vec(&record).expect("history");
    assert_eq!(
        serde_json::from_slice::<AttemptJournalRecord>(&bytes).expect("replay exact history"),
        record
    );
}

#[test]
fn observation_resolves_an_exhausted_mutation_without_authorizing_another_call() {
    let mut record = AttemptJournalRecord::new(authority(1, 1));
    record.reserve_mutation().expect("last mutation authority");
    assert_eq!(record.view().mutations_remaining, 0);
    let number = record
        .reserve_observation(1, &"45".repeat(32))
        .expect("observation remains bounded independently");
    record
        .record_observation(observation(number, ObservationOutcomeRecord::Applied))
        .expect("qualified success");
    assert!(record.view().applied);
    assert_eq!(record.view().pending_mutation, None);
    assert_eq!(record.view().pending_observation, None);
    assert_eq!(record.view().mutations_used, 1);
    assert!(matches!(
        record.reserve_mutation(),
        Err(AttemptJournalRecordError::AlreadyApplied)
    ));
    assert!(matches!(
        record.reserve_observation(1, &"45".repeat(32)),
        Err(AttemptJournalRecordError::AlreadyApplied)
    ));
}

#[test]
fn wrong_receipt_identities_and_stale_observations_preserve_all_evidence() {
    let mut record = AttemptJournalRecord::new(authority(2, 2));
    assert!(matches!(
        record.record_mutation(direct(1, MutationOutcomeRecord::Applied)),
        Err(AttemptJournalRecordError::NoPendingMutation)
    ));
    record.reserve_mutation().expect("reserve");
    let original = record.clone();
    assert!(matches!(
        record.record_mutation(direct(2, MutationOutcomeRecord::Applied)),
        Err(AttemptJournalRecordError::AttemptMismatch {
            expected: 1,
            actual: 2
        })
    ));
    let mut receipt = direct(1, MutationOutcomeRecord::Applied);
    receipt.request = "89".repeat(32);
    assert!(matches!(
        record.record_mutation(receipt),
        Err(AttemptJournalRecordError::RequestMismatch)
    ));
    assert_eq!(record, original);
    record
        .reserve_observation(1, &"45".repeat(32))
        .expect("observation");
    let original = record.clone();
    let mut reply = observation(2, ObservationOutcomeRecord::Applied);
    reply.request = "89".repeat(32);
    assert!(matches!(
        record.record_observation(reply),
        Err(AttemptJournalRecordError::RequestMismatch)
    ));
    assert_eq!(record, original);
    record
        .record_observation(observation(2, ObservationOutcomeRecord::NotApplied))
        .expect("resolve original");
    record.reserve_mutation().expect("new attempt three");
    let original = record.clone();
    assert!(matches!(
        record.record_observation(observation(2, ObservationOutcomeRecord::Applied)),
        Err(AttemptJournalRecordError::NoPendingObservation)
    ));
    assert!(matches!(
        record.reserve_observation(1, &"45".repeat(32)),
        Err(AttemptJournalRecordError::AttemptMismatch {
            expected: 3,
            actual: 1
        })
    ));
    assert_eq!(record, original);
}

#[test]
fn zero_and_maximum_limits_use_exact_bounded_history_without_refunds() {
    assert!(matches!(
        AttemptBudgetRecord::new(u32::MAX, 1),
        Err(AttemptJournalRecordError::BudgetTooLarge)
    ));
    assert!(matches!(
        AttemptBudgetRecord::new(1, MAX_OPERATION_ATTEMPTS),
        Err(AttemptJournalRecordError::BudgetTooLarge)
    ));
    let mut zero = AttemptJournalRecord::new(authority(0, 0));
    assert!(matches!(
        zero.reserve_mutation(),
        Err(AttemptJournalRecordError::MutationBudgetExhausted)
    ));
    let mut record = AttemptJournalRecord::new(authority(MAX_OPERATION_ATTEMPTS, 0));
    for _ in 0..MAX_OPERATION_ATTEMPTS {
        let attempt = record.reserve_mutation().expect("bounded reserve");
        record
            .record_mutation(direct(attempt, MutationOutcomeRecord::NotApplied))
            .expect("consumed authority remains spent");
    }
    assert_eq!(record.view().history_len, MAX_ATTEMPT_EVENTS);
    let bytes = serde_json::to_vec_pretty(&record).expect("full bounded history");
    assert!(bytes.len() as u64 <= MAX_ATTEMPT_JOURNAL_BYTES);
    assert_eq!(
        serde_json::from_slice::<AttemptJournalRecord>(&bytes).expect("decode exact event bound"),
        record
    );
    let mut value = serde_json::to_value(&record).expect("json");
    value["events"]
        .as_array_mut()
        .expect("events")
        .push(serde_json::json!({"event":"mutation_reserved","attempt":1025}));
    assert!(
        serde_json::from_value::<AttemptJournalRecord>(value)
            .expect_err("event bound")
            .to_string()
            .contains("history exceeds")
    );
}

#[test]
fn decoding_requires_closed_fields_and_valid_chronology() {
    let record = AttemptJournalRecord::new(authority(2, 2));
    let value = serde_json::to_value(&record).expect("json");
    for (container, fields) in [
        ("", vec!["version", "authority", "events"]),
        ("/authority", vec!["binding", "budget"]),
        (
            "/authority/binding",
            vec![
                "intent",
                "operation_sequence",
                "network",
                "caller",
                "target",
                "release",
                "request",
            ],
        ),
        ("/authority/budget", vec!["mutations", "observations"]),
    ] {
        for field in fields {
            let mut invalid = value.clone();
            invalid
                .pointer_mut(container)
                .expect("container")
                .as_object_mut()
                .expect("object")
                .remove(field);
            assert!(
                serde_json::from_value::<AttemptJournalRecord>(invalid).is_err(),
                "missing {field}"
            );
        }
    }
    for event in [
        serde_json::json!({"event":"mutation_reserved","attempt":2}),
        serde_json::json!({"event":"mutation_reserved","attempt":1,"extra":true}),
        serde_json::json!({"event":"mutation_resolved","mutation":1,"outcome":"applied","evidence":"ab".repeat(32)}),
        serde_json::json!({"event":"observation_reserved","attempt":1,"mutation":1,"request":"ab".repeat(32)}),
        serde_json::json!({"event":"observation_recorded","observation":1,"request":"ab".repeat(32),"outcome":"applied","evidence":"ab".repeat(32)}),
    ] {
        let mut invalid = value.clone();
        invalid["events"] = serde_json::json!([event]);
        assert!(serde_json::from_value::<AttemptJournalRecord>(invalid).is_err());
    }
    for (pointer, bad) in [
        ("/version", serde_json::json!(2)),
        ("/authority/binding/caller", serde_json::json!("bad")),
        ("/authority/binding/network", serde_json::json!("bad")),
        ("/authority/budget/mutations", serde_json::json!(1025)),
    ] {
        let mut invalid = value.clone();
        *invalid.pointer_mut(pointer).expect("field") = bad;
        assert!(serde_json::from_value::<AttemptJournalRecord>(invalid).is_err());
    }
    let mut invalid = value.clone();
    invalid["projection"] = serde_json::json!({"applied":true});
    assert!(serde_json::from_value::<AttemptJournalRecord>(invalid).is_err());
    let duplicate = serde_json::to_string(&value).expect("json").replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(serde_json::from_str::<AttemptJournalRecord>(&duplicate).is_err());
    let mut valid = record.clone();
    valid.reserve_mutation().expect("claim");
    valid
        .record_mutation(direct(1, MutationOutcomeRecord::Applied))
        .expect("receipt");
    let mut invalid = serde_json::to_value(valid).expect("json");
    invalid["events"]
        .as_array_mut()
        .expect("events")
        .push(serde_json::json!({"event":"mutation_reserved","attempt":2}));
    assert!(serde_json::from_value::<AttemptJournalRecord>(invalid).is_err());
}
