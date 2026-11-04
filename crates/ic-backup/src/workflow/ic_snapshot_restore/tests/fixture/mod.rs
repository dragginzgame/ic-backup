//! Own repeated native restore declarations; guards are borrowed at each scenario's use site.
use super::{requirement, single, source};
use crate::{
    model::{
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_request::IcManagementMethodRecord,
        operation_plan::OperationPlanRecord,
        restore_safety::{RestoreSafetyLaneRecord, RestoreSafetyRequirementRecord},
    },
    ops::persistence::{
        BackupLayoutGuard, ExecutionStageGuard, create_execution_workflow, create_operation_plan,
        create_restore_safety_requirement,
    },
};
use std::{fs, path::PathBuf};

pub(super) struct RestoreFixture {
    pub root: PathBuf,
    pub source_layout: BackupLayoutGuard,
    pub layout: BackupLayoutGuard,
    pub source: OperationPlanRecord,
    pub binding: ExecutionStageBindingRecord,
    pub requirement: RestoreSafetyRequirementRecord,
    pub sequence: u64,
    plan: OperationPlanRecord,
}
impl RestoreFixture {
    pub fn new(
        prefix: &str,
        method: IcManagementMethodRecord,
        lane: RestoreSafetyLaneRecord,
    ) -> Self {
        let root = crate::tests::support::temp_dir(prefix);
        fs::create_dir(root.join("source")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let source_layout = BackupLayoutGuard::acquire(&root.join("source")).unwrap();
        let source = source();
        create_operation_plan(&source_layout, &source).unwrap();
        let plan = single(method);
        let sequence = plan.operations()[0].operation_sequence();
        let workflow = ExecutionWorkflowRecord::new(plan.clone());
        create_execution_workflow(&layout, &workflow).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
        let requirement = requirement(&plan, &source, lane);
        Self {
            root,
            source_layout,
            layout,
            source,
            binding,
            requirement,
            sequence,
            plan,
        }
    }
    pub fn stage(&self) -> ExecutionStageGuard<'_> {
        let stage =
            ExecutionStageGuard::prepare(&self.layout, self.binding.clone(), self.plan.clone())
                .unwrap();
        create_restore_safety_requirement(
            stage.layout().unwrap(),
            &self.source_layout,
            stage.plan(),
            &self.source,
            &self.requirement,
        )
        .unwrap();
        stage
    }
}
