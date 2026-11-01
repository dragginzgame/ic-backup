//! Named invalid claims; each policy test retains its independent error assertion.
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
    ic_observation::IcObservationResponseInput,
    ic_snapshot_metadata::MAX_IC_SNAPSHOT_METADATA_BYTES,
    ic_update_response::IcUpdateResponseInput,
    operation_plan::PlanContextRecord,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ReceiptFault {
    NotApplied,
    Attempt,
    Request,
}
impl ReceiptFault {
    pub(crate) fn apply(self, receipt: &mut MutationReceiptRequest, wrong_request: &str) {
        match self {
            Self::NotApplied => receipt.outcome = MutationOutcomeRecord::NotApplied,
            Self::Attempt => receipt.attempt += 1,
            Self::Request => receipt.request = wrong_request.into(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ContextFault {
    Network,
    Caller,
    Release,
}
impl ContextFault {
    pub(crate) const ALL: [Self; 3] = [Self::Network, Self::Caller, Self::Release];

    pub(crate) const fn field(self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Caller => "caller",
            Self::Release => "release",
        }
    }

    pub(crate) fn apply(self, context: &mut PlanContextRecord, wrong_hash: &str) {
        let replacement = match self {
            Self::Caller => "aaaaa-aa",
            Self::Network | Self::Release => wrong_hash,
        };
        change_observed_context(context, self.field(), replacement.into());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResponseFault {
    Authority,
    Attempt,
    Network,
    Caller,
    Release,
    Target,
    InvalidWire,
    OversizedMetadata,
}
impl ResponseFault {
    pub(crate) const IDENTITY: [Self; 6] = [
        Self::Authority,
        Self::Attempt,
        Self::Network,
        Self::Caller,
        Self::Release,
        Self::Target,
    ];

    pub(crate) fn apply_with_later_failures(
        self,
        input: &mut IcUpdateResponseInput,
        wrong_authority: ArtifactChecksumRecord,
    ) {
        match self {
            Self::Authority => input.authority = wrong_authority,
            Self::Attempt => input.mutation_attempt = 2,
            Self::Network => change_context(input, "network", "12".repeat(32)),
            Self::Caller => change_context(input, "caller", "aaaaa-aa".into()),
            Self::Release => change_context(input, "release", "12".repeat(32)),
            Self::Target => input.target = "aaaaa-aa".into(),
            Self::InvalidWire => input.reply = b"DIDL\0\0".to_vec(),
            Self::OversizedMetadata => input.reply = vec![0; MAX_IC_SNAPSHOT_METADATA_BYTES + 1],
        }
        // Retain the original combined-invalid-input trace and its exact precedence.
        if self == Self::Authority {
            input.mutation_attempt = 2;
        }
        if matches!(self, Self::Authority | Self::Attempt) {
            change_context(input, "release", "12".repeat(32));
        }
        if matches!(
            self,
            Self::Authority | Self::Attempt | Self::Network | Self::Caller | Self::Release
        ) {
            input.target = "aaaaa-aa".into();
        }
        if Self::IDENTITY.contains(&self) {
            input.reply = b"invalid wire".to_vec();
        }
    }
}
fn change_context(input: &mut IcUpdateResponseInput, field: &str, value: String) {
    change_observed_context(&mut input.context, field, value);
}
fn change_observed_context(context: &mut PlanContextRecord, field: &str, value: String) {
    let mut fields = serde_json::to_value(&*context).unwrap();
    fields[field] = serde_json::Value::String(value);
    *context = serde_json::from_value(fields).unwrap();
}

/// Exact field drifts shared by the three independently validated observation families.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ObservationFault {
    Authority,
    BothAttempts,
    ObservationAttempt,
    Request,
    Network,
    Caller,
    Release,
    Target,
}
impl ObservationFault {
    pub(crate) const ALL: [Self; 8] = [
        Self::Authority,
        Self::BothAttempts,
        Self::ObservationAttempt,
        Self::Request,
        Self::Network,
        Self::Caller,
        Self::Release,
        Self::Target,
    ];

    pub(crate) fn apply(
        self,
        input: &mut IcObservationResponseInput,
        authority: ArtifactChecksumRecord,
        request: ArtifactChecksumRecord,
    ) {
        match self {
            Self::Authority => input.authority = authority,
            Self::BothAttempts => {
                input.mutation_attempt = 2;
                input.observation_attempt = 3;
            }
            Self::ObservationAttempt => input.observation_attempt = 3,
            Self::Request => input.request = request,
            Self::Network => {
                change_observed_context(&mut input.context, "network", "12".repeat(32));
            }
            Self::Caller => {
                change_observed_context(&mut input.context, "caller", "aaaaa-aa".into());
            }
            Self::Release => {
                change_observed_context(&mut input.context, "release", "12".repeat(32));
            }
            Self::Target => input.target = "aaaaa-aa".into(),
        }
    }
    pub(crate) fn assert_error(
        self,
        error: &crate::policy::ic_observation::IcObservationAssociationError,
    ) {
        use crate::policy::ic_observation::IcObservationAssociationError as E;
        let matches = match self {
            Self::Authority => matches!(error, E::AuthorityMismatch),
            Self::BothAttempts | Self::ObservationAttempt => matches!(error, E::AttemptMismatch),
            Self::Request => matches!(error, E::RequestMismatch),
            Self::Network | Self::Caller | Self::Release => matches!(error, E::ContextMismatch),
            Self::Target => matches!(error, E::TargetMismatch),
        };
        assert!(matches, "{self:?}: {error:?}");
    }
}
