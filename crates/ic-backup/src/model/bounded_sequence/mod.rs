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
    deserialize_with::<D, T, _, _, _, _, LIMIT, false>(deserializer, expected, overflow, |_| {
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
    deserialize_with::<D, T, _, _, C, F, LIMIT, false>(deserializer, expected, overflow, check)
}

/// IC wire values reject excessive hints and decode the one excess value by type.
pub(crate) fn deserialize_wire<'de, D, T, const LIMIT: usize>(
    deserializer: D,
    expected: impl fmt::Display,
    overflow: impl fmt::Display,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserialize_with::<D, T, _, _, _, _, LIMIT, true>(deserializer, expected, overflow, |_| {
        Ok::<(), std::convert::Infallible>(())
    })
}

fn deserialize_with<'de, D, T, X, E, C, F, const LIMIT: usize, const WIRE: bool>(
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
    struct Visitor<T, X, E, C, F, const LIMIT: usize, const WIRE: bool> {
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
        const WIRE: bool,
    > de::Visitor<'de> for Visitor<T, X, E, C, F, LIMIT, WIRE>
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}", self.expected)
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            mut self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            if WIRE && sequence.size_hint().is_some_and(|size| size > LIMIT) {
                return Err(de::Error::custom(self.overflow));
            }
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
            let excess = if WIRE {
                sequence.next_element::<T>()?.is_some()
            } else {
                sequence.next_element::<de::IgnoredAny>()?.is_some()
            };
            if excess {
                return Err(de::Error::custom(self.overflow));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(Visitor::<T, _, _, _, _, LIMIT, WIRE> {
        expected,
        overflow,
        check,
        item: PhantomData,
    })
}
