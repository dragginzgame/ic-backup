//! Bounded test-server request capture; retained diagnostics are never transport receipts.

use ic_backup_agent::MAX_SIGNED_UPDATE_BYTES;
use ic_host_process::testing::http::{
    HttpCapture, HttpCaptureFailure, HttpContentLength, capture_http_request,
};
use serde_json::json;
use std::{fs, io, net::TcpStream, path::Path, time::Duration};

const MAX_HEADER_BYTES: usize = 16 * 1024;

#[derive(Debug)]
pub(super) struct CapturedRequest {
    pub(super) bytes: Vec<u8>,
    pub(super) failure: Option<HttpCaptureFailure>,
}
impl CapturedRequest {
    pub(super) fn complete_bytes(&self, root: &Path) -> &[u8] {
        assert!(
            self.failure.is_none(),
            "incomplete HTTP request: {:?}; retained at {}",
            self.failure,
            root.display()
        );
        &self.bytes
    }
    pub(super) fn retain(&self, root: &Path, index: usize) -> io::Result<()> {
        fs::write(root.join(format!("http-request-{index}.bin")), &self.bytes)?;
        let failure = match &self.failure {
            Some(HttpCaptureFailure::Io(error)) => json!({
                "kind": format!("{:?}", error.kind()),
                "raw_os_error": error.raw_os_error(),
                "message": error.to_string(),
            }),
            other => json!({"capture_failure": format!("{other:?}")}),
        };
        fs::write(
            root.join(format!("http-request-{index}.json")),
            serde_json::to_vec(&json!({
                "bytes": self.bytes.len(), "complete": self.failure.is_none(),
                "failure": failure,
            }))?,
        )
    }
}

/// Select the Agent request bounds and require an explicit Content-Length.
pub(super) fn capture(stream: &mut TcpStream, budget: Duration) -> CapturedRequest {
    let HttpCapture { bytes, failure } = capture_http_request(
        stream,
        budget,
        MAX_HEADER_BYTES,
        MAX_SIGNED_UPDATE_BYTES,
        HttpContentLength::Required,
    );
    CapturedRequest { bytes, failure }
}

#[cfg(test)]
mod tests;
