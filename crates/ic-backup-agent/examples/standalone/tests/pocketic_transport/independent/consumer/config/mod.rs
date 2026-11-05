//! Decode the parent-owned journey configuration once before building any plans.
use ic_backup::model::operation_plan::OperationPlanRecord;
use ic_backup::model::operation_plan::PlanContextRecord;
use ic_backup::model::{artifacts::ArtifactChecksumRecord, attempt_journal::AttemptBudgetRecord};
use ic_backup_agent::AgentTransport;
use serde::{Deserialize, Serialize};
use std::{io, path::Path, sync::Arc, time::Duration};

#[derive(Deserialize, Serialize)]
pub(crate) struct Config {
    pub context: PlanContextRecord,
    pub target: String,
    pub endpoint: String,
    pub key: Vec<u8>,
    pub identity_seed: [u8; 32],
}

impl Config {
    pub(crate) fn read(root: &Path) -> io::Result<Self> {
        Ok(serde_json::from_slice(&super::read(
            &root.join("config.json"),
        )?)?)
    }

    pub(super) fn connection(&self) -> io::Result<super::standalone::Connection> {
        super::standalone::Connection::new(
            self.context.clone(),
            &self.target,
            &self.endpoint,
            Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(
                &self.identity_seed,
            )),
            self.key.clone(),
            Duration::from_secs(30),
        )
    }

    pub(super) fn singleton(
        &self,
        sequence: u64,
        request: &ArtifactChecksumRecord,
        budget: AttemptBudgetRecord,
    ) -> OperationPlanRecord {
        self.connection()
            .unwrap()
            .singleton(sequence, request, budget)
    }

    pub(super) fn transport(&self, plan: &OperationPlanRecord) -> AgentTransport {
        AgentTransport::new(
            plan.context().clone(),
            &self.endpoint,
            Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(
                &self.identity_seed,
            )),
            self.key.clone(),
            Duration::from_secs(30),
        )
        .unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct ApplicationConfig {
    #[serde(flatten)]
    pub connection: Config,
    pub reply_limit: usize,
}

#[cfg(test)]
mod tests;
