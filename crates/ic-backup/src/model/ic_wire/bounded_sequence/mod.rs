//! Bounded Serde sequence admission for IC wire values.

use serde::{Deserialize, Deserializer, de};
use std::{fmt, marker::PhantomData};

/// Reject an excessive size hint before allocation and probe one typed excess
/// value after the bound. Never reserve from a wire-provided size hint.
pub(crate) fn deserialize<'de, D, T, const LIMIT: usize>(
    deserializer: D,
    expecting: fn(&mut fmt::Formatter<'_>, usize) -> fmt::Result,
    overflow: &'static str,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct BoundedVisitor<T, const LIMIT: usize> {
        expecting: fn(&mut fmt::Formatter<'_>, usize) -> fmt::Result,
        overflow: &'static str,
        marker: PhantomData<T>,
    }

    impl<'de, T: Deserialize<'de>, const LIMIT: usize> de::Visitor<'de> for BoundedVisitor<T, LIMIT> {
        type Value = Vec<T>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            (self.expecting)(f, LIMIT)
        }

        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            if sequence.size_hint().is_some_and(|size| size > LIMIT) {
                return Err(de::Error::custom(self.overflow));
            }
            let mut values = Vec::new();
            while values.len() < LIMIT {
                match sequence.next_element()? {
                    Some(value) => values.push(value),
                    None => return Ok(values),
                }
            }
            if sequence.next_element::<T>()?.is_some() {
                return Err(de::Error::custom(self.overflow));
            }
            Ok(values)
        }
    }

    deserializer.deserialize_seq(BoundedVisitor::<T, LIMIT> {
        expecting,
        overflow,
        marker: PhantomData,
    })
}
