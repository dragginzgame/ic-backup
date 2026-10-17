//! Real filesystem/custody checks around an integration-owned terminal validator.

use super::*;
use crate::test_support::write_json_durable;
use crate::{
    model::{artifacts::ArtifactChecksumRecord, command_custody::CommandCustodyRecord},
    ops::persistence::CommandLifetimeLock,
    test_support::temp_dir,
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    process::Command,
    thread,
    time::{Duration, Instant},
};

#[derive(Deserialize, Serialize)]
struct Terminal {
    complete: bool,
    authority: ArtifactChecksumRecord,
    commands: Vec<CommandCustodyRecord>,
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
        if !terminal.complete {
            return Err(io::ErrorKind::InvalidData.into());
        }
        Ok(TerminalRestoreEvidence::new(
            terminal.authority,
            terminal.commands,
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
fn live_command_custody_refuses_until_its_real_holder_exits() {
    let mut f = Fixture::new();
    let mut custody = CommandLifetimeLock::acquire(&f.journal, 7).unwrap();
    f.terminal.commands.push(custody.record().clone());
    f.write();
    let references = f.references();
    let mut command = Command::new("sleep");
    command.arg("30");
    let mut child = custody.spawn(command).unwrap();
    drop(custody);
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::Custody(
            CommandLifetimeLockError::InFlight { .. }
        ))
    ));
    assert_eq!(f.references(), references);
    child.kill().unwrap();
    child.wait().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match f.layout.release_restore(&f.lock, &Validator) {
            Ok(outcome) => {
                assert_eq!(outcome, RestoreReferenceReleaseOutcome::Released);
                break;
            }
            Err(RestoreReferenceReleaseError::Custody(CommandLifetimeLockError::InFlight {
                ..
            })) if Instant::now() < deadline => {
                assert_eq!(f.references(), references);
                thread::sleep(Duration::from_millis(5));
            }
            result => panic!("command custody did not settle after child exit: {result:?}"),
        }
    }
}

#[test]
fn missing_or_replaced_sidecar_cannot_create_fresh_quiescence() {
    let mut f = Fixture::new();
    let custody = CommandLifetimeLock::acquire(&f.journal, 7).unwrap();
    let path = custody.path().to_path_buf();
    f.terminal.commands.push(custody.record().clone());
    f.write();
    let original_inode = fs::File::open(&path).unwrap();
    drop(custody);
    let references = f.references();
    fs::remove_file(&path).unwrap();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::Custody(_))
    ));
    assert!(!path.exists());
    fs::write(&path, b"replacement").unwrap();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::Custody(_))
    ));
    assert_eq!(f.references(), references);
    drop(original_inode);
}

#[test]
fn cross_journal_and_duplicate_command_sets_refuse_without_release() {
    let mut f = Fixture::new();
    let references = f.references();
    let other = CommandLifetimeLock::acquire(&f.parent.join("other.json"), 7).unwrap();
    f.terminal.commands = vec![other.record().clone()];
    f.write();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::InvalidCommandSet)
    ));
    drop(other);
    let custody = CommandLifetimeLock::acquire(&f.journal, 7).unwrap();
    f.terminal.commands = vec![custody.record().clone(), custody.record().clone()];
    drop(custody);
    f.write();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Validator),
        Err(RestoreReferenceReleaseError::InvalidCommandSet)
    ));
    assert_eq!(f.references(), references);
}

#[test]
fn journal_change_during_product_qualification_preserves_references() {
    struct Changing;
    impl RestoreTerminalEvidenceProvider for Changing {
        type Error = io::Error;
        fn qualify_terminal(
            &self,
            journal: &Path,
            bytes: &[u8],
        ) -> Result<TerminalRestoreEvidence, Self::Error> {
            let evidence = Validator.qualify_terminal(journal, bytes)?;
            fs::write(journal, b"changed after qualification")?;
            Ok(evidence)
        }
    }
    let f = Fixture::new();
    let references = f.references();
    assert!(matches!(
        f.layout.release_restore(&f.lock, &Changing),
        Err(RestoreReferenceReleaseError::JournalChanged)
    ));
    assert_eq!(f.references(), references);
}
