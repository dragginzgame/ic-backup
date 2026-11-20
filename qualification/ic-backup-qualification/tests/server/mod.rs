//! Fixture-owned process evidence survives both explicit close and test unwinding.

use ic_testkit::server::{self, PreparedServer, Server, Stopped};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

///
/// RetainedServer
/// Keeps the qualification server and its diagnostic evidence together.
///

pub struct RetainedServer {
    server: Option<Server>,
    evidence: PathBuf,
}

impl RetainedServer {
    pub fn start(prepared: PreparedServer, evidence: &Path, lifetime: Duration) -> Self {
        let result = Server::start(
            prepared,
            server::Limits {
                startup: Duration::from_secs(30),
                lifetime,
                stdout_bytes: 4 * 1024 * 1024,
                stderr_bytes: 4 * 1024 * 1024,
            },
        );
        if let Err(server::Error::Startup {
            evidence: stopped, ..
        }) = &result
        {
            retain(evidence, stopped);
        }

        Self {
            server: Some(result.unwrap()),
            evidence: evidence.to_owned(),
        }
    }

    pub fn server(&self) -> &Server {
        self.server.as_ref().unwrap()
    }

    fn stop(&mut self) -> Stopped {
        let stopped = self.server.as_mut().unwrap().stop().unwrap();
        self.server = None;
        retain(&self.evidence, &stopped);

        stopped
    }
}

impl Drop for RetainedServer {
    fn drop(&mut self) {
        if let Some(server) = self.server.as_mut() {
            // Do not turn unwinding into a second panic. A failed stop remains evidence.
            match server.stop() {
                Ok(stopped) => {
                    let _ = fs::write(self.evidence.join("server.stdout"), &stopped.output.stdout);
                    let _ = fs::write(self.evidence.join("server.stderr"), &stopped.output.stderr);
                    let _ = fs::write(
                        self.evidence.join("server.process"),
                        format!("{stopped:?}\n"),
                    );
                }
                Err(error) => {
                    let _ = fs::write(
                        self.evidence.join("server.stop-error"),
                        format!("{error:?}\n"),
                    );
                }
            }
        }
    }
}

fn retain(evidence: &Path, stopped: &Stopped) {
    fs::write(evidence.join("server.stdout"), &stopped.output.stdout).unwrap();
    fs::write(evidence.join("server.stderr"), &stopped.output.stderr).unwrap();
    fs::write(evidence.join("server.process"), format!("{stopped:?}\n")).unwrap();
}

pub async fn close(instance: server::Instance, server: &mut RetainedServer) {
    let deletion = instance.close(Duration::from_secs(30)).await;
    let stopped = server.stop();
    assert!(matches!(deletion, server::Deletion::Attempted));
    assert_eq!(stopped.abandoned, 0);
    assert!(stopped.process.cleanup.is_empty(), "{stopped:?}");
    assert!(
        matches!(
            stopped.process.failure,
            Some(ic_host_process::Failure::Cancelled)
        ),
        "{stopped:?}"
    );
}
