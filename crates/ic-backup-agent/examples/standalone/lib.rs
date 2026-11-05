//! Supported standalone Core/Agent workflow assembly with mandatory application authority.

/// Complete bounded capture/restore assembly; application decisions have no defaults.
pub mod workflow;

#[cfg(all(test, unix))]
extern crate self as standalone;

#[cfg(all(test, unix))]
mod tests;
