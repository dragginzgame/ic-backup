//! Host-side snapshot backup and same-release recovery for Internet Computer canisters.
//!
//! This crate currently establishes the independent Rust package boundary.
//! Backup, restore, transport and persistence APIs have not been extracted yet.
//! The maintained extraction design is in `docs/extraction-design.md` at the
//! repository root.
//!
//! The library will own generic artifact, journal and recovery mechanisms.
//! Applications will own membership, release identity, control routing,
//! quiescence and external-effect settlement. Filesystem access, credentials
//! and restore execution remain on the operator host.
