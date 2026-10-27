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
    deserialize_with::<D, T, _, _, _, _, LIMIT>(deserializer, expected, overflow, |_| {
        Ok::<(), std::convert::Infallible>(())
    })
}

/// Persisted sequences may check an aggregate bound before storing each item.
pub(super) fn deserialize_checked<'de, D, T, C, F, const LIMIT: usize>(
    deserializer: D,
    expected: impl fmt::Display,
    overflow: impl fmt::Display,
    check: F,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
    C: fmt::Display,
    F: FnMut(&T) -> Result<(), C>,
{
    deserialize_with::<D, T, _, _, C, F, LIMIT>(deserializer, expected, overflow, check)
}

fn deserialize_with<'de, D, T, X, E, C, F, const LIMIT: usize>(
    deserializer: D,
    expected: X,
    overflow: E,
    check: F,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
    X: fmt::Display,
    E: fmt::Display,
    C: fmt::Display,
    F: FnMut(&T) -> Result<(), C>,
{
    struct Visitor<T, X, E, C, F, const LIMIT: usize> {
        expected: X,
        overflow: E,
        check: F,
        item: PhantomData<(T, C)>,
    }
    impl<
        'de,
        T: Deserialize<'de>,
        X: fmt::Display,
        E: fmt::Display,
        C: fmt::Display,
        F: FnMut(&T) -> Result<(), C>,
        const LIMIT: usize,
    > de::Visitor<'de> for Visitor<T, X, E, C, F, LIMIT>
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}", self.expected)
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            mut self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = Vec::new();
            while entries.len() < LIMIT {
                match sequence.next_element()? {
                    Some(entry) => {
                        (self.check)(&entry).map_err(de::Error::custom)?;
                        entries.push(entry);
                    }
                    None => return Ok(entries),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(self.overflow));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(Visitor::<T, _, _, _, _, LIMIT> {
        expected,
        overflow,
        check,
        item: PhantomData,
    })
}
