//! One bounded HTTP dispatch of already-retained initial status ingress.
use super::{
    CaptureError, CapturePlan, PreparedStatus, StatusAdmissionRecord, StatusCertificateError,
    StatusEvidence, StatusRequestError,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_http::{
    ReadError, read_body,
    reqwest::{self, Client, Url, header},
};
use ic_platform::{
    canister_sig_creation::extract_raw_root_pk_from_der,
    ic_protocol::{TransportCallResponse, decode_call_response},
};
use std::{
    net::IpAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

/// Fixed direct-host HTTP route and provisioned IC trust, with no mutable client escape.
///
/// This helper composes with a trusted [`super::StatusProvider`]; it is not a full
/// provider. It authenticates direct-key envelopes; the integration signs, admits current
/// application authority and retains requests through the operation before dispatch.
/// No identity, root discovery, polling, retry or detached task is supplied here.
pub struct StatusHttpTransport {
    client: Client,
    origin: Url,
    network: Sha256Digest,
    root_key_der: Vec<u8>,
    timeout: Duration,
}

/// Redacted transport refusal or uncertain dispatch. No variant grants a retry.
#[derive(Debug, Error)]
pub enum StatusHttpError {
    /// Invalid origin, root-key framing or finite timeout selection.
    #[error("invalid status HTTP configuration")]
    Configuration,
    /// Original authority, call content or request framing refused before dispatch.
    #[error("status HTTP request fails original admission")]
    Admission(#[source] CaptureError),
    /// The retained direct-key envelope failed signature or identity authentication.
    #[error("status HTTP request authentication failed")]
    Request(#[source] StatusRequestError),
    /// The retained plan differs from the configured network or trust root.
    #[error("status HTTP network or root binding mismatch")]
    Binding,
    /// Expiry/deadline elapsed or the host clock cannot supply Unix nanoseconds.
    #[error("status HTTP deadline or clock refused")]
    Deadline,
    /// Connection, body read, allocation or timeout failed; the request may have taken effect.
    #[error("status HTTP dispatch outcome is uncertain")]
    Uncertain,
    /// Non-200 status, including acceptance, redirects, throttling and server errors.
    #[error("status HTTP response has status {0}; outcome is unresolved")]
    HttpStatus(u16),
    /// Encoded HTTP body exceeds the original result allowance.
    #[error("status HTTP response exceeds original byte allowance")]
    ResponseLimit,
    /// Unsupported encoding/media type or malformed external response.
    #[error("invalid status HTTP response")]
    Response,
    /// Acceptance or a non-replicated rejection supplies no certified terminal result.
    #[error("status HTTP response has no terminal certificate")]
    NoCertificate,
    /// Original certificate, result association or complete evidence bounds refused.
    #[error("status HTTP certificate evidence refused")]
    Authentication(#[source] StatusCertificateError),
}

impl StatusHttpTransport {
    /// Exact HTTP route/trust/timeout commitment to retain before lookup signing.
    #[must_use]
    pub fn lookup_commitment(&self) -> [u8; 32] {
        self.commitment()
    }

    pub(super) fn admit_plan(&self, plan: &CapturePlan) -> Result<(), StatusHttpError> {
        self.admit_binding(plan.request().target.network, plan.request().trust_root)
    }

    pub(in crate::store) fn admit_binding(
        &self,
        network: Sha256Digest,
        trust_root: Sha256Digest,
    ) -> Result<(), StatusHttpError> {
        if network != self.network || trust_root != Sha256Digest::compute(&self.root_key_der) {
            return Err(StatusHttpError::Binding);
        }
        Ok(())
    }

    pub(in crate::store) fn root_key_der(&self) -> &[u8] {
        &self.root_key_der
    }

    pub(in crate::store) fn commitment(&self) -> [u8; 32] {
        let mut bytes = b"ic-backup/status-http/1\n".to_vec();
        bytes.extend_from_slice(self.network.as_bytes());
        bytes.extend_from_slice(Sha256Digest::compute(&self.root_key_der).as_bytes());
        bytes.extend_from_slice(&(self.timeout.as_nanos() as u64).to_le_bytes());
        bytes.extend_from_slice(self.origin.as_str().as_bytes());
        *Sha256Digest::compute(&bytes).as_bytes()
    }

    /// Configure HTTPS or an explicit literal-loopback HTTP root origin.
    ///
    /// Credentials, written paths (including normalized dot segments), query,
    /// fragment and whitespace refuse. HTTP DNS names and alternate numeric IP
    /// spellings refuse. The IC DER root and network commitment are provisioned
    /// out of band; TLS verification remains enabled independently of IC trust.
    /// Timeout must be nonzero and at most 300 seconds. No network request occurs.
    ///
    /// # Errors
    /// Refuses invalid configuration or HTTP client construction.
    pub fn new(
        endpoint: &str,
        network: Sha256Digest,
        root_key_der: Vec<u8>,
        timeout: Duration,
    ) -> Result<Self, StatusHttpError> {
        let origin = origin(endpoint)?;
        if timeout.is_zero()
            || timeout > Duration::from_secs(300)
            || extract_raw_root_pk_from_der(&root_key_der).is_err()
        {
            return Err(StatusHttpError::Configuration);
        }
        let client = Client::builder()
            .tls_backend_rustls()
            .retry(reqwest::retry::never())
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .referer(false)
            .http1_only()
            .pool_max_idle_per_host(0)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .timeout(timeout)
            .build()
            .map_err(|_| StatusHttpError::Configuration)?;
        Ok(Self {
            client,
            origin,
            network,
            root_key_der,
            timeout,
        })
    }

    /// Send the original envelope once, then authenticate a retained terminal result.
    ///
    /// Call only from the live provider invocation backed by the operation's durable
    /// submission reservation. Calling this helper directly does not prove that
    /// reservation and never creates authority to replay a request. It does not
    /// implement signing or application-continuity admission.
    ///
    /// The original network/root, exact status call and framed request bounds are
    /// checked before dispatch, along with direct-key signature authentication through
    /// [`super::authenticate_status_request`]. Effective routing uses the original target. The
    /// request timeout is the lesser of the configured maximum, original submit
    /// deadline and ingress expiry. That timeout spans connection and body reads;
    /// absolute time is rechecked before returning authenticated evidence.
    ///
    /// Only HTTP 200 CBOR containing a certificate can settle. Raw body accumulation
    /// is capped at the original result allowance before each copy; compressed
    /// responses refuse. Original certificate/result limits still apply after decoding.
    /// Headers and internal network buffers retain the HTTP supplier's bounds.
    /// No redirects, proxies, connection reuse, decompression, retries, polling,
    /// root fetching, re-signing or detached recovery occurs. Cancellation leaves
    /// original spending intact; the server may still execute an accepted request.
    ///
    /// ```no_run
    /// use ic_backup::store::{CapturePlan, PreparedStatus, StatusAdmissionRecord,
    ///     StatusEvidence, StatusHttpError, StatusHttpTransport};
    ///
    /// async fn reserved_provider_submission(
    ///     transport: &StatusHttpTransport, plan: &CapturePlan,
    ///     admission: &StatusAdmissionRecord, prepared: &PreparedStatus,
    /// ) -> Result<StatusEvidence, StatusHttpError> {
    ///     // Invoked by a provider only after advance_status reserves submission.
    ///     transport.submit_once(plan, admission, prepared).await
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses invalid bindings, exhausted time/bytes, nonterminal or malformed
    /// responses and failed certificate authentication. Errors never refund spending.
    ///
    /// # Panics
    /// The future requires a Tokio runtime with IO and timers enabled, as required
    /// by the selected HTTP client. Polling without that runtime can panic.
    pub async fn submit_once(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<StatusEvidence, StatusHttpError> {
        self.submit_call(plan, admission, prepared, super::CaptureCall::Status)
            .await
    }

    pub(super) async fn submit_call(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        call: super::CaptureCall,
    ) -> Result<StatusEvidence, StatusHttpError> {
        let arguments = call.arguments(plan).map_err(StatusHttpError::Admission)?;
        self.submit_expected(plan, admission, prepared, call.method_name(), &arguments)
            .await
    }

    pub(super) async fn submit_expected(
        &self,
        plan: &CapturePlan,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        method: &str,
        arguments: &[u8],
    ) -> Result<StatusEvidence, StatusHttpError> {
        super::request::authenticate_expected_request(plan, admission, prepared, method, arguments)
            .map_err(|error| match error {
                StatusRequestError::Admission(error) => StatusHttpError::Admission(error),
                error => StatusHttpError::Request(error),
            })?;
        self.admit_plan(plan)?;
        let deadline = admission.submit_deadline_ns.min(prepared.expires_ns);
        let certificate = self
            .call_certificate(plan.request().target.canister, admission, prepared)
            .await?;
        let (rejected, response) = super::certificate::authenticated_expected_response(
            plan,
            admission,
            prepared,
            &certificate,
            &self.root_key_der,
            method,
            arguments,
        )
        .map_err(StatusHttpError::Authentication)?;
        let evidence = StatusEvidence {
            rejected,
            response,
            proof: certificate,
        };
        evidence
            .encode(admission.result_bytes)
            .map_err(StatusHttpError::Admission)?;
        remaining(deadline)?;
        Ok(evidence)
    }

    // Caller admits exact signed call, original authority and durable reservation.
    pub(in crate::store) async fn call_certificate(
        &self,
        canister: ic_platform::Principal,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<Vec<u8>, StatusHttpError> {
        let mut url = self.origin.clone();
        url.set_path(&format!("/api/v3/canister/{}/call", canister.to_text()));
        let deadline = admission.submit_deadline_ns.min(prepared.expires_ns);
        let maximum = admission.result_bytes as usize;
        let bytes = self
            .post_cbor(url, prepared.envelope.clone(), deadline, maximum)
            .await?;
        let TransportCallResponse::Replied { certificate } =
            decode_call_response(&bytes, maximum).map_err(|_| StatusHttpError::Response)?
        else {
            return Err(StatusHttpError::NoCertificate);
        };
        Ok(certificate)
    }

    async fn post_cbor(
        &self,
        url: Url,
        body: Vec<u8>,
        deadline: u64,
        maximum: usize,
    ) -> Result<Vec<u8>, StatusHttpError> {
        let timeout = remaining(deadline)?.min(self.timeout);
        let response = self
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/cbor")
            .header(header::ACCEPT, "application/cbor")
            .body(body)
            .timeout(timeout)
            .send()
            .await
            .map_err(|_| StatusHttpError::Uncertain)?;
        if response.status().as_u16() != 200 {
            return Err(StatusHttpError::HttpStatus(response.status().as_u16()));
        }
        if response.headers().contains_key(header::CONTENT_ENCODING)
            || !response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| {
                    value
                        .split(';')
                        .next()
                        .is_some_and(|media| media.trim().eq_ignore_ascii_case("application/cbor"))
                })
        {
            return Err(StatusHttpError::Response);
        }
        let bytes = read_body(response, maximum)
            .await
            .map_err(|error| match error {
                ReadError::DeclaredLength { .. } | ReadError::Limit { .. } => {
                    StatusHttpError::ResponseLimit
                }
                ReadError::Transport(_) | ReadError::Allocation(_) => StatusHttpError::Uncertain,
            })?;
        remaining(deadline)?;
        Ok(bytes)
    }
}

pub(super) fn remaining(deadline: u64) -> Result<Duration, StatusHttpError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusHttpError::Deadline)?;
    let now = u64::try_from(now.as_nanos()).map_err(|_| StatusHttpError::Deadline)?;
    deadline
        .checked_sub(now)
        .filter(|value| *value > 0)
        .map(Duration::from_nanos)
        .ok_or(StatusHttpError::Deadline)
}

fn origin(endpoint: &str) -> Result<Url, StatusHttpError> {
    if endpoint.len() > 2048
        || endpoint
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(StatusHttpError::Configuration);
    }
    let written = endpoint
        .strip_prefix("https://")
        .or_else(|| endpoint.strip_prefix("http://"))
        .ok_or(StatusHttpError::Configuration)?;
    let authority = written.strip_suffix('/').unwrap_or(written);
    if authority.contains(['/', '\\', '@', '?', '#']) {
        return Err(StatusHttpError::Configuration);
    }
    let url = Url::parse(endpoint).map_err(|_| StatusHttpError::Configuration)?;
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(StatusHttpError::Configuration);
    }
    if url.scheme() == "http" {
        let host = if let Some(ipv6) = authority.strip_prefix('[') {
            ipv6.split_once(']')
                .map(|(host, _)| host)
                .ok_or(StatusHttpError::Configuration)?
        } else {
            authority
                .split(':')
                .next()
                .ok_or(StatusHttpError::Configuration)?
        };
        if !host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback()) {
            return Err(StatusHttpError::Configuration);
        }
    }
    Ok(url)
}

mod lookup;
pub use lookup::{CaptureLookupEvidence, CaptureLookupHttpError, CaptureLookupHttpPolicy};
