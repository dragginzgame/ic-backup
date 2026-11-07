//! Real filesystem/custody checks around an integration-owned terminal validator.

use super::*;
use crate::tests::support::write_json_durable;
use crate::{model::artifacts::ArtifactChecksumRecord, tests::support::temp_dir};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Deserialize, Serialize)]
struct Terminal {
    complete: bool,
    authority: ArtifactChecksumRecord,
    commands: Vec<serde_json::Value>,
}

struct Validator;
impl RestoreTerminalEvidenceProvider for Validator {
    type Error = io::Error;
    fn qualify_terminal(
        &self,
        _: &Path,
        bytes: &[u8],
    ) -> Result<TerminalRestoreEvidence, Self::Error> {
        let terminal: Terminal = serde_json::from_slice(bytes)?;
        if !terminal.complete || !terminal.commands.is_empty() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(TerminalRestoreEvidence::for_in_process_restore(
            terminal.authority,
        ))
    }
}

struct Fixture {
    parent: PathBuf,
    journal: PathBuf,
    layout: BackupLayoutGuard,
    lock: JournalLock,
    terminal: Terminal,
}
impl Fixture {
    fn new() -> Self {
        let parent = temp_dir("guarded-reference-release");
        fs::create_dir_all(parent.join("backup")).unwrap();
        let parent = parent.canonicalize().unwrap();
        let journal = parent.join("restore.json");
        let layout = BackupLayoutGuard::acquire(&parent.join("backup")).unwrap();
        let terminal = Terminal {
            complete: true,
            authority: ArtifactChecksumRecord::from_bytes(b"original restore"),
            commands: vec![],
        };
        layout
            .retain_restore(&journal, terminal.authority.hash())
            .unwrap();
        write_json_durable(&journal, &terminal).unwrap();
        let lock = JournalLock::acquire(&journal).unwrap();
        Self {
            parent,
            journal,
            layout,
            lock,
            terminal,
        }
    }
    fn write(&self) {
        write_json_durable(&self.journal, &self.terminal).unwrap();
    }
    fn references(&self) -> Vec<u8> {
        fs::read(self.layout.root().join("restore-references.json")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.parent).unwrap();
    }
}

#[test]
fn exact_terminal_release_and_immediate_replay_preserve_journal_bytes() {
    let f = Fixture::new();
    let journal = fs::read(&f.journal).unwrap();
    assert_eq!(
        f.layout.release_restore(&f.lock, &Validator).unwrap(),
        RestoreReferenceReleaseOutcome::Released
    );
    assert!(!f.layout.has_restore_references().unwrap());
    let references = f.references();
    assert_eq!(
        f.layout.release_restore(&f.lock, &Validator).unwrap(),
        RestoreReferenceReleaseOutcome::AlreadyReleased
    );
    assert_eq!(f.references(), references);
    assert_eq!(fs::read(&f.journal).unwrap(), journal);
}

#[test]
fn incomplete_terminal_and_wrong_original_intent_preserve_reference_bytes() {
    let mut f = Fixture::new();
    let references = f.references();
    f.terminal.complete = false;
    f.write();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::Terminal(_))
    ));
    assert_eq!(f.references(), references);
    f.terminal.complete = true;
    f.terminal.authority = ArtifactChecksumRecord::from_bytes(b"other restore");
    f.write();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::Persistence(
            PersistenceError::RestoreReference(_)
        ))
    ));
    assert_eq!(f.references(), references);
}

#[test]
fn command_backed_originals_refuse_without_changing_references() {
    let mut f = Fixture::new();
    let references = f.references();
    f.terminal
        .commands
        .push(serde_json::json!({"operation_sequence": 7}));
    f.write();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::Terminal(_))
    ));
    assert_eq!(f.references(), references);
}

#[test]
fn retained_command_sidecars_refuse_even_with_empty_product_declaration() {
    let f = Fixture::new();
    let references = f.references();
    let sidecar = f.parent.join("restore.json.command-7.lock");
    fs::write(&sidecar, b"retained original custody").unwrap();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::RetiredCommandState)
    ));
    assert_eq!(fs::read(&sidecar).unwrap(), b"retained original custody");
    assert_eq!(f.references(), references);
    fs::remove_file(&sidecar).unwrap();
    std::os::unix::fs::symlink("missing", &sidecar).unwrap();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::RetiredCommandState)
    ));
    assert_eq!(f.references(), references);
}

#[test]
fn journal_or_command_evidence_change_during_qualification_preserves_references() {
    struct Changing(bool);
    impl RestoreTerminalEvidenceProvider for Changing {
        type Error = io::Error;
        fn qualify_terminal(
            &self,
            journal: &Path,
            bytes: &[u8],
        ) -> Result<TerminalRestoreEvidence, Self::Error> {
            let evidence = Validator.qualify_terminal(journal, bytes)?;
            if self.0 {
                fs::write(journal, b"changed after qualification")?;
            } else {
                let mut sidecar = journal.as_os_str().to_os_string();
                sidecar.push(".command-malformed");
                fs::write(PathBuf::from(sidecar), b"retained during qualification")?;
            }
            Ok(evidence)
        }
    }
    for change_journal in [true, false] {
        let f = Fixture::new();
        let references = f.references();
        let result = f.layout.release_restore(&f.lock, &Changing(change_journal));
        if change_journal {
            assert!(matches!(
                result,
                Err(RestoreReferenceReleaseError::JournalChanged)
            ));
        } else {
            assert!(matches!(
                result,
                Err(RestoreReferenceReleaseError::RetiredCommandState)
            ));
        }
        assert_eq!(f.references(), references);
    }
}
