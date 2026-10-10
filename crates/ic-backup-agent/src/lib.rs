//! Direct, single-update IC transport over exact original `ic-backup` reservations.
//!
//! This package performs no retries, polling, root-key discovery or journal transitions.
//! Integrations retain signed ingress evidence before dispatch and independently qualify
//! actual network trust, fresh permissions, application safety and never-dispatched custody.
//! A pending reservation or reconstructed request never grants permission to repeat a call.

mod request;
pub use request::ReservedUpdate;
mod mutation;
pub use mutation::AgentMutationProvider;
mod transfer_read;
pub use transfer_read::AgentSnapshotTransferReadProvider;

use ic_agent::{
    Agent, Identity, RequestId,
    agent::CallResponse,
    export::{Principal, reqwest},
};
use ic_backup::model::{attempt_journal::AttemptJournalRecord, operation_plan::PlanContextRecord};
use std::{sync::Arc, time::Duration};
use thiserror::Error;

/// Maximum HTTP body including certificate framing; method replies have smaller owner bounds.
pub const MAX_HTTP_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
/// Maximum opaque application fence-acquisition reply; no application interpretation is inferred.
pub const MAX_APPLICATION_REPLY_BYTES: usize = 1024 * 1024;
/// Maximum retained signed ingress envelope including bounded upload bytes and signing overhead.
pub const MAX_SIGNED_UPDATE_BYTES: usize = 3 * 1024 * 1024;

/// Explicit fixed endpoint, trusted root and caller; no implicit discovery or mutable agent escape.
///
/// The context's network/release hashes are integration declarations. This transport
/// checks their exact match and actual signer, but does not derive network identity
/// from a URL or prove application release/permissions. Consumers own trust provisioning.
pub struct AgentTransport {
    agent: Agent,
    context: PlanContextRecord,
}

impl AgentTransport {
    /// Configure a no-retry, no-redirect client with a finite per-HTTP deadline.
    ///
    /// HTTPS origins and explicit loopback HTTP origins are accepted. Credentials,
    /// paths, queries and fragments reject. The trusted root is supplied out of band;
    /// it is never fetched from the endpoint. Timeout/ingress expiry must be 1..=300s.
    /// An async executor with network/timer support is required at submission.
    /// # Errors
    /// Rejects invalid configuration, signer/context mismatch or client construction.
    pub fn new(
        context: PlanContextRecord,
        endpoint: &str,
        identity: Arc<dyn Identity>,
        trusted_root_key: Vec<u8>,
        timeout: Duration,
    ) -> Result<Self, TransportError> {
        let url = reqwest::Url::parse(endpoint).map_err(|_| TransportError::Configuration)?;
        let local = url.host_str().is_some_and(|host| {
            host.trim_matches(['[', ']'])
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
        });
        if !(url.scheme() == "https" || (url.scheme() == "http" && local))
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
            || timeout < Duration::from_secs(1)
            || timeout > Duration::from_secs(300)
            || trusted_root_key.is_empty()
            || trusted_root_key.len() > 1024
        {
            return Err(TransportError::Configuration);
        }
        if identity
            .sender()
            .map_err(|_| TransportError::Signing)?
            .to_text()
            != context.caller()
        {
            return Err(TransportError::Context);
        }
        let client = reqwest::Client::builder()
            .retry(reqwest::retry::never())
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .http1_only()
            .timeout(timeout)
            .build()
            .map_err(|_| TransportError::Configuration)?;
        // Supplying middleware bypasses Agent's default 429/503 RetryLogic wrapper.
        let agent = Agent::builder()
            .with_url(url.as_str())
            .with_arc_identity(identity)
            .with_arc_http_middleware(Arc::new(client))
            .with_max_tcp_error_retries(0)
            .with_max_response_body_size(MAX_HTTP_RESPONSE_BYTES)
            .with_ingress_expiry(timeout)
            .build()
            .map_err(|_| TransportError::Configuration)?;
        agent.set_root_key(trusted_root_key);
        Ok(Self { agent, context })
    }

    /// Sign an exact existing request after rechecking its original pending journal.
    ///
    /// No network call occurs. Retain the returned exact envelope/request ID with
    /// original plan, reservation and exclusive command custody before submission.
    /// Holding the original journal guard prevents cooperating record changes.
    /// Fresh authority and proof of no prior dispatch remain integration prerequisites.
    /// # Errors
    /// Rejects changed reservations/context, signing failures or oversized envelopes.
    pub fn prepare<'a>(
        &'a self,
        request: ReservedUpdate<'a>,
        journal: &'a AttemptJournalRecord,
    ) -> Result<PreparedUpdate<'a>, TransportError> {
        if !request.validate(journal) {
            return Err(TransportError::Reservation);
        }
        let wire = request.wire();
        if wire.context != &self.context {
            return Err(TransportError::Context);
        }
        let receiver =
            Principal::from_text(wire.receiver).map_err(|_| TransportError::Configuration)?;
        let target =
            Principal::from_text(wire.target).map_err(|_| TransportError::Configuration)?;
        let signed = self
            .agent
            .update(&receiver, wire.method)
            .with_effective_canister_id(target)
            .with_arg(wire.arguments)
            .sign()
            .map_err(|_| TransportError::Signing)?;
        if signed.sender.to_text() != self.context.caller() {
            return Err(TransportError::Context);
        }
        if signed.signed_update.len() > MAX_SIGNED_UPDATE_BYTES {
            return Err(TransportError::EnvelopeTooLarge);
        }
        let reply_limit = wire.reply_limit;
        Ok(PreparedUpdate {
            transport: self,
            request,
            journal,
            target,
            envelope: signed.signed_update,
            request_id: signed.request_id,
            reply_limit,
        })
    }
}

/// Non-cloneable signed update tied to its fixed transport and borrowed original evidence.
///
/// Dropping, cancelling or failing submission grants no refund or repeat-call authority.
/// There is deliberately no envelope re-import/reissue API. Retained bytes are recovery
/// evidence; a successor call needs separate qualified admission and original accounting.
pub struct PreparedUpdate<'a> {
    transport: &'a AgentTransport,
    request: ReservedUpdate<'a>,
    journal: &'a AttemptJournalRecord,
    target: Principal,
    envelope: Vec<u8>,
    request_id: RequestId,
    reply_limit: usize,
}
impl PreparedUpdate<'_> {
    /// Read the exact signed bytes to retain durably before dispatch; contains opaque sensitive arguments.
    #[must_use]
    pub fn envelope(&self) -> &[u8] {
        &self.envelope
    }
    /// Read the original IC ingress identity before dispatch.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Consume this preparation for one update HTTP request, without automatic wait or observation.
    ///
    /// A verified certificate reply is still passive method evidence, never an
    /// automatic Applied receipt. Accepted/processing results return Pending; callers
    /// must account separate recovery observations. Every error, cancellation or lost
    /// reply leaves original spending pending; no error establishes nonapplication.
    /// # Errors
    /// Rejects stale reservations before dispatch; transport/authentication failures
    /// and over-limit replies retain indeterminate original evidence.
    pub async fn submit(self) -> Result<UpdateOutcome, TransportError> {
        if !self.request.validate(self.journal) {
            return Err(TransportError::Reservation);
        }
        let response = self
            .transport
            .agent
            .update_signed(self.target, self.envelope)
            .await
            .map_err(|_| TransportError::Indeterminate)?;
        match response {
            CallResponse::Response(reply) => {
                if reply.len() > self.reply_limit {
                    return Err(TransportError::ReplyTooLarge);
                }
                Ok(UpdateOutcome::Replied {
                    request_id: self.request_id,
                    reply,
                })
            }
            CallResponse::Poll(id) if id == self.request_id => {
                Ok(UpdateOutcome::Pending { request_id: id })
            }
            CallResponse::Poll(_) => Err(TransportError::Indeterminate),
        }
    }
}

/// Passive single-update result; neither branch mutates or settles original journals.
pub enum UpdateOutcome {
    /// Locally certificate-verified exact request reply; method decoding/attribution remain separate.
    Replied {
        /// Exact original signed ingress identity.
        request_id: RequestId,
        /// Bounded raw method reply; use the existing method-specific decoder.
        reply: Vec<u8>,
    },
    /// Accepted/processing without a settled reply; no follow-up call was made.
    Pending {
        /// Retained original ingress identity, never a redispatch permit.
        request_id: RequestId,
    },
}

/// Redacted transport denial; no variant authorizes retry, refund or nonapplication.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TransportError {
    /// Endpoint, timeout, trust or HTTP client configuration is invalid.
    #[error("invalid agent transport configuration")]
    Configuration,
    /// Actual signer or exact declared network/release context differs.
    #[error("agent transport context mismatch")]
    Context,
    /// Original reservation is absent, replaced, settled or already recovering.
    #[error("agent transport reservation mismatch")]
    Reservation,
    /// Trusted identity could not sign the exact original request.
    #[error("agent transport signing failed")]
    Signing,
    /// Signed envelope exceeded its finite limit; no dispatch.
    #[error("agent transport envelope too large")]
    EnvelopeTooLarge,
    /// Submission, authentication or reply outcome is unknown; preserve pending evidence.
    #[error("agent update indeterminate")]
    Indeterminate,
    /// Reply exceeded its method owner bound after submission; preserve pending evidence.
    #[error("agent update reply too large")]
    ReplyTooLarge,
}
