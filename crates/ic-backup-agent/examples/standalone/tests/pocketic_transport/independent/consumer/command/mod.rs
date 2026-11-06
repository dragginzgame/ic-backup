//! Closed fixture command syntax; parsed once without changing product authority.
use super::Stage;
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
pub(super) struct Arguments {
    #[arg(value_enum)]
    pub mode: Mode,
    #[arg(allow_hyphen_values = true)]
    pub root: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum Mode {
    ApplicationDie,
    ApplicationInit,
    ApplicationObserve,
    ApplicationRefuseReply,
    ApplicationReplay,
    ApplicationResume,
    ApplicationSmall,
    Capture,
    CaptureContinue,
    CaptureFailIngress,
    CaptureFailPartial,
    CaptureFailRetention,
    InspectCapture,
    InspectRestore,
    LoseAllocation,
    LoseLoad,
    LoseStart,
    LoseUploadFirst,
    LoseUploadSecond,
    Replay,
    Restore,
    RestoreStep,
    Start,
}
// Derived execution actions; fault-specific modes retain their original names and evidence.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Action {
    ApplicationNew,
    ApplicationObserve,
    ApplicationRetained,
    CaptureNew,
    CaptureContinue,
    InspectCapture,
    InspectRestore,
    LoseReply,
    Replay,
    Restore,
    RestoreStep,
    Start,
}

// Resolved once at argv admission; callbacks receive faults without reinterpreting modes.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Fault {
    None,
    CaptureIngress,
    CaptureRetention,
    CapturePartial,
    LostReply(Stage, u64),
    ApplicationReply,
    ApplicationExit,
}

impl Mode {
    // The parent harness projects the same parsed fault without another mode catalog.
    #[cfg(test)]
    pub(crate) fn lost_reply(self) -> Option<(Stage, u64)> {
        match self.configuration().1 {
            Fault::LostReply(stage, sequence) => Some((stage, sequence)),
            _ => None,
        }
    }

    pub(super) fn configuration(self) -> (Action, Fault) {
        use Action as A;
        use Fault as F;
        match self {
            Self::ApplicationInit | Self::ApplicationSmall => (A::ApplicationNew, F::None),
            Self::ApplicationDie => (A::ApplicationNew, F::ApplicationExit),
            Self::ApplicationRefuseReply => (A::ApplicationNew, F::ApplicationReply),
            Self::ApplicationObserve => (A::ApplicationObserve, F::None),
            Self::ApplicationReplay | Self::ApplicationResume => (A::ApplicationRetained, F::None),
            Self::Capture => (A::CaptureNew, F::None),
            Self::CaptureFailIngress => (A::CaptureNew, F::CaptureIngress),
            Self::CaptureFailPartial => (A::CaptureNew, F::CapturePartial),
            Self::CaptureFailRetention => (A::CaptureNew, F::CaptureRetention),
            Self::CaptureContinue => (A::CaptureContinue, F::None),
            Self::InspectCapture => (A::InspectCapture, F::None),
            Self::InspectRestore => (A::InspectRestore, F::None),
            Self::LoseAllocation => (
                A::LoseReply,
                F::LostReply(Stage::Allocation, Stage::Allocation.sequence()),
            ),
            Self::LoseLoad => (
                A::LoseReply,
                F::LostReply(Stage::Load, Stage::Load.sequence()),
            ),
            Self::LoseStart => (
                A::LoseReply,
                F::LostReply(Stage::Start, Stage::Start.sequence()),
            ),
            Self::LoseUploadFirst => (A::LoseReply, F::LostReply(Stage::Upload, 0)),
            Self::LoseUploadSecond => (A::LoseReply, F::LostReply(Stage::Upload, 1)),
            Self::Replay => (A::Replay, F::None),
            Self::Restore => (A::Restore, F::None),
            Self::RestoreStep => (A::RestoreStep, F::None),
            Self::Start => (A::Start, F::None),
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.to_possible_value()
                .expect("every fixture mode is a registered Clap value")
                .get_name(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    #[test]
    fn original_command_preserves_native_root_bytes_and_rejects_unknown_argv() {
        let root = OsString::from_vec(b"/tmp/native-\xff".to_vec());
        let parsed = Arguments::try_parse_from([
            OsString::from("consumer"),
            OsString::from("application-refuse-reply"),
            root.clone(),
        ])
        .unwrap();
        assert_eq!(parsed.mode, Mode::ApplicationRefuseReply);
        assert_eq!(parsed.root, PathBuf::from(root));
        let parsed = Arguments::try_parse_from(["consumer", "capture", "-retained"]).unwrap();
        assert_eq!(parsed.root, PathBuf::from("-retained"));
        for args in [
            vec!["consumer"],
            vec!["consumer", "application-unknown", "/tmp"],
            vec!["consumer", "capture", "/tmp", "extra"],
            vec!["consumer", "--help"],
        ] {
            assert!(Arguments::try_parse_from(args).is_err());
        }
    }
}
