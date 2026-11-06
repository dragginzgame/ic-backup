//! Build the canonical consumer with normal dependencies and an isolated lock/target.

use std::fmt::Write as _;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

pub(super) fn binary() -> &'static PathBuf {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(build)
}

fn build() -> PathBuf {
    let root = workspace();
    let evidence = super::super::support::root("independent-build");
    let selected = evidence.join("consumer");
    fs::create_dir_all(&selected).unwrap();
    let registry = registry_selected();
    let original = fs::read(root.join("Cargo.lock")).unwrap();
    snapshot(&root, &evidence);
    write_manifest(&root, &evidence, &selected, registry);
    prepare_graph(&root, &evidence, &selected, registry);
    compile(&selected, &evidence);
    assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), original);
    eprintln!(
        "Independent {} normal-consumer graph/build evidence: {}",
        if registry {
            "registry-only"
        } else {
            "candidate"
        },
        evidence.display()
    );
    evidence.join("target/debug/ic-backup-journey-consumer")
}

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}

// Explicit online setup only; normal candidate/registry qualification stays offline.
#[test]
#[ignore = "explicit registry cache preparation; requires published workspace versions and network"]
fn prepare_registry_consumer() {
    let root = workspace();
    let evidence = super::super::support::root("registry-prepare");
    let selected = evidence.join("consumer");
    fs::create_dir_all(&selected).unwrap();
    let original = fs::read(root.join("Cargo.lock")).unwrap();
    snapshot(&root, &evidence);
    write_manifest(&root, &evidence, &selected, true);
    fs::copy(evidence.join("original.lock"), selected.join("Cargo.lock")).unwrap();
    run_logged(
        Command::new("cargo")
            .arg(format!("+{}", env!("CARGO_PKG_RUST_VERSION")))
            .args(["fetch", "--manifest-path"])
            .arg(selected.join("Cargo.toml"))
            .env("RUSTUP_AUTO_INSTALL", "0"),
        &evidence.join("fetch.log"),
    );
    prepare_graph(&root, &evidence, &selected, true);
    assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), original);
    eprintln!(
        "Registry cache preparation evidence: {}",
        evidence.display()
    );
}

fn registry_selected() -> bool {
    match std::env::var("IC_BACKUP_CONSUMER_REGISTRY") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(value) if value == "1" => true,
        _ => panic!("IC_BACKUP_CONSUMER_REGISTRY must be absent or exactly 1"),
    }
}

fn snapshot(root: &Path, evidence: &Path) {
    run_logged(
        Command::new("bash").arg("-c")
            .arg("set -e; source \"$1/scripts/ci/consumer-fixture.sh\"; consumer_fixture_snapshot \"$1\" \"$2\" ''")
            .arg("consumer-snapshot").arg(root).arg(evidence),
        &evidence.join("snapshot.log"),
    );
}

fn write_manifest(root: &Path, evidence: &Path, selected: &Path, registry: bool) {
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(evidence.join("original.json")).unwrap()).unwrap();
    let agent = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "ic-backup-agent")
        .unwrap();
    let requirement = |name: &str| {
        agent["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["name"] == name)
            .unwrap()["req"]
            .as_str()
            .unwrap()
    };
    let mut manifest = format!(
        "[workspace]\n[package]\nname = \"ic-backup-journey-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = {}\n[dependencies]\n",
        agent["rust_version"]
    );
    for name in ["ic-backup", "ic-backup-agent"] {
        if registry {
            writeln!(
                manifest,
                r#"{name} = {{ version = "={version}", default-features = false }}"#,
                version = env!("CARGO_PKG_VERSION")
            )
            .unwrap();
        } else {
            writeln!(
                manifest,
                "{name} = {{ path = {} }}",
                serde_json::to_string(&root.join("crates").join(name)).unwrap()
            )
            .unwrap();
        }
    }
    write!(manifest,
        "ic-platform = {{ version = {:?}, default-features = false, features = [\"agent\", \"candid\", \"management-canister-types\"] }}\nserde = {{ version = {:?}, features = [\"derive\"] }}\nserde_json = {:?}\ntokio = {{ version = {:?}, features = [\"macros\", \"rt-multi-thread\"] }}\n",
        requirement("ic-platform"), requirement("serde"), requirement("serde_json"), requirement("tokio")
    ).unwrap();
    writeln!(
        manifest,
        "ic-host-fs = {{ version = {:?}, default-features = false }}",
        requirement("ic-host-fs")
    )
    .unwrap();
    writeln!(
        manifest,
        "clap = {{ version = {:?}, default-features = false, features = [\"derive\", \"std\"] }}",
        requirement("clap")
    )
    .unwrap();
    let example = root.join("crates/ic-backup-agent/examples/standalone");
    write!(
        manifest,
        "\n[package.metadata.shared-tooling.loc]\ncategory = \"tests\"\n\n[lib]\nname = \"standalone\"\npath = {}\n\n[[bin]]\nname = \"ic-backup-journey-consumer\"\npath = {}\n",
        serde_json::to_string(&example.join("lib.rs")).unwrap(),
        serde_json::to_string(&example.join("tests/pocketic_transport/independent/consumer/mod.rs")).unwrap(),
    )
    .unwrap();
    fs::write(selected.join("Cargo.toml"), manifest).unwrap();
}

fn prepare_graph(root: &Path, evidence: &Path, selected: &Path, registry: bool) {
    eprintln!(
        "Consumer graph preparation details: {}",
        selected.join("preparation.log").display()
    );
    run_logged(
        Command::new("bash").arg("-c")
            .arg("set -e; source \"$1/scripts/ci/consumer-fixture.sh\"; consumer_fixture_prepare \"$1\" \"$2\" \"$3\" ic-backup-journey-consumer '' \"$4\"")
            .arg("consumer-prepare").arg(root).arg(evidence).arg(selected)
            .arg(if registry { "registry" } else { "candidate" }),
        &evidence.join("graph-check.log"),
    );
}

fn compile(selected: &Path, evidence: &Path) {
    run_logged(
        Command::new("cargo")
            .arg(format!("+{}", env!("CARGO_PKG_RUST_VERSION")))
            .args(["build", "--offline", "--locked", "--manifest-path"])
            .arg(selected.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", evidence.join("target"))
            .env("RUSTUP_AUTO_INSTALL", "0"),
        &evidence.join("build.log"),
    );
}

fn run_logged(command: &mut Command, log: &Path) {
    let output = command.output().unwrap();
    fs::write(log, &output.stderr).unwrap();
    fs::write(log.with_extension("stdout"), &output.stdout).unwrap();
    assert!(
        output.status.success(),
        "{}: {}",
        log.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}
