//! The mpl-core integration test crate.
//!
//! One crate for every Mollusk test module (each file under `tests/` would
//! otherwise be its own crate linking the whole runtime). Modules mirror the
//! `clients/js/test` tree; `harness` holds the self-checks of the shared
//! library in `tests/common`.

#[path = "../common/mod.rs"]
mod common;

mod harness;

mod account_ownership;
mod agent_identity;
mod execution_delegate;
