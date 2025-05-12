// Sub-modules containing the individual handlers.
pub mod assets;
pub mod child_groups;
pub mod close;
pub mod collections;
pub mod create;
pub mod metadata;

// Wild-card re-exports (same pattern as `processor/mod.rs`).
pub(crate) use assets::*;
pub(crate) use child_groups::*;
pub(crate) use close::*;
pub(crate) use collections::*;
pub(crate) use create::*;
pub(crate) use metadata::*;

pub(crate) use crate::plugins::{
    add_group_plugin, approve_group_plugin, remove_group_plugin, revoke_group_plugin,
    update_group_plugin, AddGroupPluginArgs, ApproveGroupPluginArgs, UpdateGroupPluginArgs,
};
