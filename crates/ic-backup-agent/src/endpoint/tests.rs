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
