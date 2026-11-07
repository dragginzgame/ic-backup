//! Native evidence replay setup; request decoding and coverage assertions stay with callers.

use ic_backup::{
    model::{
        attempt_journal::AttemptJournalView, operation_plan::OperationPlanRecord,
        restore_references::RestoreReferencesRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, create_operation_plan},
};
use std::fs;

pub(crate) struct PendingObservation {
    mutation: u32,
    observation: u32,
    view: AttemptJournalView,
    journal_bytes: Vec<u8>,
    plan_bytes: Vec<u8>,
    references: RestoreReferencesRecord,
}

impl PendingObservation {
    pub(crate) fn reserve<'a>(
        layout: &'a BackupLayoutGuard,
        plan: &OperationPlanRecord,
        request: &str,
    ) -> (AttemptJournalGuard<'a>, Self) {
        create_operation_plan(layout, plan).unwrap();
        layout
            .retain_restore(
                &layout.root().join("restore-reference.json"),
                plan.digest().hash(),
            )
            .unwrap();
        let references = layout.restore_references().unwrap();
        let mut journal =
            AttemptJournalGuard::create(layout, plan.attempt_authority(7).unwrap()).unwrap();
        let mutation = journal.reserve_mutation().unwrap();
        let observation = journal.reserve_observation(mutation, request).unwrap();
        let original = Self {
            mutation,
            observation,
            view: journal.record().unwrap().view(),
            journal_bytes: fs::read(journal.path()).unwrap(),
            plan_bytes: fs::read(layout.root().join("operation-plan.json")).unwrap(),
            references,
        };
        (journal, original)
    }

    pub(crate) fn assert_unchanged(
        &self,
        layout: &BackupLayoutGuard,
        journal: &mut AttemptJournalGuard<'_>,
        request: &str,
    ) {
        assert_eq!(journal.record().unwrap().view(), self.view);
        assert_eq!(self.view.pending_mutation, Some(self.mutation));
        assert_eq!(self.view.pending_observation, Some(self.observation));
        assert!(!self.view.applied);
        assert_eq!(
            (
                self.view.mutations_remaining,
                self.view.observations_remaining
            ),
            (0, 0)
        );
        assert!(journal.reserve_mutation().is_err());
        assert!(journal.reserve_observation(self.mutation, request).is_err());
        assert_eq!(fs::read(journal.path()).unwrap(), self.journal_bytes);
        assert_eq!(
            fs::read(layout.root().join("operation-plan.json")).unwrap(),
            self.plan_bytes
        );
        assert_eq!(layout.restore_references().unwrap(), self.references);
    }
}
