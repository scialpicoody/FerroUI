//! Part C of the crate: the folders of the reference listed below, one module per folder.
//!
//! The part owns its classes ([`TYPES`]), their public Rust paths and its markup types
//! ([`register`]); `lib.rs` re-exports everything public of it.

use ferroui_base::TypeInfo;

/// The classes of this part, registered by `register_types`.
pub(crate) const TYPES: &[&TypeInfo] = &[];

/// Registers the Rust paths (`ferroui_base::ferro_rust_paths!`) and the markup types
/// (`MarkupType::register_all`) of this part. Called once by `register_types`.
pub(crate) fn register() {}
