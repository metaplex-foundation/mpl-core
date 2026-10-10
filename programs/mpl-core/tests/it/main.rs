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

// --- m1 ---
mod burn_transfer;
mod create;
mod plugin_management;
mod update;
// --- end m1 ---
// --- m2 ---
mod external_plugins;
mod plugins_internal;
// --- end m2 ---
// --- m3 ---
mod adversarial;
mod collect;
mod compression;
mod groups;
// --- end m3 ---
