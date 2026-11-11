//! Emit a machine-readable manifest of first-party (internal) plugin
//! classification, for consumption by the JavaScript client code generator.
//!
//! Everything here is read from the program itself so the generated TypeScript
//! cannot drift from on-chain behavior:
//!   - `manager`     comes directly from `PluginType::manager()`, the authority
//!                   that gates plugin creation (Owner vs UpdateAuthority vs a
//!                   fixed Address). This drives the owner-managed vs
//!                   authority-managed split in the JS arg unions.
//!   - `createOnly`  is the set of plugins that can only be set at creation and
//!                   never added afterwards: the `PERMANENT_DELEGATES` const
//!                   plus Edition and BubblegumV2, whose `validate_add_plugin`
//!                   impls unconditionally reject. This drives the JS
//!                   `CreateOnlyPluginArgsV2` union.
//!
//! Run via `cargo run --manifest-path programs/mpl-core/Cargo.toml \
//!   --example dump_plugin_manifest` (wired into `pnpm generate:plugin-manifest`).
//! The output is written to stdout as JSON.

use mpl_core_program::plugins::{PluginType, PERMANENT_DELEGATES};
use mpl_core_program::state::Authority;
use strum::IntoEnumIterator;

fn manager_str(pt: &PluginType) -> &'static str {
    match pt.manager() {
        Authority::None => "None",
        Authority::Owner => "Owner",
        Authority::UpdateAuthority => "UpdateAuthority",
        Authority::Address { .. } => "Address",
    }
}

// A plugin is create-only when it can never be added after creation. That is
// the permanent-delegate set (authoritative const) plus Edition and BubblegumV2,
// which reject `validate_add_plugin` unconditionally.
fn is_create_only(pt: &PluginType) -> bool {
    PERMANENT_DELEGATES.contains(pt) || matches!(pt, PluginType::Edition | PluginType::BubblegumV2)
}

fn main() {
    let entries: Vec<String> = PluginType::iter()
        .map(|pt| {
            format!(
                "  {{ \"name\": \"{:?}\", \"manager\": \"{}\", \"createOnly\": {} }}",
                pt,
                manager_str(&pt),
                is_create_only(&pt)
            )
        })
        .collect();
    println!("[\n{}\n]", entries.join(",\n"));
}
