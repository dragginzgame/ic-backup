//! Fixed capture dispatcher over the shared one-shot recorder.
use super::custody::CaptureOperation;
use super::runtime::{Action, Connection, Error, Guard, Progress, Signer};
use super::{FactRecord, Journal, Step};
use crate::store::{
    CapturePlan, OperationStoreError,
    capture::procedure::{Next, Procedure},
};
fn action(step: Step) -> Result<Action, OperationStoreError> {
    Ok(match step {
        Step::InitialStatus | Step::ListSnapshots | Step::FinalStatus => Action::Inspect,
        Step::TakeSnapshot => Action::TakeSnapshot,
        Step::Metadata | Step::Data(_) => Action::ReadSnapshot,
        _ => return Err(OperationStoreError::Changed),
    })
}
impl CaptureOperation {
    pub(in crate::store) fn project<'a>(
        &self,
        journal: &Journal,
        intent: &'a CapturePlan,
    ) -> Result<Procedure<'a>, super::replay::Error> {
        journal.replay(
            intent,
            |call, role| self.read_role(call, role),
            |flow| self.materialize(journal, flow, false).map(|_| ()),
        )
    }
    /// Perform the next fixed capture action with at most one new management submission.
    /// # Errors
    /// Refuses changed custody, authority, finite allowance or original result policy.
    pub async fn advance<S: Signer, G: Guard>(
        &mut self,
        connection: &mut Connection<S, G>,
    ) -> Result<Progress, Error<S::Error, G::Error>> {
        let intent = self.intent.clone();
        let mut journal = self.check()?;
        let mut flow = self.project(&journal, &intent)?;
        let step = match flow.next()? {
            Next::Materialize => {
                if !journal.materialization {
                    journal = self.append(&journal, FactRecord::MaterializationStarted, None)?;
                }
                self.usable = false;
                self.materialize(&journal, &flow, true)?;
                self.finish_write(&journal)?;
                flow.after_materialization()?;
                Step::FinalStatus
            }
            Next::Publish => {
                return Ok(Progress::Complete(Box::new(
                    self.publish_closed(journal, &flow)?,
                )));
            }
            Next::Call(step) => step,
        };
        if journal.calls.last().is_none_or(|c| c.step != step) {
            match step {
                Step::TakeSnapshot => journal.remaining(4)?,
                Step::Data(0) => journal.remaining(
                    flow.transfer()?
                        .data_calls()
                        .checked_add(1)
                        .ok_or(OperationStoreError::StatusAdmission)?,
                )?,
                _ => (),
            }
        }
        let (method, arguments) = flow.request()?;
        let Some(result) = self
            .run(
                journal,
                connection,
                step,
                action(step)?,
                method,
                &arguments,
                || Ok(()),
            )
            .await?
        else {
            return Ok(Progress::NeedsAttention(step));
        };
        if step == Step::FinalStatus {
            self.materialize(&self.check()?, &flow, false)?;
        }
        flow.accept(result.rejected, step, result.request_id, &result.response)?;
        Ok(Progress::Advanced)
    }
    /// Observe the original pending capture request once, with separate finite allowance.
    /// # Errors
    /// Refuses resolved, changed or exhausted original authority and invalid evidence.
    pub async fn reconcile_observation<S: Signer, G: Guard>(
        &mut self,
        connection: &mut Connection<S, G>,
    ) -> Result<Progress, Error<S::Error, G::Error>> {
        let intent = self.intent.clone();
        let journal = self.check()?;
        let mut flow = self.project(&journal, &intent)?;
        let step = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?
            .step;
        if flow.next()? != Next::Call(step) {
            return Err(Error::Resolved);
        }
        let Some(result) = self.observe(journal, connection, || Ok(())).await? else {
            return Ok(Progress::NeedsAttention(step));
        };
        if step == Step::FinalStatus {
            self.materialize(&self.check()?, &flow, false)?;
        }
        flow.accept(result.rejected, step, result.request_id, &result.response)?;
        Ok(Progress::Advanced)
    }
}
