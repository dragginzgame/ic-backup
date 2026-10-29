//! Sequential inspection of original workflow journals, without execution authority.

use crate::{
    model::artifacts::ArtifactChecksumRecord, policy::execution_progress::ExecutionProgressView,
};
use serde::Serialize;

/// Local projection of every declared stage, including unresolved original attempts.
///
/// No absent stage implies unused authority. Applied child journals alone do not
/// establish a checkpoint, artifact completeness, product terminal or safe release.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExecutionWorkflowProgressView {
    /// Exact original workflow identity.
    pub workflow: ArtifactChecksumRecord,
    /// One row per original stage in dependency planning order.
    pub stages: Vec<ExecutionStageProgressView>,
}

/// One allocated stage and its retained local condition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExecutionStageProgressView {
    /// Original opaque stage identity, never an array index.
    pub stage_sequence: u64,
    /// Absent or admitted complete child journals; no state is dispatch permission.
    pub state: ExecutionStageProgressState,
}

/// Original local stage evidence, with no invented missing journals or receipts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ExecutionStageProgressState {
    /// The original derived stage path is absent. This proves no never-dispatched custody.
    Unprepared,
    /// Exact child plan, binding, ancestors and complete original journals were admitted.
    Prepared {
        /// Observed retained binding, not independently qualified learned evidence.
        binding: ArtifactChecksumRecord,
        /// Current original accounting, including both pending reservation lanes.
        progress: Box<ExecutionProgressView>,
    },
}
