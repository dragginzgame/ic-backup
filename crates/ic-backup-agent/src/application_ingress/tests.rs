use super::*;
#[test]
fn passive_reply_debug_never_renders_opaque_reply_bytes() {
    let response = ApplicationIngressResponse {
        request: ArtifactChecksumRecord::from_bytes(b"original request"),
        request_id: RequestId::new(&[7; 32]),
        reply: b"private application reply".to_vec(),
        evidence: ArtifactChecksumRecord::from_bytes(b"qualified by its owner"),
    };
    let debug = format!("{response:?}");
    assert!(!debug.contains("private application reply"));
    assert!(!debug.contains(&format!("{:?}", response.reply())));
    assert!(debug.contains("reply_bytes: 25"));
}
