//! Named invalid claims; each policy test retains its independent error assertion.
use crate::model::{
    artifacts::ArtifactChecksumRecord, ic_snapshot_metadata::MAX_IC_SNAPSHOT_METADATA_BYTES,
    ic_update_response::IcUpdateResponseInput,
};

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
    let mut context = serde_json::to_value(&input.context).unwrap();
    context[field] = serde_json::Value::String(value);
    input.context = serde_json::from_value(context).unwrap();
}
