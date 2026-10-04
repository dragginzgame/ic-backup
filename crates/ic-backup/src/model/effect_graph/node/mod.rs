//! Passive exact operation identity and bounded explicit prerequisite admission.

use super::EffectGraphError;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;

/// Maximum direct prerequisites on one operation, independently of total graph bounds.
pub const MAX_EFFECT_DEPENDENCIES: usize = 1024;

/// Passive operation dependency declaration, without an execution request or authority.
#[derive(Clone, Debug)]
pub struct EffectNodeRequest {
    /// Exact opaque operation identity; zero and nonconsecutive u64 values are admitted.
    pub operation_sequence: u64,
    /// Exact operations required before this operation; duplicate declarations reject.
    pub depends_on: Vec<u64>,
}

/// Canonical immutable operation node with sorted unique explicit dependencies.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "NodeFields")]
pub struct EffectNodeRecord {
    operation_sequence: u64,
    depends_on: Vec<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NodeFields {
    operation_sequence: u64,
    #[serde(deserialize_with = "bounded_dependencies")]
    depends_on: Vec<u64>,
}
impl TryFrom<NodeFields> for EffectNodeRecord {
    type Error = EffectGraphError;
    fn try_from(fields: NodeFields) -> Result<Self, Self::Error> {
        Self::new(EffectNodeRequest {
            operation_sequence: fields.operation_sequence,
            depends_on: fields.depends_on,
        })
    }
}
impl EffectNodeRecord {
    /// Admit finite sorted explicit dependencies; graph admission owns closure/cycles.
    ///
    /// # Errors
    /// Rejects excessive direct dependencies and duplicate exact prerequisites.
    pub fn new(mut request: EffectNodeRequest) -> Result<Self, EffectGraphError> {
        if request.depends_on.len() > MAX_EFFECT_DEPENDENCIES {
            return Err(EffectGraphError::TooManyDependencies);
        }
        request.depends_on.sort_unstable();
        for pair in request.depends_on.windows(2) {
            if pair[0] == pair[1] {
                return Err(EffectGraphError::DuplicateDependency {
                    operation_sequence: request.operation_sequence,
                    dependency: pair[0],
                });
            }
        }
        Ok(Self {
            operation_sequence: request.operation_sequence,
            depends_on: request.depends_on,
        })
    }
    /// Read the exact opaque operation sequence identity.
    #[must_use]
    pub const fn operation_sequence(&self) -> u64 {
        self.operation_sequence
    }
    /// Read sorted exact explicit prerequisites.
    #[must_use]
    pub fn depends_on(&self) -> &[u64] {
        &self.depends_on
    }
}

fn bounded_dependencies<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u64>, D::Error> {
    struct DependenciesVisitor;
    impl<'de> de::Visitor<'de> for DependenciesVisitor {
        type Value = Vec<u64>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("bounded exact effect dependencies")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut dependencies = Vec::new();
            while dependencies.len() < MAX_EFFECT_DEPENDENCIES {
                match sequence.next_element()? {
                    Some(dependency) => dependencies.push(dependency),
                    None => return Ok(dependencies),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(EffectGraphError::TooManyDependencies));
            }
            Ok(dependencies)
        }
    }
    deserializer.deserialize_seq(DependenciesVisitor)
}
