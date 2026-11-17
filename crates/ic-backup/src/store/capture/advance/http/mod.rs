//! One bounded HTTP dispatch of already-retained direct management ingress.
use super::{
    OperationStoreError, PreparedStatus, StatusAdmissionRecord, StatusCertificateError,
    StatusEvidence, StatusRequestError,
};
use crate::store::BackupDigest;
#[cfg(test)]
use crate::store::capture::CapturePolicy;
use ic_platform::certificate::PreparedRootKey;
use ic_platform::ingress::{CallResponse as TransportCallResponse, decode_call_response};
use reqwest::{Client, Url, header};
use std::{
    net::IpAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

/// Bounded original certificate received before a later failure. Not proof of success.
/// Retain these sensitive bytes under the integration's original custody; replay
/// must authenticate them again. Debug deliberately omits identity and contents.
pub struct ReceivedCertificate {
    pub(in crate::store) request_id: [u8; 32],
    pub(in crate::store) certificate: Vec<u8>,
}
impl ReceivedCertificate {
    /// Original management request, including when the certificate came from read-state.
    #[must_use]
    pub fn request_id(&self) -> [u8; 32] {
        self.request_id
    }
    /// Exact received certificate; it may have failed authentication or retention.
    #[must_use]
    pub fn certificate(&self) -> &[u8] {
        &self.certificate
    }
}
impl std::fmt::Debug for ReceivedCertificate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReceivedCertificate")
            .field("bytes", &self.certificate.len())
            .finish_non_exhaustive()
    }
}

/// Fixed direct-host HTTP route and provisioned IC trust, with no mutable client escape.
///
/// The fixed capture and restore operations use this transport
/// after durable reservation. It authenticates exact retained direct-key envelopes;
/// the integration supplies bounded signing and current application admission.
/// No identity, root discovery, polling, retry or detached task is supplied here.
pub struct BackupHttpTransport {
    client: Client,
    origin: Url,
    network: BackupDigest,
    root_key_der: Vec<u8>,
    timeout: Duration,
}

/// Redacted transport refusal or uncertain dispatch. No variant grants a retry.
#[derive(Debug, Error)]
pub enum BackupHttpError {
    /// A bounded certificate arrived, but authentication or closing admission failed.
    #[error("received certificate was not admitted: {source}")]
    AfterResponse {
        /// Original typed refusal. No retry or success is inferred.
        source: Box<BackupHttpError>,
        /// Sensitive original bytes retained for explicit recovery/diagnosis.
        received: ReceivedCertificate,
    },
    /// Invalid origin, root-key framing or finite timeout selection.
    #[error("invalid backup HTTP configuration")]
    Configuration,
    /// Original authority, call content or request framing refused before dispatch.
    #[error("backup HTTP request fails original admission")]
    Admission(#[source] OperationStoreError),
    /// The retained direct-key envelope failed signature or identity authentication.
    #[error("backup HTTP request authentication failed")]
    Request(#[source] StatusRequestError),
    /// The retained plan differs from the configured network or trust root.
    #[error("backup HTTP network or root binding mismatch")]
    Binding,
    /// Expiry/deadline elapsed or the host clock cannot supply Unix nanoseconds.
    #[error("backup HTTP deadline or clock refused")]
    Deadline,
    /// Connection, body read, allocation or timeout failed; the request may have taken effect.
    #[error("backup HTTP dispatch outcome is uncertain")]
    Uncertain,
    /// Non-200 status, including acceptance, redirects, throttling and server errors.
    #[error("backup HTTP response has status {0}; outcome is unresolved")]
    HttpStatus(u16),
    /// Encoded HTTP body exceeds the original result allowance.
    #[error("backup HTTP response exceeds original byte allowance")]
    ResponseLimit,
    /// Unsupported encoding/media type or malformed external response.
    #[error("invalid backup HTTP response")]
    Response,
    /// Acceptance or a non-replicated rejection supplies no certified terminal result.
    #[error("backup HTTP response has no terminal certificate")]
    NoCertificate,
    /// Original certificate, result association or complete evidence bounds refused.
    #[error("backup HTTP certificate evidence refused")]
    Authentication(#[source] StatusCertificateError),
}

impl BackupHttpError {
    /// Received bytes available despite this refusal; never a terminal receipt.
    #[must_use]
    pub fn received_certificate(&self) -> Option<&ReceivedCertificate> {
        match self {
            Self::AfterResponse { received, .. } => Some(received),
            _ => None,
        }
    }
    pub(in crate::store) fn after_response(
        self,
        request_id: [u8; 32],
        certificate: Vec<u8>,
    ) -> Self {
        Self::AfterResponse {
            source: Box::new(self),
            received: ReceivedCertificate {
                request_id,
                certificate,
            },
        }
    }
}

impl BackupHttpTransport {
    /// Exact HTTP route/trust/timeout commitment to retain before lookup signing.
    #[must_use]
    pub fn lookup_commitment(&self) -> [u8; 32] {
        self.commitment()
    }

    #[cfg(test)]
    pub(super) fn admit_plan(&self, plan: &CapturePolicy) -> Result<(), BackupHttpError> {
        self.admit_binding(plan.request().target.network, plan.request().trust_root)
    }

    pub(in crate::store) fn admit_binding(
        &self,
        network: BackupDigest,
        trust_root: BackupDigest,
    ) -> Result<(), BackupHttpError> {
        if network != self.network || trust_root != BackupDigest::compute(&self.root_key_der) {
            return Err(BackupHttpError::Binding);
        }
        Ok(())
    }

    pub(in crate::store) fn root_key_der(&self) -> &[u8] {
        &self.root_key_der
    }

    pub(in crate::store) fn commitment(&self) -> [u8; 32] {
        let mut bytes = b"ic-backup/status-http/1\n".to_vec();
        bytes.extend_from_slice(self.network.as_bytes());
        bytes.extend_from_slice(BackupDigest::compute(&self.root_key_der).as_bytes());
        bytes.extend_from_slice(&(self.timeout.as_nanos() as u64).to_le_bytes());
        bytes.extend_from_slice(self.origin.as_str().as_bytes());
        *BackupDigest::compute(&bytes).as_bytes()
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
        network: BackupDigest,
        root_key_der: Vec<u8>,
        timeout: Duration,
    ) -> Result<Self, BackupHttpError> {
        let origin = origin(endpoint)?;
        if timeout.is_zero()
            || timeout > Duration::from_secs(300)
            || PreparedRootKey::from_der(&root_key_der).is_err()
        {
            return Err(BackupHttpError::Configuration);
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
            .map_err(|_| BackupHttpError::Configuration)?;
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
    ///
    /// # Errors
    /// Refuses invalid bindings, exhausted time/bytes, nonterminal or malformed
    /// responses and failed certificate authentication. Errors never refund spending.
    ///
    /// # Panics
    /// The future requires a Tokio runtime with IO and timers enabled, as required
    /// by the selected HTTP client. Polling without that runtime can panic.
    #[cfg(test)]
    pub(in crate::store) async fn submit_once(
        &self,
        plan: &CapturePolicy,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<StatusEvidence, BackupHttpError> {
        self.submit_call(plan, admission, prepared, super::CaptureCall::Status)
            .await
    }

    #[cfg(test)]
    pub(super) async fn submit_call(
        &self,
        plan: &CapturePolicy,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        call: super::CaptureCall,
    ) -> Result<StatusEvidence, BackupHttpError> {
        let arguments = call.arguments(plan).map_err(BackupHttpError::Admission)?;
        self.submit_expected(
            super::request::DirectRequirements {
                target: plan.request().target,
                sender: plan.request().sender,
                trust_root: plan.request().trust_root,
            },
            admission,
            prepared,
            call.method_name(),
            &arguments,
        )
        .await
    }

    pub(in crate::store) async fn submit_expected(
        &self,
        requirements: super::request::DirectRequirements,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        method: &str,
        arguments: &[u8],
    ) -> Result<StatusEvidence, BackupHttpError> {
        super::request::authenticate_direct_request(
            requirements.sender,
            admission.request_bytes,
            prepared,
            method,
            arguments,
        )
        .map_err(|error| match error {
            StatusRequestError::Admission(error) => BackupHttpError::Admission(error),
            error => BackupHttpError::Request(error),
        })?;
        self.admit_binding(requirements.target.network, requirements.trust_root)?;
        let certificate = self
            .call_certificate(requirements.target.canister, admission, prepared)
            .await?;
        self.admit_received_certificate(&requirements, admission, prepared, certificate)
    }

    // Keep received bytes on all failures after bounded HTTP extraction, including
    // a deadline reached during authentication. This grants no new time window.
    pub(in crate::store) fn admit_received_certificate(
        &self,
        requirements: &super::request::DirectRequirements,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
        certificate: Vec<u8>,
    ) -> Result<StatusEvidence, BackupHttpError> {
        let deadline = admission.submit_deadline_ns.min(prepared.expires_ns);
        let (rejected, response) = super::CertificateRequirements {
            canister: requirements.target.canister,
            trust_root: requirements.trust_root,
        }
        .response(admission, prepared, &certificate, &self.root_key_der)
        .map_err(|error| {
            BackupHttpError::Authentication(error)
                .after_response(prepared.request_id, certificate.clone())
        })?;
        let evidence = StatusEvidence {
            rejected,
            response,
            proof: certificate,
        };
        let result = (|| {
            evidence
                .encode(admission.result_bytes)
                .map_err(BackupHttpError::Admission)?;
            remaining(deadline)?;
            Ok(())
        })();
        if let Err(error) = result {
            return Err(BackupHttpError::after_response(
                error,
                prepared.request_id,
                evidence.proof,
            ));
        }
        Ok(evidence)
    }

    // Caller admits exact signed call, original authority and durable reservation.
    pub(in crate::store) async fn call_certificate(
        &self,
        canister: ic_platform_types::Principal,
        admission: &StatusAdmissionRecord,
        prepared: &PreparedStatus,
    ) -> Result<Vec<u8>, BackupHttpError> {
        let mut url = self.origin.clone();
        url.set_path(&format!("/api/v3/canister/{}/call", canister.to_text()));
        let deadline = admission.submit_deadline_ns.min(prepared.expires_ns);
        let maximum = admission.result_bytes as usize;
        let bytes = self
            .post_cbor(url, prepared.envelope.clone(), deadline, maximum)
            .await?;
        let TransportCallResponse::Replied { certificate } = decode_call_response(&bytes, maximum)
            .map_err(|_| BackupHttpError::Response)?
            .value
        else {
            return Err(BackupHttpError::NoCertificate);
        };
        Ok(certificate)
    }

    async fn post_cbor(
        &self,
        url: Url,
        body: Vec<u8>,
        deadline: u64,
        maximum: usize,
    ) -> Result<Vec<u8>, BackupHttpError> {
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
            .map_err(|_| BackupHttpError::Uncertain)?;
        if response.status().as_u16() != 200 {
            return Err(BackupHttpError::HttpStatus(response.status().as_u16()));
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
            return Err(BackupHttpError::Response);
        }
        let bytes = ic_host_http::body(response, maximum)
            .await
            .map_err(|error| match error {
                ic_host_http::Error::TooLarge { .. } => BackupHttpError::ResponseLimit,
                ic_host_http::Error::Transport(_) | ic_host_http::Error::Allocation(_) => {
                    BackupHttpError::Uncertain
                }
            })?;
        Ok(bytes)
    }
}

pub(super) fn remaining(deadline: u64) -> Result<Duration, BackupHttpError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BackupHttpError::Deadline)?;
    let now = u64::try_from(now.as_nanos()).map_err(|_| BackupHttpError::Deadline)?;
    deadline
        .checked_sub(now)
        .filter(|value| *value > 0)
        .map(Duration::from_nanos)
        .ok_or(BackupHttpError::Deadline)
}

fn origin(endpoint: &str) -> Result<Url, BackupHttpError> {
    if endpoint.len() > 2048
        || endpoint
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(BackupHttpError::Configuration);
    }
    let written = endpoint
        .strip_prefix("https://")
        .or_else(|| endpoint.strip_prefix("http://"))
        .ok_or(BackupHttpError::Configuration)?;
    let authority = written.strip_suffix('/').unwrap_or(written);
    if authority.contains(['/', '\\', '@', '?', '#']) {
        return Err(BackupHttpError::Configuration);
    }
    let url = Url::parse(endpoint).map_err(|_| BackupHttpError::Configuration)?;
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(BackupHttpError::Configuration);
    }
    if url.scheme() == "http" {
        let host = if let Some(ipv6) = authority.strip_prefix('[') {
            ipv6.split_once(']')
                .map(|(host, _)| host)
                .ok_or(BackupHttpError::Configuration)?
        } else {
            authority
                .split(':')
                .next()
                .ok_or(BackupHttpError::Configuration)?
        };
        if !host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback()) {
            return Err(BackupHttpError::Configuration);
        }
    }
    Ok(url)
}

mod lookup;
#[cfg(test)]
pub use lookup::{CaptureLookupHttpError, CaptureLookupHttpPolicy};
