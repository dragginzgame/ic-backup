//! Exact schema mutations; record owners supply every required field and path.

use serde::de::DeserializeOwned;
use serde_json::{Value, json};

pub(crate) fn required_fields<T: DeserializeOwned>(value: &Value, path: &str, fields: &[&str]) {
    assert!(
        serde_json::from_value::<T>(value.clone()).is_ok(),
        "valid schema baseline"
    );
    for field in fields {
        let mut invalid = value.clone();
        assert!(
            invalid
                .pointer_mut(path)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(*field)
                .is_some(),
            "missing fixture field {path}/{field}"
        );
        assert!(
            serde_json::from_value::<T>(invalid).is_err(),
            "required {path}/{field}"
        );
    }
}

pub(crate) fn closed_fields<T: DeserializeOwned>(value: &Value, path: &str, fields: &[&str]) {
    required_fields::<T>(value, path, fields);
    let mut invalid = value.clone();
    assert!(
        invalid
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), json!(true))
            .is_none()
    );
    assert!(
        serde_json::from_value::<T>(invalid).is_err(),
        "unknown field at {path}"
    );
}

/// Keep the original first-serialized-version case and explicitly reject a
/// duplicate root version even when a nested record is serialized first.
pub(crate) fn duplicate_versions<T: DeserializeOwned>(value: &Value) {
    assert!(serde_json::from_value::<T>(value.clone()).is_ok());
    let field = format!("\"version\":{}", value.get("version").unwrap());
    let encoded = serde_json::to_string(value).unwrap();
    let first = encoded.replacen(&field, &format!("{field},{field}"), 1);
    let root = format!("{{{field},{}", encoded.strip_prefix('{').unwrap());
    for invalid in [first, root] {
        assert_ne!(
            invalid, encoded,
            "duplicate version mutation must change the fixture"
        );
        assert!(serde_json::from_str::<T>(&invalid).is_err());
    }
}
