//! Fixed byte identities protect retained evidence across internal consolidation.
use super::*;

#[test]
fn original_adapter_domains_and_chronological_attempt_framing_match_fixed_goldens() {
    let authority = ArtifactChecksumRecord::from_bytes(b"original authority");
    let id = RequestId::new(&[7; 32]);
    for (domain, attempts, expected) in [
        (
            b"ic-backup-agent.mutation-reply.v1\0".as_slice(),
            &[1][..],
            "a732fdccaeb6c2b1e9e983ab4504a4156b58c0572854937b7993a30ac954c003",
        ),
        (
            b"ic-backup-agent.transfer-read-reply.v1\0".as_slice(),
            &[1][..],
            "13d365e218b869e21871432ee2a6d4d76d7f5d7ebb7c1981a8f7028eacb5be8d",
        ),
        (
            b"ic-backup-agent.upload-reply.v1\0".as_slice(),
            &[1][..],
            "445589357d4bbf3728d67baea36316c7132dbf24377beb8ef037b1a6dae76dc5",
        ),
        (
            b"ic-backup-agent.observation-reply.v1\0".as_slice(),
            &[1, 2][..],
            "c0c162c89d5a5660f090566ba71b51b4d767dfce4ad5795f5cfeb6fd02ac927f",
        ),
    ] {
        assert_eq!(
            reply_evidence(domain, &authority, attempts, &id, b"retained reply").hash(),
            expected
        );
    }
}
