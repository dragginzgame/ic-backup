//! Bounded persisted-record sequences without allocating from untrusted hints.

use serde::{Deserialize, Deserializer, de};
use std::{fmt, marker::PhantomData};

pub(super) fn deserialize<'de, D, T, const LIMIT: usize>(
    deserializer: D,
    expected: impl fmt::Display,
    overflow: impl fmt::Display,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Visitor<T, X, E, const LIMIT: usize> {
        expected: X,
        overflow: E,
        item: PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>, X: fmt::Display, E: fmt::Display, const LIMIT: usize>
        de::Visitor<'de> for Visitor<T, X, E, LIMIT>
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}", self.expected)
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = Vec::new();
            while entries.len() < LIMIT {
                match sequence.next_element()? {
                    Some(entry) => entries.push(entry),
                    None => return Ok(entries),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(self.overflow));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(Visitor::<T, _, _, LIMIT> {
        expected,
        overflow,
        item: PhantomData,
    })
}
