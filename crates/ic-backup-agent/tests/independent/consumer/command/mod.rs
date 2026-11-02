//! Closed fixture command syntax; parsed once without changing product authority.
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
    LoseLoad,
    Replay,
    Restore,
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
    LoseLoad,
    Replay,
    Restore,
    Start,
}

impl Mode {
    pub(super) fn action(self) -> Action {
        match self {
            Self::ApplicationDie
            | Self::ApplicationInit
            | Self::ApplicationRefuseReply
            | Self::ApplicationSmall => Action::ApplicationNew,
            Self::ApplicationObserve => Action::ApplicationObserve,
            Self::ApplicationReplay | Self::ApplicationResume => Action::ApplicationRetained,
            Self::Capture
            | Self::CaptureFailIngress
            | Self::CaptureFailPartial
            | Self::CaptureFailRetention => Action::CaptureNew,
            Self::CaptureContinue => Action::CaptureContinue,
            Self::InspectCapture => Action::InspectCapture,
            Self::InspectRestore => Action::InspectRestore,
            Self::LoseLoad => Action::LoseLoad,
            Self::Replay => Action::Replay,
            Self::Restore => Action::Restore,
            Self::Start => Action::Start,
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
