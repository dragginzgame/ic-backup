//! Independent metadata wire vectors shared by fixture setup.

pub fn raw(name: &str) -> Vec<u8> {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    crate::tests::support::hex_bytes(
        cases.iter().find(|case| case["name"] == name).unwrap()["reply_hex"]
            .as_str()
            .unwrap(),
    )
}
