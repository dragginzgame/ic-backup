//! Capture dispatch and explicit application-owned reply settlement.
use super::{
    Application, Digest, Driver, ExecutionStageGuard, IcSnapshotReply, Method, PreparedUpdate,
    Reply, ReservedUpdate, Stage, error, io,
};
use ic_backup::{
    ops::persistence::AttemptJournalGuard, workflow::ic_snapshot_capture::capture_snapshot,
};

impl<A: Application> Driver<'_, A> {
    pub(super) async fn capture_stage(
        &self,
        stage: &ExecutionStageGuard<'_>,
    ) -> io::Result<Digest> {
        let payload = self.payload(Method::TakeCanisterSnapshot, None);
        let transport = &self.config.transport;
        let root = stage.layout().map_err(error)?.root();
        let mut provider = ic_backup_agent::AgentUpdateProvider::new(
            transport,
            |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                self.application
                    .retain_ingress(Stage::Capture, root, request, prepared)
                    .map_err(|_| {
                        ic_backup::ports::ic_mutation::IcMutationProviderError::Unavailable
                    })
            },
        );
        let reply = capture_snapshot(stage, 0, &payload, &mut provider, async |_| {
            self.application
                .admit(Stage::Capture, stage.plan())
                .await
                .map(|_| ())
        })
        .await
        .map_err(error)?;
        let receipt = self
            .application
            .qualify_reply(Reply {
                root,
                stage: Stage::Capture,
                plan: stage.plan(),
                sequence: 0,
                attempt: 1,
                request: &payload.digest(),
                arguments: payload.arguments(),
                bytes: &reply.input().reply,
            })
            .await?;
        let mut journal = AttemptJournalGuard::open(
            stage.layout().map_err(error)?,
            &stage.plan().attempt_authority(0).map_err(error)?,
        )
        .map_err(error)?;
        journal.record_mutation(receipt).map_err(error)?;
        Ok(IcSnapshotReply::decode(&payload, &reply.input().reply)
            .map_err(error)?
            .digest())
    }
}
