//! Pure admission of a written root origin before URL normalization hides paths.

use crate::TransportError;
use reqwest::Url;
use std::net::IpAddr;

pub(super) fn admit_written_origin(endpoint: &str) -> Result<Url, TransportError> {
    let url = Url::parse(endpoint).map_err(|_| TransportError::Configuration)?;
    if !written_root_origin(endpoint) || !root_origin(&url) || !allowed_transport(&url) {
        return Err(TransportError::Configuration);
    }
    Ok(url)
}

fn written_root_origin(endpoint: &str) -> bool {
    // URL parsing removes dot segments, so inspect the written path before
    // accepting a root origin. A normalized `/path/..` is still a path input.
    let root_path = endpoint.split_once("://").is_some_and(|(_, authority)| {
        !authority.contains('\\')
            && authority
                .find('/')
                .is_none_or(|path_start| &authority[path_start..] == "/")
    });
    root_path
        && !endpoint
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
}

fn root_origin(url: &Url) -> bool {
    url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none()
}

fn allowed_transport(url: &Url) -> bool {
    match url.scheme() {
        "https" => true,
        "http" => url.host_str().is_some_and(|host| {
            host.trim_matches(['[', ']'])
                .parse::<IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::admit_written_origin;

    #[test]
    fn admits_https_and_literal_loopback_without_written_paths() {
        for endpoint in [
            "https://example.com",
            "https://example.com/",
            "http://127.0.0.1/",
            "http://[::1]/",
        ] {
            assert!(admit_written_origin(endpoint).is_ok(), "{endpoint}");
        }
    }

    #[test]
    fn rejects_normalized_paths_and_nonliteral_http_hosts() {
        for endpoint in [
            "https://example.com/path/..",
            "https://example.com\\path\\..",
            "http://localhost/",
            "http://example.com/",
            "https://example.com/?query",
            "https://example.com/#fragment",
            "https://user:password@example.com/",
            "https://example.com/%2e",
            "https://example.com/\n",
            "ftp://example.com/",
        ] {
            assert!(admit_written_origin(endpoint).is_err(), "{endpoint}");
        }
    }
}
