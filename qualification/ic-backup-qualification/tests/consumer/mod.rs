//! Build and run the isolated fixture consumer through its normal dependency graph.

use ic_backup::store::BackupDigest;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub struct Consumer {
    binary: PathBuf,
    evidence: PathBuf,
    endpoint: String,
    network: BackupDigest,
}
impl Consumer {
    pub fn build(workspace: &Path, evidence: &Path, endpoint: &str, network: BackupDigest) -> Self {
        let mut tree = Command::new("cargo");
        tree.args([
            "tree",
            "--locked",
            "--edges",
            "normal",
            "-p",
            "ic-backup-consumer",
            "--manifest-path",
        ])
        .arg(workspace.join("Cargo.toml"));
        command(tree, evidence, "consumer-graph");
        let graph = fs::read_to_string(evidence.join("consumer-graph.stdout")).unwrap();
        assert!(graph.contains("ic-backup v"));
        assert!(
            !graph.contains("ic-testkit")
                && !graph.contains("pocket-ic")
                && !graph.contains("ic-backup-qualification")
        );
        let mut build = Command::new("cargo");
        build
            .args([
                "build",
                "--locked",
                "-p",
                "ic-backup-consumer",
                "--manifest-path",
            ])
            .arg(workspace.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", workspace.join("target"));
        command(build, evidence, "consumer-build");

        Self {
            binary: workspace.join("target/debug/ic-backup-consumer"),
            evidence: evidence.to_owned(),
            endpoint: endpoint.into(),
            network,
        }
    }

    pub fn call(&self, mode: &str, id: &str, approval: BackupDigest, live: bool) {
        let mut process = Command::new(&self.binary);
        process
            .arg(mode)
            .arg(self.evidence.join("store"))
            .arg(id)
            .arg(approval.to_string());
        if live {
            process
                .arg(&self.endpoint)
                .arg(self.evidence.join("root.der"))
                .arg(self.network.to_string());
        }
        command(process, &self.evidence, mode);
    }
}

fn command(mut command: Command, evidence: &Path, label: &str) {
    struct Held(std::process::Child);
    impl Drop for Held {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    command.stdout(Stdio::from(
        fs::File::create(evidence.join(format!("{label}.stdout"))).unwrap(),
    ));
    command.stderr(Stdio::from(
        fs::File::create(evidence.join(format!("{label}.stderr"))).unwrap(),
    ));
    let mut child = Held(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(180);

    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(
                status.success(),
                "{label}: {status}; evidence {}",
                evidence.display()
            );
            break;
        }
        assert!(Instant::now() < deadline, "{label} timed out");
        std::thread::sleep(Duration::from_millis(10));
    }
}
