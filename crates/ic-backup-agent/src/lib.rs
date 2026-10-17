//! Direct, single-update IC transport over exact original `ic-backup` reservations.
//!
//! This package performs no retries, polling, root-key discovery or journal transitions.
//! Integrations retain signed ingress evidence before dispatch and independently qualify
//! actual network trust, fresh permissions, application safety and never-dispatched custody.
//! A pending reservation or reconstructed request never grants permission to repeat a call.

mod fence_acquisition;
pub use fence_acquisition::AgentFenceAcquisitionProvider;
mod acknowledgement;
mod endpoint;
mod observation;
mod reply_evidence;
mod submission;
pub use observation::AgentObservationProvider;

mod request;
pub use request::ReservedUpdate;
mod update;
pub use update::AgentUpdateProvider;
mod transfer_read;
pub use transfer_read::AgentSnapshotTransferReadProvider;

use ic_agent::{
    Agent, Identity, RequestId,
    export::{Principal, reqwest},
};
use ic_backup::model::operation_plan::PlanContextRecord;
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
        let url = endpoint::admit_written_origin(endpoint)?;
        if timeout < Duration::from_secs(1)
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
}

/// Non-cloneable signed update tied to its fixed transport and borrowed original evidence.
///
/// Dropping, cancelling or failing submission grants no refund or repeat-call authority.
/// There is deliberately no envelope re-import/reissue API. Retained bytes are recovery
/// evidence; a successor call needs separate qualified admission and original accounting.
pub struct PreparedUpdate<'a> {
    transport: &'a AgentTransport,
    request: ReservedUpdate<'a>,
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
}

/// Redacted transport construction denial; provider errors own submission outcomes.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TransportError {
    /// Endpoint, timeout, trust or HTTP client configuration is invalid.
    #[error("invalid agent transport configuration")]
    Configuration,
    /// Actual signer or exact declared network/release context differs.
    #[error("agent transport context mismatch")]
    Context,
    /// Trusted identity could not sign the exact original request.
    #[error("agent transport signing failed")]
    Signing,
}
