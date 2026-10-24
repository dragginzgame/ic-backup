//! Build the canonical consumer with normal dependencies and an isolated lock/target.

use std::fmt::Write as _;
use std::{fs, path::PathBuf, process::Command, sync::OnceLock};

pub(super) fn binary() -> &'static PathBuf {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(|| {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap().to_owned();
        let evidence = super::super::support::root("independent-build");
        let selected = evidence.join("consumer");
        fs::create_dir_all(selected.join("src/consumer/transfer")).unwrap();
        fs::create_dir_all(selected.join("src/consumer/restore")).unwrap();
        for (name, bytes) in [
            ("mod.rs", include_str!("../consumer/mod.rs")),
            ("transfer/mod.rs", include_str!("../consumer/transfer/mod.rs")),
            ("restore/mod.rs", include_str!("../consumer/restore/mod.rs")),
        ] { fs::write(selected.join("src/consumer").join(name), bytes).unwrap(); }
        fs::write(selected.join("src/main.rs"), "mod consumer;\nfn main() { consumer::entry(); }\n").unwrap();
        let registry = match std::env::var("IC_BACKUP_CONSUMER_REGISTRY") {
            Err(std::env::VarError::NotPresent) => false,
            Ok(value) if value == "1" => true,
            _ => panic!("IC_BACKUP_CONSUMER_REGISTRY must be absent or exactly 1"),
        };
        let original = fs::read(root.join("Cargo.lock")).unwrap();
        let snapshot = Command::new("bash").arg("-c").arg("set -e; source \"$1/scripts/ci/consumer-fixture.sh\"; consumer_fixture_snapshot \"$1\" \"$2\" ''").arg("consumer-snapshot").arg(&root).arg(&evidence).output().unwrap();
        fs::write(evidence.join("snapshot.log"), &snapshot.stderr).unwrap();
        assert!(snapshot.status.success(), "{}", String::from_utf8_lossy(&snapshot.stderr));
        let metadata: serde_json::Value = serde_json::from_slice(&fs::read(evidence.join("original.json")).unwrap()).unwrap();
        let agent = metadata["packages"].as_array().unwrap().iter().find(|p| p["name"] == "ic-backup-agent").unwrap();
        let requirement = |name: &str| agent["dependencies"].as_array().unwrap().iter().find(|d| d["name"] == name).unwrap()["req"].as_str().unwrap();
        let mut manifest = format!("[workspace]\n[package]\nname = \"ic-backup-journey-consumer\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = {}\n[dependencies]\n", agent["rust_version"]);
        for name in ["ic-backup", "ic-backup-agent"] {
            if registry {
                writeln!(manifest, r#"{name} = {{ version = "={version}", default-features = false }}"#, version = env!("CARGO_PKG_VERSION")).unwrap();
            } else {
                writeln!(manifest, "{name} = {{ path = {} }}", serde_json::to_string(&root.join("crates").join(name)).unwrap()).unwrap();
            }
        }
        write!(manifest, "ic-platform = {{ version = {:?}, default-features = false, features = [\"agent\", \"candid\", \"management-canister-types\"] }}\nserde_json = {:?}\ntokio = {{ version = {:?}, features = [\"macros\", \"rt-multi-thread\"] }}\n", requirement("ic-platform"), requirement("serde_json"), requirement("tokio")).unwrap();
        fs::write(selected.join("Cargo.toml"), manifest).unwrap();
        let prepared = Command::new("bash").arg("-c").arg("set -e; source \"$1/scripts/ci/consumer-fixture.sh\"; consumer_fixture_prepare \"$1\" \"$2\" \"$3\" ic-backup-journey-consumer '' \"$4\"").arg("consumer-prepare").arg(&root).arg(&evidence).arg(&selected).arg(if registry { "registry" } else { "candidate" }).output().unwrap();
        fs::write(evidence.join("graph-check.log"), &prepared.stderr).unwrap();
        assert!(prepared.status.success(), "{}", String::from_utf8_lossy(&prepared.stderr));
        let output = Command::new("cargo").args(["build", "--offline", "--locked", "--manifest-path"]).arg(selected.join("Cargo.toml")).env("CARGO_TARGET_DIR", evidence.join("target")).env("RUSTUP_AUTO_INSTALL", "0").output().unwrap();
        fs::write(evidence.join("build.log"), &output.stderr).unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(fs::read(root.join("Cargo.lock")).unwrap(), original);
        eprintln!("Independent {} normal-consumer graph/build evidence: {}", if registry { "registry-only" } else { "candidate" }, evidence.display());
        evidence.join("target/debug/ic-backup-journey-consumer")
    })
}
