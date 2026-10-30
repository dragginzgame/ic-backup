//! Persisted fields that must be present even when their value is null.
use serde::{Deserialize, Deserializer};

pub(super) fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}
