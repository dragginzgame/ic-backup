//! Pure host boundary to the integration-owned Root protocol. No adapter method
//! may sign, dispatch, observe a clock or access external state.
use super::{
    Step,
    effect::Effect,
    intent::{Error, Plan},
};
use crate::store::BackupDigest;

/// Original logical request from the fixed Backup procedure.
pub struct RootRequest<'a> {
    /// Original capture or restore approval, including caller, child and network.
    pub plan: Plan<'a>,
    /// Sixteen original operation ID bytes.
    pub id: [u8; 16],
    /// Approved Root authority generation.
    pub generation: u64,
    /// Fixed procedure slot including the original ordinal.
    pub step: Step,
    /// Typed Platform arguments supplied to the manager.
    pub effect: &'a Effect,
    /// Exact inner method and bounded encoded argument identity.
    pub method: &'static str,
    /// SHA-256 of the exact external arguments.
    pub arguments_hash: BackupDigest,
    /// Original encoded external argument length.
    pub arguments_bytes: u32,
    /// Original execute deadline, also used when preparing a later lookup.
    pub expires_at_ns: u64,
    /// Original cycles attached to the inner management call.
    pub cycles: u64,
    /// Maximum call cost including attachment.
    pub cycle_limit: u64,
    /// Original maximum retained inner outcome size.
    pub reply_bytes: u32,
}

/// A pure codec for the approved Root exchange.
pub trait RootCodec: std::fmt::Debug + Send + Sync {
    /// Immutable identity of the selected protocol implementation.
    fn protocol(&self) -> [u8; 32];
    /// Prepare the exact execute request and lookup of its original binding.
    /// Enforce the frame limit while encoding both requests.
    fn prepare(&self, request: RootRequest<'_>, frame_limit: u32) -> Option<Box<dyn RootBinding>>;
}

/// Original request frames and the decoder bound to their full logical identity.
pub trait RootBinding: Send + Sync {
    /// Complete Wire request frame for snapshot_execute.
    fn execute(&self) -> &[u8];
    /// Complete Wire request frame for snapshot_lookup of the original request.
    fn lookup(&self) -> &[u8];
    /// Check the response frame direction, method and complete original binding.
    /// Only a successful authenticated outer payload is supplied.
    fn decode(&self, exchange: RootExchange, payload: &[u8]) -> Option<RootOutcome>;
}

/// The separately signed Root ingress whose response is being interpreted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootExchange {
    /// Original outer execute update.
    Execute,
    /// Explicit original-result lookup update.
    Lookup,
}
/// Original inner outcome attested by Root.
pub enum RootOutcome {
    /// No terminal original result is available.
    Pending,
    /// Original management reply.
    Reply(Vec<u8>),
    /// Original management rejection code and exact message.
    Rejected(u32, String),
    /// Root retained a pre-dispatch refusal.
    Refused,
}

pub(in crate::store) enum Call {
    Direct(&'static str, Vec<u8>),
    Root {
        binding: Box<dyn RootBinding>,
        reply_bytes: u32,
    },
}

impl Call {
    pub(in crate::store) fn prepare(
        plan: Plan<'_>,
        step: Step,
        reference: u64,
        effect: &Effect,
        limit: usize,
    ) -> Result<Self, Error> {
        let (method, arguments) = effect.encode(limit).map_err(|_| Error::Profile)?;
        let profile = plan.profile();
        let Some(manager) = &profile.manager else {
            return Ok(Self::Direct(method, arguments));
        };
        let codec = profile
            .codec
            .filter(|codec| codec.protocol() == manager.protocol)
            .ok_or(Error::ManagerExchange)?;
        let mut id = [0; 16];
        for (ordinal, byte) in id.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&plan.operation_id()[2 * ordinal..2 * ordinal + 2], 16)
                .map_err(|_| Error::Profile)?;
        }
        let binding = codec
            .prepare(
                RootRequest {
                    plan,
                    id,
                    generation: manager.generation,
                    step,
                    effect,
                    method,
                    arguments_hash: BackupDigest::compute(&arguments),
                    arguments_bytes: arguments.len().try_into().map_err(|_| Error::Profile)?,
                    expires_at_ns: plan.admission(reference)?.submit_deadline_ns,
                    cycles: manager.cycles,
                    cycle_limit: manager.cycle_limit,
                    reply_bytes: manager.reply_bytes,
                },
                profile.request_bytes,
            )
            .ok_or(Error::ManagerExchange)?;
        for frame in [binding.execute(), binding.lookup()] {
            if frame.is_empty() || frame.len() > profile.request_bytes as usize {
                return Err(Error::ManagerExchange);
            }
        }
        Ok(Self::Root {
            binding,
            reply_bytes: manager.reply_bytes,
        })
    }
    pub(in crate::store) fn execute(&self) -> (&'static str, &[u8]) {
        match self {
            Self::Direct(method, arguments) => (method, arguments),
            Self::Root { binding, .. } => ("snapshot_execute", binding.execute()),
        }
    }
    pub(in crate::store) fn result(
        &self,
        exchange: RootExchange,
        rejected: bool,
        bytes: Vec<u8>,
    ) -> Result<Option<(bool, Vec<u8>)>, Error> {
        let Self::Root {
            binding,
            reply_bytes,
        } = self
        else {
            return Ok(Some((rejected, bytes)));
        };
        if rejected {
            return Ok(None);
        }
        match binding.decode(exchange, &bytes) {
            Some(RootOutcome::Reply(reply)) if reply.len() <= *reply_bytes as usize => {
                Ok(Some((false, reply)))
            }
            Some(RootOutcome::Rejected(_, message)) if message.len() <= *reply_bytes as usize => {
                Ok(Some((true, message.into_bytes())))
            }
            Some(RootOutcome::Refused) => Err(Error::RootRefused),
            Some(RootOutcome::Pending) | None => Ok(None),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests;
