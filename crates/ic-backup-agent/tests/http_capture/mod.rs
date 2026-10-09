//! Bounded test-server request capture; retained diagnostics are never transport receipts.

use ic_backup_agent::MAX_SIGNED_UPDATE_BYTES;
use serde_json::json;
use std::{
    fs,
    io::{self, Read},
    net::TcpStream,
    path::Path,
    time::{Duration, Instant},
};

const MAX_HEADER_BYTES: usize = 16 * 1024;
const MAX_REQUEST_BYTES: usize = MAX_HEADER_BYTES + MAX_SIGNED_UPDATE_BYTES;

#[derive(Debug)]
pub(super) enum ReadFailure {
    Io(io::Error),
    Deadline,
    Eof,
    HeaderBound,
    ContentLength,
    BodyBound,
    TrailingBytes,
    InvalidReadCount,
}

#[derive(Debug)]
pub(super) struct CapturedRequest {
    pub(super) bytes: Vec<u8>,
    pub(super) failure: Option<ReadFailure>,
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
            Some(ReadFailure::Io(error)) => json!({
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

/// Accepted socket flags differ across hosts. Select blocking reads explicitly,
/// then bound every read by the remaining original three-second capture deadline.
pub(super) fn capture(stream: &mut TcpStream, budget: Duration) -> CapturedRequest {
    if let Err(error) = stream.set_nonblocking(false) {
        return CapturedRequest {
            bytes: vec![],
            failure: Some(ReadFailure::Io(error)),
        };
    }
    read_request(
        |buffer, remaining| {
            stream.set_read_timeout(Some(remaining))?;
            stream.read(buffer)
        },
        budget,
    )
}

fn frame_length(bytes: &[u8]) -> Result<Option<usize>, ReadFailure> {
    let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
        return if bytes.len() > MAX_HEADER_BYTES {
            Err(ReadFailure::HeaderBound)
        } else {
            Ok(None)
        };
    };
    if end + 4 > MAX_HEADER_BYTES {
        return Err(ReadFailure::HeaderBound);
    }
    let header = std::str::from_utf8(&bytes[..end]).map_err(|_| ReadFailure::ContentLength)?;
    let mut length = None;
    for line in header.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            return Err(ReadFailure::ContentLength);
        };
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(ReadFailure::ContentLength);
        }
        if name.eq_ignore_ascii_case("content-length") {
            if length.is_some()
                || value.trim().is_empty()
                || !value.trim().bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(ReadFailure::ContentLength);
            }
            length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| ReadFailure::ContentLength)?,
            );
        }
    }
    let length = length.ok_or(ReadFailure::ContentLength)?;
    if length > MAX_SIGNED_UPDATE_BYTES {
        return Err(ReadFailure::BodyBound);
    }
    Ok(Some(end + 4 + length))
}

fn read_request(
    mut read: impl FnMut(&mut [u8], Duration) -> io::Result<usize>,
    budget: Duration,
) -> CapturedRequest {
    let deadline = Instant::now() + budget;
    let mut bytes = Vec::new();
    let mut chunk = [0; 4096];
    let failure = loop {
        let Some(remaining) = deadline
            .checked_duration_since(Instant::now())
            .filter(|value| !value.is_zero())
        else {
            break ReadFailure::Deadline;
        };
        let window = (MAX_REQUEST_BYTES - bytes.len()).min(chunk.len());
        if window == 0 {
            break ReadFailure::BodyBound;
        }
        match read(&mut chunk[..window], remaining) {
            Ok(0) => break ReadFailure::Eof,
            Ok(count) if count > window => break ReadFailure::InvalidReadCount,
            Ok(count) => bytes.extend_from_slice(&chunk[..count]),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => break ReadFailure::Io(error),
        }
        match frame_length(&bytes) {
            Ok(Some(length)) if bytes.len() == length => {
                return CapturedRequest {
                    bytes,
                    failure: None,
                };
            }
            Ok(Some(length)) if bytes.len() > length => break ReadFailure::TrailingBytes,
            Ok(_) => {}
            Err(failure) => break failure,
        }
    };
    CapturedRequest {
        bytes,
        failure: Some(failure),
    }
}

#[cfg(test)]
mod tests;
