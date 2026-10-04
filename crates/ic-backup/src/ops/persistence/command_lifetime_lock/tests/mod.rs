//! Fresh native descriptor-custody regressions; no IC executor is substituted.

use super::*;
use crate::model::command_custody::MAX_COMMAND_CUSTODY_RECORD_BYTES;
use crate::{
    ops::persistence::{create_json_durable, read_json},
    test_support::{
        hold_at_acknowledged_barrier, kill_child_at_acknowledged_barrier, temp_dir,
        wait_for_child_path, wait_for_path,
    },
};
use rustix::io::{FdFlags, fcntl_getfd};
use std::{
    os::unix::fs::{PermissionsExt, symlink},
    process::Stdio,
};

#[test]
fn one_spawn_inherits_custody_without_parent_or_command_descriptor_leaks() {
    let root = temp_dir("ic-backup-command-custody-spawn");
    fs::create_dir_all(&root).expect("create fixture");
    let mut lock =
        CommandLifetimeLock::acquire(&root.join("journal.json"), 7).expect("acquire custody");
    let record = lock.record().clone();
    create_json_durable(&root.join("custody.json"), &record).expect("persist before dispatch");
    assert_eq!(
        fs::metadata(lock.path())
            .expect("sidecar metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(
        fcntl_getfd(&lock.file)
            .expect("owner flags")
            .contains(FdFlags::CLOEXEC)
    );
    let mut command = Command::new("python3");
    command.args(["-c",r"import os,fcntl,json
fd=int(os.environ['IC_BACKUP_COMMAND_CUSTODY_FD'])
stat=os.fstat(fd)
print(json.dumps({'device':stat.st_dev,'inode':stat.st_ino,'cloexec':bool(fcntl.fcntl(fd,fcntl.F_GETFD)&fcntl.FD_CLOEXEC),'stdin_empty':os.read(0,1)==b''}))
"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let output = lock
        .spawn(command)
        .expect("one spawn")
        .wait_with_output()
        .expect("reap child");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let observed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("exact child evidence");
    assert_eq!(
        observed,
        serde_json::json!({"device":record.device(),"inode":record.inode(),"cloexec":false,"stdin_empty":true})
    );
    assert!(
        fcntl_getfd(&lock.file)
            .expect("unchanged owner flags")
            .contains(FdFlags::CLOEXEC)
    );
    assert!(matches!(
        lock.spawn(Command::new("unattempted-second-command")),
        Err(CommandLifetimeLockError::AlreadyDispatched { .. })
    ));
    let quiescent = lock.finish().expect("no command retains cloned custody");
    assert_eq!(quiescent.record(), &record);
    assert!(matches!(
        CommandLifetimeLock::acquire(record.journal(), 7),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    drop(quiescent);
    let lock = CommandLifetimeLock::acquire(record.journal(), 7).expect("after guard release");
    assert_eq!(lock.record(), &record);
    drop(lock);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn failed_spawn_consumes_the_allowance_without_leaking_custody() {
    let root = temp_dir("ic-backup-command-custody-failed-spawn");
    fs::create_dir_all(&root).expect("create fixture");
    let mut lock = CommandLifetimeLock::acquire(&root.join("journal.json"), 0).expect("acquire");
    assert!(
        matches!(lock.spawn(Command::new(root.join("absent-binary"))),Err(CommandLifetimeLockError::Io(error)) if error.kind()==io::ErrorKind::NotFound)
    );
    assert!(matches!(
        lock.spawn(Command::new("unattempted-retry")),
        Err(CommandLifetimeLockError::AlreadyDispatched { .. })
    ));
    let guard = lock
        .finish()
        .expect("failed exec retains no child descriptor");
    drop(guard);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn missing_replaced_or_unsafe_sidecars_do_not_prove_quiescence() {
    let root = temp_dir("ic-backup-command-custody-identity");
    fs::create_dir_all(&root).expect("create fixture");
    let mut lock = CommandLifetimeLock::acquire(&root.join("journal.json"), 3).expect("acquire");
    let record = lock.record().clone();
    let path = lock.path().to_path_buf();
    let retained = root.join("retained-original-sidecar");
    fs::rename(&path, &retained).expect("retain original identity");
    fs::write(&path, b"replacement").expect("different regular sidecar");
    assert!(
        matches!(lock.spawn(Command::new("unattempted-effect")),Err(CommandLifetimeLockError::IdentityChanged { path:actual }) if actual==path)
    );
    assert!(matches!(
        lock.finish(),
        Err(CommandLifetimeLockError::IdentityChanged { .. })
    ));
    assert!(matches!(
        CommandQuiescenceGuard::acquire(&record),
        Err(CommandLifetimeLockError::IdentityChanged { .. })
    ));
    assert_eq!(
        fs::read(&path).expect("preserve replacement evidence"),
        b"replacement"
    );
    fs::remove_file(&path).expect("successful test replacement cleanup");
    assert!(
        matches!(CommandQuiescenceGuard::acquire(&record),Err(CommandLifetimeLockError::Io(error)) if error.kind()==io::ErrorKind::NotFound)
    );
    assert!(!path.exists());
    symlink(&retained, &path).expect("unsafe sidecar alias");
    assert!(
        matches!(CommandQuiescenceGuard::acquire(&record),Err(CommandLifetimeLockError::UnsafeEntry { path:actual,kind }) if actual==path && kind=="Symlink")
    );
    fs::remove_file(&path).expect("successful test alias cleanup");
    fs::create_dir(&path).expect("unsafe sidecar directory");
    assert!(
        matches!(CommandQuiescenceGuard::acquire(&record),Err(CommandLifetimeLockError::UnsafeEntry { kind,.. }) if kind=="Directory")
    );
    assert!(retained.is_file());
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn parent_aliases_share_custody_and_unsafe_journal_leaves_reject() {
    let root = temp_dir("ic-backup-command-custody-location");
    fs::create_dir_all(root.join("real")).expect("create fixture");
    symlink(root.join("real"), root.join("alias")).expect("operator parent alias");
    let lock = CommandLifetimeLock::acquire(&root.join("alias/journal.json"), u64::MAX)
        .expect("resolve parent");
    assert_eq!(lock.record().journal(), root.join("real/journal.json"));
    assert!(matches!(
        CommandLifetimeLock::acquire(&root.join("real/journal.json"), u64::MAX),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    symlink(root.join("real/journal.json"), root.join("linked.json")).expect("unsafe leaf");
    assert!(matches!(
        CommandLifetimeLock::acquire(&root.join("linked.json"), 0),
        Err(CommandLifetimeLockError::InvalidJournal { .. })
    ));
    drop(lock);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

const DESCENDANT: &str = r"import os,sys,fcntl,time,pathlib
root=pathlib.Path(sys.argv[1]); fd=int(os.environ['IC_BACKUP_COMMAND_CUSTODY_FD'])
assert not fcntl.fcntl(fd,fcntl.F_GETFD)&fcntl.FD_CLOEXEC
parent=os.getppid(); deadline=time.monotonic()+10
(root/'descendant-ready').write_text('ready')
while os.getppid()==parent:
    assert time.monotonic()<deadline
    time.sleep(.005)
(root/'direct-exited').write_text('observed reparenting')
while not (root/'allow-descendant-exit').exists():
    assert time.monotonic()<deadline
    time.sleep(.005)
";

const DIRECT: &str = r"import os,sys,fcntl,time,pathlib,subprocess
root=pathlib.Path(sys.argv[1]); fd=int(os.environ['IC_BACKUP_COMMAND_CUSTODY_FD'])
assert not fcntl.fcntl(fd,fcntl.F_GETFD)&fcntl.FD_CLOEXEC
subprocess.Popen([sys.executable,'-c',os.environ['IC_BACKUP_TEST_DESCENDANT_SOURCE'],str(root)],close_fds=False)
deadline=time.monotonic()+10
while not (root/'descendant-ready').exists():
    assert time.monotonic()<deadline
    time.sleep(.005)
(root/'direct-ready').write_text('ready')
while not (root/'allow-direct-exit').exists():
    assert time.monotonic()<deadline
    time.sleep(.005)
os._exit(0)
";

#[test]
fn owner_death_keeps_custody_through_direct_child_and_descendant_exit() {
    const ROOT_ENV: &str = "IC_BACKUP_COMMAND_CUSTODY_CRASH_ROOT";
    if let Some(root) = std::env::var_os(ROOT_ENV) {
        let root = PathBuf::from(root);
        let mut lock =
            CommandLifetimeLock::acquire(&root.join("journal.json"), 7).expect("child acquire");
        create_json_durable(&root.join("custody.json"), lock.record())
            .expect("persist exact custody before dispatch");
        let mut command = Command::new("python3");
        command
            .args(["-c", DIRECT])
            .arg(&root)
            .env("IC_BACKUP_TEST_DESCENDANT_SOURCE", DESCENDANT)
            .stdin(Stdio::null());
        let mut child = lock.spawn(command).expect("direct child and descendant");
        wait_for_child_path(
            &mut child,
            &root.join("direct-ready"),
            "command tree readiness",
        );
        hold_at_acknowledged_barrier(&root);
    }
    let root = temp_dir("ic-backup-command-custody-owner-death");
    fs::create_dir_all(&root).expect("create fixture");
    let mut owner=Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact","ops::persistence::command_lifetime_lock::tests::owner_death_keeps_custody_through_direct_child_and_descendant_exit","--nocapture"])
        .env(ROOT_ENV,&root).spawn().expect("spawn owner");
    wait_for_child_path(
        &mut owner,
        &root.join("barrier-ready"),
        "armed command tree",
    );
    let record: CommandCustodyRecord =
        read_json(&root.join("custody.json"), MAX_COMMAND_CUSTODY_RECORD_BYTES)
            .expect("recover exact dispatch custody");
    assert!(matches!(
        CommandQuiescenceGuard::acquire(&record),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    kill_child_at_acknowledged_barrier(&mut owner, &root);
    assert!(matches!(
        CommandQuiescenceGuard::acquire(&record),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    fs::write(root.join("allow-direct-exit"), b"exit").expect("release direct child");
    wait_for_path(
        &root.join("direct-exited"),
        "descendant reparenting after direct exit",
    );
    assert!(matches!(
        CommandQuiescenceGuard::acquire(&record),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    fs::write(root.join("allow-descendant-exit"), b"exit").expect("release last descendant");
    let deadline = Instant::now() + Duration::from_secs(5);
    let guard = loop {
        match CommandQuiescenceGuard::acquire(&record) {
            Ok(guard) => break guard,
            Err(CommandLifetimeLockError::InFlight { .. }) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(5));
            }
            result => panic!("custody did not settle: {result:?}"),
        }
    };
    assert_eq!(guard.record(), &record);
    assert_eq!(
        read_json::<CommandCustodyRecord>(
            &root.join("custody.json"),
            MAX_COMMAND_CUSTODY_RECORD_BYTES
        )
        .expect("unchanged retained identity"),
        record
    );
    drop(guard);
    fs::remove_dir_all(root).expect("clean successful fixture");
}

#[test]
fn finish_is_bounded_while_a_dispatched_command_still_holds_custody() {
    const CHILD_ENV: &str = "IC_BACKUP_COMMAND_FINISH_CHILD_ROOT";
    if let Some(child_root) = std::env::var_os(CHILD_ENV) {
        hold_at_acknowledged_barrier(Path::new(&child_root));
    }
    let root = temp_dir("ic-backup-command-custody-finish");
    fs::create_dir_all(&root).expect("create fixture");
    let mut lock = CommandLifetimeLock::acquire(&root.join("journal.json"), 1).expect("acquire");
    let record = lock.record().clone();
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command.args(["--exact","ops::persistence::command_lifetime_lock::tests::finish_is_bounded_while_a_dispatched_command_still_holds_custody","--nocapture"]);
    // The child branch keeps the inherited descriptor while acknowledging its barrier.
    command.env(CHILD_ENV, &root);
    let mut child = lock.spawn(command).expect("spawn inherited holder");
    wait_for_child_path(&mut child, &root.join("barrier-ready"), "inherited holder");
    assert!(matches!(
        lock.finish(),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    assert!(matches!(
        CommandQuiescenceGuard::acquire(&record),
        Err(CommandLifetimeLockError::InFlight { .. })
    ));
    kill_child_at_acknowledged_barrier(&mut child, &root);
    let guard =
        CommandQuiescenceGuard::acquire(&record).expect("fresh quiescence after child exit");
    drop(guard);
    fs::remove_dir_all(root).expect("clean successful fixture");
}
