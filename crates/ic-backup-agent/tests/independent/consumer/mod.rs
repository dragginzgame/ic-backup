//! Fixture entrypoint and application adapter around the supported standalone assembly.

mod application;
mod authority;
mod command;
mod config;
mod evidence;
mod standalone;

use clap::Parser;
pub(super) use command::Mode;
use command::{Action, Arguments};
pub(super) use config::{ApplicationConfig, Config};
use ic_backup::model::attempt_journal::AttemptBudgetRecord;
use ic_backup::workflow::execution_workflow::ExecutionWorkflowAdvance;
pub(super) use standalone::{Stage, error, read, retain};
use std::io;

#[tokio::main(flavor = "multi_thread")]
pub(super) async fn entry() {
    if let Err(error) = run().await {
        eprintln!("consumer refused: {error}");
        std::process::exit(1);
    }
}

async fn run() -> io::Result<()> {
    let Arguments { mode, root } =
        Arguments::try_parse().map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    let action = mode.action();
    let (phase, create) = match action {
        Action::ApplicationNew | Action::ApplicationObserve | Action::ApplicationRetained => {
            return application::run(&root, mode, action).await;
        }
        Action::CaptureNew => (standalone::Phase::Capture, true),
        Action::CaptureContinue | Action::InspectCapture => (standalone::Phase::Capture, false),
        Action::InspectRestore
        | Action::LoseReply
        | Action::Replay
        | Action::Restore
        | Action::RestoreStep
        | Action::Start => (standalone::Phase::Restore, false),
    };
    let config = Config::read(&root)?;
    let connection = config.connection()?;
    let authority = authority::Authority {
        root: root.clone(),
        config,
        mode,
    };
    let mut driver = standalone::Driver::new(root, connection, &authority);
    if create {
        driver.create()?;
    }
    if matches!(action, Action::InspectCapture | Action::InspectRestore) {
        serde_json::to_writer(io::stdout(), &driver.inspect(phase)?)?;
        println!();
        return Ok(());
    }
    if action == Action::Replay {
        return driver.release();
    }
    let limit = std::num::NonZeroUsize::new(match action {
        Action::RestoreStep => 1,
        Action::Restore | Action::LoseReply => 3,
        _ => usize::MAX,
    })
    .unwrap();
    match driver.advance(phase, limit).await? {
        ExecutionWorkflowAdvance::Complete(_) if action == Action::Start => driver.release(),
        ExecutionWorkflowAdvance::Complete(_) => Ok(()),
        ExecutionWorkflowAdvance::Paused
            if matches!(action, Action::Restore | Action::RestoreStep) =>
        {
            Ok(())
        }
        ExecutionWorkflowAdvance::Paused => Err(io::ErrorKind::InvalidData.into()),
    }
}
