//! Module: `hash`
//!
//! Responsibility: encode arbitrary IC wire bytes as lowercase hexadecimal.
//! Does not own: artifact traversal, topology canonicalization, or validation.
//! Boundary: formats byte fields; raw SHA-256 identity belongs to Host Artifacts.

/// Encode bytes as lowercase hexadecimal without allocation beyond output.
#[must_use]
pub(crate) fn hex_bytes(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    let mut encoded = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        encoded.push(hex_char(byte >> 4));
        encoded.push(hex_char(byte & 0x0f));
    }

    encoded
}

fn hex_char(nibble: u8) -> char {
    char::from(b"0123456789abcdef"[usize::from(nibble & 0x0f)])
}
