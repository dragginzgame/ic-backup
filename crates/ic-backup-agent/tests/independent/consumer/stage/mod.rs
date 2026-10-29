//! Original fixture stage identities, declarations and retained paths.
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u64)]
pub(super) enum Stage {
    Capture = 0,
    Metadata = 1,
    Download = 2,
    Allocation = 3,
    Upload = 4,
    Load = 5,
    Start = 6,
}
impl Stage {
    pub const CAPTURE: [Self; 3] = [Self::Capture, Self::Metadata, Self::Download];
    pub const RESTORE: [Self; 4] = [Self::Allocation, Self::Upload, Self::Load, Self::Start];

    pub fn from_sequence(sequence: u64) -> io::Result<Self> {
        Self::CAPTURE
            .into_iter()
            .chain(Self::RESTORE)
            .find(|stage| stage.sequence() == sequence)
            .ok_or_else(|| io::ErrorKind::InvalidInput.into())
    }
    pub const fn sequence(self) -> u64 {
        self as u64
    }
    pub const fn mutations(self) -> u32 {
        match self {
            Self::Download | Self::Upload => 8,
            _ => 1,
        }
    }
    pub fn path(self, root: &Path) -> PathBuf {
        let workflow = match self {
            Self::Capture | Self::Metadata | Self::Download => "capture",
            Self::Allocation | Self::Upload | Self::Load | Self::Start => "restore",
        };
        root.join(workflow)
            .join(format!("execution-stage-{}", self.sequence()))
    }
}
