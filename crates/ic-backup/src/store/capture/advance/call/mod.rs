//! Finite capture calls; no arbitrary management dispatch.
use super::{
    CaptureError, CapturePlan, PreparedStatus, StatusAdmissionRecord, StatusEvidence,
    StatusProvider, arguments,
};
use serde::{Deserialize, Serialize};
use std::future::Future;

/// The management calls currently implemented by the capture workflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CaptureCall {
    /// Check the original stopped/module/controller preconditions.
    Status,
    /// Observe the original snapshot inventory before allocating.
    ListSnapshots,
    /// Take one new snapshot, without replacement or uninstalling code.
    TakeSnapshot,
}
impl CaptureCall {
    /// Exact external method name. This projection grants no dispatch authority.
    #[must_use]
    pub const fn method_name(self) -> &'static str {
        match self {
            Self::Status => "canister_status",
            Self::ListSnapshots => "list_canister_snapshots",
            Self::TakeSnapshot => "take_canister_snapshot",
        }
    }
    pub(in crate::store::capture) fn arguments(
        self,
        plan: &CapturePlan,
    ) -> Result<Vec<u8>, CaptureError> {
        match self {
            Self::Status | Self::ListSnapshots => arguments(plan),
            Self::TakeSnapshot => {
                // Only build canonical bytes here. The workflow independently admits
                // its authenticated inventory and original capacity before reservation.
                let args = plan
                    .take_snapshot_arguments(&[], 1)
                    .map_err(|_| CaptureError::StatusAdmission)?;
                ic_platform::candid_codec::encode_one(&args, 512)
                    .map_err(|_| CaptureError::StatusAdmission)
            }
        }
    }
}

/// Trusted capture provider extending the original status boundary.
///
/// The same exact-byte, bounded-effect and historical-proof obligations as
/// [`StatusProvider`] apply to each call. The workflow supplies only the canonical
/// finite method/arguments and reserves each signing and submission separately.
/// Admission/authentication remain local and effect-free. No method may retry,
/// poll, replace a snapshot, uninstall code or infer a take result from list differences.
pub trait CaptureProvider: StatusProvider {
    /// Admit live original authority and optional exact retained request.
    fn admit_call(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error>;
    /// Prepare one exact call under its already-durable signing reservation.
    fn prepare_call(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
    ) -> impl Future<Output = Result<PreparedStatus, Self::Error>> + Send;
    /// Submit the retained bytes once under their already-durable reservation.
    fn submit_call(
        &mut self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: &PreparedStatus,
    ) -> impl Future<Output = Result<StatusEvidence, Self::Error>> + Send;
    /// Authenticate original request/result association locally, including cold replay.
    fn authenticate_call(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: &PreparedStatus,
        evidence: &StatusEvidence,
    ) -> Result<(), Self::Error>;
}
