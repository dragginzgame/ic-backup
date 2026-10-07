//! Actual pinned CLI routing refusal; not an installed transport or safe retry proof.

use super::{Method, backend::Backend, management};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord, attempt_journal::AttemptJournalRecordError,
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_operation_plan,
    },
};
use ic_host_process::tool::{
    AdmittedTool, ExecutionContext, ExecutionEvidence, ExecutionFailure, OutputLimits, ToolSpec,
};
use std::{
    ffi::OsString, fmt::Write as _, fs, os::unix::fs::PermissionsExt, path::Path, time::Duration,
};

fn retain(root: &Path, name: &str, evidence: &ExecutionEvidence) {
    fs::write(root.join(format!("{name}.stdout")), &evidence.stdout).unwrap();
    fs::write(root.join(format!("{name}.stderr")), &evidence.stderr).unwrap();
    fs::write(root.join(format!("{name}.json")), serde_json::to_vec_pretty(&serde_json::json!({
        "exit_code":evidence.status.and_then(|s|s.code()),
        "stdout_truncated":evidence.stdout_truncated,"stderr_truncated":evidence.stderr_truncated,
        "stdout_sha256":ArtifactChecksumRecord::from_bytes(&evidence.stdout).hash(),
        "stderr_sha256":ArtifactChecksumRecord::from_bytes(&evidence.stderr).hash(),
    })).unwrap()).unwrap();
}

fn admit_tool(context: &ExecutionContext<'_>, limits: OutputLimits) -> AdmittedTool {
    let tools = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.tools/ic");
    let executable = tools
        .join("bin/icp")
        .canonicalize()
        .expect("prepare pinned ICP CLI explicitly");
    let installed = fs::read_to_string(tools.join("files.sha256")).unwrap();
    let expected = installed
        .lines()
        .find_map(|row| row.strip_suffix("  bin/icp"))
        .expect("installed ICP checksum");
    let version_arguments = [OsString::from("--version")];
    let tool = AdmittedTool::admit(
        &ToolSpec {
            executable: &executable,
            sha256: expected.parse().unwrap(),
            executable_bytes: 256 * 1024 * 1024,
            version_arguments: &version_arguments,
            version_identity: "icp 1.6.0",
        },
        context,
        limits,
    )
    .unwrap();
    fs::write(context.current_dir.join("icp.sha256"), expected).unwrap();
    tool
}

fn status_control(
    tool: &AdmittedTool,
    context: &ExecutionContext<'_>,
    limits: OutputLimits,
    target: &str,
    gateway: &str,
    key_hex: &str,
) {
    // Independent fixture control: qualify this gateway/root/identity combination
    // with the dedicated correctly routed status command before the reserved call.
    // It is not a recovery observation or permission/dispatch authority.
    let status_args = [
        "canister",
        "status",
        target,
        "--network",
        gateway,
        "--root-key",
        key_hex,
        "--identity",
        "anonymous",
        "--json",
    ]
    .map(OsString::from);
    let control = tool.run(&status_args, context, limits).unwrap();
    retain(context.current_dir, "status-control", &control);
    assert!(!control.stdout_truncated && !control.stderr_truncated);
    let control_json: serde_json::Value = serde_json::from_slice(&control.stdout).unwrap();
    assert_eq!(control_json["id"], target);
    assert_eq!(control_json["status"], "Running");
    assert_eq!(
        control_json["settings"]["controllers"],
        serde_json::json!(["2vxsx-fae"])
    );
    fs::write(
        context.current_dir.join("status-control-parsed.json"),
        serde_json::to_vec_pretty(&control_json).unwrap(),
    )
    .unwrap();
}

fn assert_pending_reopen(
    mut journal: AttemptJournalGuard<'_>,
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    original: &[u8],
) {
    let view = journal.record().unwrap().view();
    assert_eq!(
        (view.pending_mutation, view.pending_observation),
        (Some(1), None)
    );
    assert_eq!((view.mutations_used, view.mutations_remaining), (1, 0));
    assert!(!view.applied);
    assert!(matches!(
        journal.reserve_mutation(),
        Err(AttemptJournalError::Record(
            AttemptJournalRecordError::MutationPending { attempt: 1 }
        ))
    ));
    assert_eq!(fs::read(journal.path()).unwrap(), original);
    drop(journal);
    let reopened = AttemptJournalGuard::open(layout, &plan.attempt_authority(1).unwrap()).unwrap();
    assert_eq!(fs::read(reopened.path()).unwrap(), original);
}

pub(crate) fn generic_management_route() {
    let mut backend = Backend::new_with_nns();
    let gateway = backend.pic.make_live(None);
    let root_key = backend.pic.root_key().expect("owned NNS root key");
    let mut key_hex = String::with_capacity(root_key.len() * 2);
    for byte in root_key {
        write!(key_hex, "{byte:02x}").unwrap();
    }
    let root = backend.root.join("icp probe with spaces");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    // The pinned CLI's portable override owns config/data/cache together.
    // XDG overrides alone do not isolate its macOS ProjectDirs implementation.
    let environment = [
        (
            OsString::from("ICP_HOME"),
            root.join("icp-home").into_os_string(),
        ),
        (OsString::from("TOKIO_WORKER_THREADS"), OsString::from("2")),
    ];
    let context = ExecutionContext {
        current_dir: &root,
        environment: &environment,
    };
    let limits = OutputLimits {
        stdout_bytes: 64 * 1024,
        stderr_bytes: 64 * 1024,
        timeout: Duration::from_secs(60),
    };
    let tool = admit_tool(&context, limits);
    fs::write(root.join("network.json"),serde_json::to_vec_pretty(&serde_json::json!({"url":gateway.as_str(),"root_key_hex":key_hex,"instance":backend.pic.instance_id(),"target":backend.target.to_text()})).unwrap()).unwrap();
    status_control(
        &tool,
        &context,
        limits,
        &backend.target.to_text(),
        gateway.as_str(),
        &key_hex,
    );
    let did = root.join("local.did");
    fs::write(&did, "service : { stop_canister : (reserved) -> (); }\n").unwrap();
    let request = management(&backend, Method::StopCanister, None);
    let arguments = root.join("arguments.candid");
    fs::write(&arguments, request.arguments()).unwrap();
    let layout = ic_backup::ops::persistence::BackupLayoutGuard::acquire(&root).unwrap();
    let plan = backend.plan(&request.digest(), 1);
    create_operation_plan(&layout, &plan).unwrap();
    let mut journal =
        AttemptJournalGuard::create(&layout, plan.attempt_authority(1).unwrap()).unwrap();
    journal.reserve_mutation().unwrap();
    let original = fs::read(journal.path()).unwrap();
    let call_args = [
        OsString::from("canister"),
        OsString::from("call"),
        OsString::from(request.receiver()),
        OsString::from(request.method().name()),
        OsString::from("--args-file"),
        arguments.into_os_string(),
        OsString::from("--args-format"),
        OsString::from("bin"),
        OsString::from("--candid"),
        did.into_os_string(),
        OsString::from("--network"),
        OsString::from(gateway.as_str()),
        OsString::from("--root-key"),
        OsString::from(key_hex),
        OsString::from("--identity"),
        OsString::from("anonymous"),
        OsString::from("--output"),
        OsString::from("hex"),
        OsString::from("--json"),
    ];
    fs::write(
        root.join("argv.json"),
        serde_json::to_vec_pretty(
            &call_args
                .iter()
                .map(|s| s.to_str().unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    )
    .unwrap();
    let error = tool
        .run(&call_args, &context, limits)
        .expect_err("pinned generic call has no effective management target");
    assert!(matches!(
        error.execution_error().unwrap().failure,
        ExecutionFailure::ExitStatus
    ));
    let evidence = error.evidence().expect("bounded actual CLI failure");
    retain(&root, "management", evidence);
    assert!(evidence.status.is_some_and(|status| !status.success()));
    assert!(!evidence.stdout_truncated && !evidence.stderr_truncated);
    // This simulator oracle is independent of the failed CLI reply. It does not
    // attribute a NotApplied receipt or settle/refund the original pending attempt.
    let status = backend.pic.canister_status(backend.target, None).unwrap();
    assert_eq!(serde_json::to_value(status.status).unwrap(), "running");
    assert_pending_reopen(journal, &layout, &plan, &original);
    fs::write(root.join("qualification-result.json"),serde_json::to_vec_pretty(&serde_json::json!({"result":"passed-unsupported-route","tool":"icp 1.6.0","receiver":request.receiver(),"required_effective_target":request.target(),"original_plan":plan.digest(),"mutation_used":1,"mutation_remaining":0,"pending_mutation":1,"dedicated_status_control":true,"target_running_oracle":true,"cli_effect_invocations":1,"fixture_status_control_invocations":1,"http_ingress_count":"not observed; no single-ingress/retry qualification inferred","production_transport":false})).unwrap()).unwrap();
    backend.pic.stop_live();
    eprintln!("Pinned ICP route refusal retained at {}", root.display());
}
