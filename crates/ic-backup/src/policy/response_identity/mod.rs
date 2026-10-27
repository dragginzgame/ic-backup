//! Ordered passive update-response identity; reservation and decoding stay with callers.

use crate::model::{
    artifacts::ArtifactChecksumRecord, ic_update_response::IcUpdateResponseInput,
    operation_plan::PlanContextRecord,
};

pub(super) struct ResponseIdentity<'a> {
    pub authority: &'a ArtifactChecksumRecord,
    pub mutation_attempt: u32,
    pub context: &'a PlanContextRecord,
    pub target: &'a str,
}

impl<'a> From<&'a IcUpdateResponseInput> for ResponseIdentity<'a> {
    fn from(input: &'a IcUpdateResponseInput) -> Self {
        Self {
            authority: &input.authority,
            mutation_attempt: input.mutation_attempt,
            context: &input.context,
            target: &input.target,
        }
    }
}

impl ResponseIdentity<'_> {
    pub fn compare(&self, expected: &Self) -> Result<(), ResponseIdentityMismatch> {
        if self.authority != expected.authority {
            return Err(ResponseIdentityMismatch::Authority);
        }
        if self.mutation_attempt != expected.mutation_attempt {
            return Err(ResponseIdentityMismatch::Attempt);
        }
        if self.context != expected.context {
            return Err(ResponseIdentityMismatch::Context);
        }
        if self.target != expected.target {
            return Err(ResponseIdentityMismatch::Target);
        }
        Ok(())
    }
}

pub(super) enum ResponseIdentityMismatch {
    Authority,
    Attempt,
    Context,
    Target,
}
