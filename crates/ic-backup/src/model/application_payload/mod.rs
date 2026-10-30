//! Shared opaque update syntax admission; routing, digests and authority remain caller-owned.

#[derive(Debug)]
pub(super) enum ApplicationPayloadError {
    ArgumentsTooLarge,
    InvalidMethod,
    InvalidPrincipal,
    ManagementPrincipal,
}

pub(super) fn validate_bytes(
    arguments: &[u8],
    argument_limit: usize,
    method: &str,
    method_limit: usize,
) -> Result<(), ApplicationPayloadError> {
    if arguments.len() > argument_limit {
        return Err(ApplicationPayloadError::ArgumentsTooLarge);
    }
    if method.is_empty()
        || method.len() > method_limit
        || !method.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(ApplicationPayloadError::InvalidMethod);
    }
    Ok(())
}

pub(super) fn principal(value: &str) -> Result<(String, Vec<u8>), ApplicationPayloadError> {
    let (text, principal) = super::principal::canonical_parts(value)
        .ok_or(ApplicationPayloadError::InvalidPrincipal)?;
    if principal.as_slice().is_empty() {
        return Err(ApplicationPayloadError::ManagementPrincipal);
    }
    Ok((text, principal.as_slice().to_vec()))
}
