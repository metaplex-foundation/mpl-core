use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::state::{GroupV1, SolanaAccount};

/// Derive the PDA address for a `CollectionGroupPluginV1` account that tracks
/// which groups a given collection belongs to.
///
/// Seeds: [ "collection_group", collection_pubkey ]
pub(crate) fn find_collection_group_plugin_address(collection: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"collection_group", collection.as_ref()], &crate::ID)
}

/// Derive the PDA address for an `AssetGroupPluginV1` account that tracks
/// which groups a given asset belongs to.
///
/// Seeds: [ "asset_group", asset_pubkey ]
pub(crate) fn find_asset_group_plugin_address(asset: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"asset_group", asset.as_ref()], &crate::ID)
}

// ---------------------------------------------------------------------------
//  Cycle-detection helper
// ---------------------------------------------------------------------------
// Detects whether adding `child_key` as a child to the group identified by
// `parent_key` would create a circular reference anywhere in the group
// hierarchy. The function walks the `child_key`'s descendants by loading their
// accounts from the `accounts` slice and recurses depth-first.
// ---------------------------------------------------------------------------

pub(crate) fn is_circular_reference(
    parent_key: &Pubkey,
    child_key: &Pubkey,
    accounts: &[AccountInfo],
) -> Result<bool, ProgramError> {
    // Direct self-link always forms a cycle.
    if parent_key == child_key {
        return Ok(true);
    }

    // Locate the prospective child account from the passed-in accounts. If the
    // account is not provided the caller cannot prove a cycle so we treat it
    // as non-circular. This mirrors typical Solana CPI patterns where only the
    // relevant portion of the graph is supplied.
    let child_account_opt = accounts.iter().find(|a| a.key == child_key);

    if let Some(child_account) = child_account_opt {
        // Attempt to deserialize the group. If deserialization fails we treat
        // as non-circular and bubble the error upwards so the caller can
        // handle corruption appropriately.
        let child_group = GroupV1::load(child_account, 0)?;

        for grandchild_key in &child_group.child_groups {
            // Immediate cycle
            if grandchild_key == parent_key {
                return Ok(true);
            }

            // Recurse to detect deeper cycles. Propagate any program errors.
            if is_circular_reference(parent_key, grandchild_key, accounts)? {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
