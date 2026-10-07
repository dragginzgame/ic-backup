//! Immutable original restore/source declarations; no current safety or fence authority.

use crate::model::{
    artifacts::{ArtifactChecksumRecord, ChecksumError, canonical_hash},
    operation_plan::OperationPlanRecord,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum raw input and canonical output bytes for the retained safety requirement.
pub const MAX_RESTORE_SAFETY_REQUIREMENT_BYTES: u64 = 1024;
/// Explicit original safety lane; neither declaration proves the application safe.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreSafetyLaneRecord {
    /// An application-qualified absence of irreversible external effects is required.
    NoIrreversibleEffects,
    /// An exact continuously retained fence outside rewindable state is required.
    ApplicationFenced,
}
/// Original integration-retained fence and authority revisions; never a release token.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreFenceBindingRecord {
    /// Exact original fence identity.
    pub identity: ArtifactChecksumRecord,
    /// Original membership authority revision bound to that fence.
    pub membership_revision: ArtifactChecksumRecord,
    /// Original external-obligation authority revision, retained outside the snapshot.
    pub external_obligations_revision: ArtifactChecksumRecord,
}
/// Passive original source and application safety declaration; no neutral default.
#[derive(Clone, Debug)]
pub struct RestoreSafetyRequirementRequest {
    /// Exact integration-qualified complete source artifact/manifest digest.
    pub source_artifacts: ArtifactChecksumRecord,
    /// Explicit required application safety lane.
    pub safety: RestoreSafetyLaneRecord,
    /// Required only for the fenced lane; recovered from the integration's durable owner.
    pub expected_fence: Option<RestoreFenceBindingRecord>,
}
/// Strict v1 immutable restore intent, original source and safety declaration.
///
/// Source plan and artifacts must be qualified and kept in stable custody by the
/// integration. This record authenticates no backup, creates no fence, and grants
/// no load/start, paid call, artifact completeness or reference-release authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "RequirementFields")]
pub struct RestoreSafetyRequirementRecord {
    version: u16,
    plan_intent: String,
    source_plan_intent: String,
    source_artifacts: ArtifactChecksumRecord,
    safety: RestoreSafetyLaneRecord,
    expected_fence: Option<RestoreFenceBindingRecord>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequirementFields {
    version: u16,
    plan_intent: String,
    source_plan_intent: String,
    source_artifacts: ArtifactChecksumRecord,
    safety: RestoreSafetyLaneRecord,
    #[serde(deserialize_with = "required_fence")]
    expected_fence: Option<RestoreFenceBindingRecord>,
}
fn required_fence<'de, D: serde::Deserializer<'de>>(
    decoder: D,
) -> Result<Option<RestoreFenceBindingRecord>, D::Error> {
    Option::deserialize(decoder)
}
impl TryFrom<RequirementFields> for RestoreSafetyRequirementRecord {
    type Error = RestoreSafetyRequirementError;
    fn try_from(fields: RequirementFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(RestoreSafetyRequirementError::UnsupportedVersion(
                fields.version,
            ));
        }
        validate_lane(fields.safety, fields.expected_fence.as_ref())?;
        Ok(Self {
            version: 1,
            plan_intent: canonical_hash(&fields.plan_intent)?,
            source_plan_intent: canonical_hash(&fields.source_plan_intent)?,
            source_artifacts: fields.source_artifacts,
            safety: fields.safety,
            expected_fence: fields.expected_fence,
        })
    }
}
fn validate_lane(
    safety: RestoreSafetyLaneRecord,
    fence: Option<&RestoreFenceBindingRecord>,
) -> Result<(), RestoreSafetyRequirementError> {
    match (safety, fence) {
        (RestoreSafetyLaneRecord::ApplicationFenced, None) => {
            Err(RestoreSafetyRequirementError::FenceRequired)
        }
        (RestoreSafetyLaneRecord::NoIrreversibleEffects, Some(_)) => {
            Err(RestoreSafetyRequirementError::UnexpectedFence)
        }
        _ => Ok(()),
    }
}
fn validate_source(
    plan: &OperationPlanRecord,
    source: &OperationPlanRecord,
) -> Result<(), RestoreSafetyRequirementError> {
    if plan.context().network() != source.context().network() {
        return Err(RestoreSafetyRequirementError::SourceNetworkMismatch);
    }
    if plan.context().release() != source.context().release() {
        return Err(RestoreSafetyRequirementError::SourceReleaseMismatch);
    }
    if plan
        .selected_targets()
        .iter()
        .any(|target| source.selected_targets().binary_search(target).is_err())
    {
        return Err(RestoreSafetyRequirementError::SourceSelectionMismatch);
    }
    Ok(())
}
impl RestoreSafetyRequirementRecord {
    /// Bind original restore and source plans, same network/release/IDs and explicit safety lane.
    ///
    /// A restore selection may be a subset of the source selection. Application
    /// safety for that exact subset remains qualified by the provider. Source caller
    /// equality is not required; current caller permissions are a separate boundary.
    /// # Errors
    /// Rejects source network/release/selection mismatch or an inappropriate fence.
    pub fn new(
        plan: &OperationPlanRecord,
        source: &OperationPlanRecord,
        input: RestoreSafetyRequirementRequest,
    ) -> Result<Self, RestoreSafetyRequirementError> {
        validate_source(plan, source)?;
        validate_lane(input.safety, input.expected_fence.as_ref())?;
        Ok(Self {
            version: 1,
            plan_intent: plan.digest().hash().into(),
            source_plan_intent: source.digest().hash().into(),
            source_artifacts: input.source_artifacts,
            safety: input.safety,
            expected_fence: input.expected_fence,
        })
    }
    /// Read full original restore intent, including requests and attempt allowances.
    #[must_use]
    pub fn plan_intent(&self) -> &str {
        &self.plan_intent
    }
    /// Read full original source plan identity; not source completeness or authenticity.
    #[must_use]
    pub fn source_plan_intent(&self) -> &str {
        &self.source_plan_intent
    }
    /// Read exact original source artifact binding supplied by its qualified owner.
    #[must_use]
    pub const fn source_artifacts(&self) -> &ArtifactChecksumRecord {
        &self.source_artifacts
    }
    /// Read the original explicit safety lane.
    #[must_use]
    pub const fn safety(&self) -> RestoreSafetyLaneRecord {
        self.safety
    }
    /// Read original retained fence/revisions; creates no current custody or release capability.
    #[must_use]
    pub const fn expected_fence(&self) -> Option<&RestoreFenceBindingRecord> {
        self.expected_fence.as_ref()
    }
    /// Validate both exact original plans and same-network/release/ID source admission.
    /// # Errors
    /// Rejects changed original intent/source or source context/selection mismatch.
    pub fn validate_plans(
        &self,
        plan: &OperationPlanRecord,
        source: &OperationPlanRecord,
    ) -> Result<(), RestoreSafetyRequirementError> {
        if self.plan_intent != plan.digest().hash() {
            return Err(RestoreSafetyRequirementError::PlanMismatch);
        }
        if self.source_plan_intent != source.digest().hash() {
            return Err(RestoreSafetyRequirementError::SourcePlanMismatch);
        }
        validate_source(plan, source)
    }
    /// Hash NUL-terminated v1 ASCII domain, three 64-byte ASCII digests and safety tag.
    ///
    /// Tags: no irreversible effects=0, application fenced=1. The fenced lane
    /// appends 64 ASCII bytes each of fence identity, membership revision and
    /// external-obligations revision. Version is bound through the domain.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/restore-safety-requirement/v1\0".to_vec();
        bytes.extend_from_slice(self.plan_intent.as_bytes());
        bytes.extend_from_slice(self.source_plan_intent.as_bytes());
        bytes.extend_from_slice(self.source_artifacts.hash().as_bytes());
        bytes.push(match self.safety {
            RestoreSafetyLaneRecord::NoIrreversibleEffects => 0,
            RestoreSafetyLaneRecord::ApplicationFenced => 1,
        });
        if let Some(fence) = &self.expected_fence {
            bytes.extend_from_slice(fence.identity.hash().as_bytes());
            bytes.extend_from_slice(fence.membership_revision.hash().as_bytes());
            bytes.extend_from_slice(fence.external_obligations_revision.hash().as_bytes());
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
/// Typed original source/safety declaration denial; retains all existing obligations.
#[derive(Debug, Error)]
pub enum RestoreSafetyRequirementError {
    /// Only v1 is maintained.
    #[error("unsupported restore safety requirement version {0}")]
    UnsupportedVersion(u16),
    /// Full original restore intent differs.
    #[error("restore safety original plan mismatch")]
    PlanMismatch,
    /// Full original source plan differs.
    #[error("restore safety original source plan mismatch")]
    SourcePlanMismatch,
    /// Cross-network recovery is outside this product scope.
    #[error("restore source network mismatch")]
    SourceNetworkMismatch,
    /// Cross-release recovery is outside this product scope.
    #[error("restore source release mismatch")]
    SourceReleaseMismatch,
    /// Every selected exact target must also have been selected in the source.
    #[error("restore source selection mismatch")]
    SourceSelectionMismatch,
    /// The application-fenced lane must retain original fence/revisions.
    #[error("restore safety requires original fence binding")]
    FenceRequired,
    /// The no-irreversible-effects lane cannot silently add a fence obligation.
    #[error("unexpected restore safety fence binding")]
    UnexpectedFence,
    /// Intent hash admission failed.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
}
