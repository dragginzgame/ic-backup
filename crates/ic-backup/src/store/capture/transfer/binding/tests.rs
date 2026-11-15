use super::super::tests::{chunk, metadata, plan};
use super::*;

#[test]
fn commitment_binds_original_evidence_identity_geometry_and_canonical_chunks() {
    let p = plan(4);
    let mut m = metadata(9, 0, 5);
    m.wasm_chunk_store = vec![chunk(b"x"), chunk(b"")];
    let request = EvidenceRecord::new(b"request");
    let result = EvidenceRecord::new(b"certified metadata");
    let bind = |p: &crate::store::CapturePlan, id: &[u8], m: &_| {
        p.plan_transfer(id, m).unwrap().binding(&request, &result)
    };
    let expected = bind(&p, b"snapshot", &m);
    assert_eq!((expected.data_calls, expected.payload_ceiling), (7, 22));
    m.wasm_chunk_store.reverse();
    assert_eq!(bind(&p, b"snapshot", &m), expected);
    assert_ne!(bind(&plan(8), b"snapshot", &m), expected);
    assert_ne!(bind(&p, b"other", &m), expected);
    m.wasm_memory_size = 1;
    assert_ne!(bind(&p, b"snapshot", &m), expected);
    m.wasm_memory_size = 0;
    let schedule = p.plan_transfer(b"snapshot", &m).unwrap();
    assert_ne!(
        schedule.binding(&EvidenceRecord::new(b"other"), &result),
        expected
    );
    assert_ne!(
        schedule.binding(&request, &EvidenceRecord::new(b"other")),
        expected
    );
    let encoded = serde_json::to_vec(&expected).unwrap();
    assert_eq!(
        serde_json::from_slice::<TransferBindingRecord>(&encoded).unwrap(),
        expected
    );
}
