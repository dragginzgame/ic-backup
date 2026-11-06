//! Fixed original stage identities and finite example transfer ceilings.
use std::{
    io,
    path::{Path, PathBuf},
};

/// Select an original catalog; this does not create a new execution mode or journal.
#[derive(Clone, Copy)]
pub enum Phase {
    /// Capture, metadata and complete local download.
    Capture,
    /// Source-bound allocation/upload, same-ID load and accepted start.
    Restore,
}

impl Phase {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Capture => "capture",
            Self::Restore => "restore",
        }
    }
    pub(super) const fn stages(self) -> &'static [Stage] {
        match self {
            Self::Capture => &Stage::CAPTURE,
            Self::Restore => &Stage::RESTORE,
        }
    }
}

/// Original catalog identities shared by the example and qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum Stage {
    /// Capture one original snapshot.
    Capture = 0,
    /// Read its original metadata.
    Metadata = 1,
    /// Download all admitted source extents.
    Download = 2,
    /// Allocate one source-bound upload destination.
    Allocation = 3,
    /// Upload every original data extent.
    Upload = 4,
    /// Load the completed upload into the same existing canister.
    Load = 5,
    /// Start only after independent restored acceptance.
    Start = 6,
}
impl Stage {
    /// Original capture catalog order.
    pub const CAPTURE: [Self; 3] = [Self::Capture, Self::Metadata, Self::Download];
    /// Original restore catalog order.
    pub const RESTORE: [Self; 4] = [Self::Allocation, Self::Upload, Self::Load, Self::Start];

    /// Resolve an exact stage identity.
    /// # Errors
    /// Rejects identities outside this catalog.
    pub fn from_sequence(sequence: u64) -> io::Result<Self> {
        [Phase::Capture, Phase::Restore]
            .into_iter()
            .flat_map(Phase::stages)
            .copied()
            .find(|stage| stage.sequence() == sequence)
            .ok_or_else(|| io::ErrorKind::InvalidInput.into())
    }
    /// Read the original opaque sequence.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self as u64
    }
    /// Read the fixed original mutation ceiling; unused headroom grants no retry.
    #[must_use]
    pub const fn mutations(self) -> u32 {
        match self {
            Self::Download | Self::Upload => 8,
            _ => 1,
        }
    }
    /// Resolve the retained stage directory below a caller-owned root.
    #[must_use]
    pub fn path(self, root: &Path) -> PathBuf {
        let phase = if Phase::Capture.stages().contains(&self) {
            Phase::Capture
        } else {
            Phase::Restore
        };
        root.join(phase.name())
            .join(format!("execution-stage-{}", self.sequence()))
    }
}

// Decode original evidence once; callers still admit plans and predecessor commitments.
pub(super) fn read_binding(
    root: &Path,
    sequence: u64,
) -> io::Result<ic_backup::model::execution_workflow::ExecutionStageBindingRecord> {
    let bytes = ic_host_fs::read::read_file_no_follow(
        &root.join("stage-binding.json"),
        usize::try_from(ic_backup::model::execution_workflow::MAX_EXECUTION_STAGE_BYTES)
            .map_err(super::error)?,
    )
    .map_err(super::error)?;
    let binding: ic_backup::model::execution_workflow::ExecutionStageBindingRecord =
        serde_json::from_slice(&bytes)?;
    if binding.stage_sequence() != sequence {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(binding)
}

#[cfg(test)]
mod tests;
