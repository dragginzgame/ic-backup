//! Ordered passive restore-fence matching; callers retain purpose-specific denials.

use crate::model::{
    restore_safety::ApplicationFenceState,
    restore_safety::{RestoreFenceBindingRecord, RestoreFenceEvidence},
};

pub(super) enum RestoreFenceMismatch {
    Identity,
    MembershipRevision,
    ExternalObligationsRevision,
    Inactive,
}

pub(super) fn validate(
    original: &RestoreFenceBindingRecord,
    observed: &RestoreFenceEvidence,
) -> Result<(), RestoreFenceMismatch> {
    if observed.binding.identity != original.identity {
        return Err(RestoreFenceMismatch::Identity);
    }
    if observed.binding.membership_revision != original.membership_revision {
        return Err(RestoreFenceMismatch::MembershipRevision);
    }
    if observed.binding.external_obligations_revision != original.external_obligations_revision {
        return Err(RestoreFenceMismatch::ExternalObligationsRevision);
    }
    if observed.state != ApplicationFenceState::Active {
        return Err(RestoreFenceMismatch::Inactive);
    }
    Ok(())
}
