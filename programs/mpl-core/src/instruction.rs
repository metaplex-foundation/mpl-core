#![allow(missing_docs)]
use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    AddAssetsToGroupArgs, AddCollectionExternalPluginAdapterV1Args, AddCollectionPluginV1Args,
    AddCollectionsToGroupArgs, AddExternalPluginAdapterV1Args, AddGroupPluginArgs,
    AddGroupsToGroupArgs, AddPluginV1Args, ApproveCollectionPluginAuthorityV1Args,
    ApproveGroupPluginArgs, ApprovePluginAuthorityV1Args, BurnCollectionV1Args, BurnV1Args,
    CompressV1Args, CreateCollectionV1Args, CreateCollectionV2Args, CreateGroupArgs, CreateV1Args,
    CreateV2Args, DecompressV1Args, ExecuteV1Args, RemoveAssetsFromGroupArgs,
    RemoveCollectionExternalPluginAdapterV1Args, RemoveCollectionPluginV1Args,
    RemoveCollectionsFromGroupArgs, RemoveExternalPluginAdapterV1Args, RemoveGroupsFromGroupArgs,
    RemovePluginV1Args, RevokeCollectionPluginAuthorityV1Args, RevokePluginAuthorityV1Args,
    TransferV1Args, UpdateCollectionExternalPluginAdapterV1Args, UpdateCollectionInfoV1Args,
    UpdateCollectionPluginV1Args, UpdateCollectionV1Args, UpdateExternalPluginAdapterV1Args,
    UpdateGroupMetadataArgs, UpdateGroupPluginArgs, UpdatePluginV1Args, UpdateV1Args, UpdateV2Args,
    WriteCollectionExternalPluginAdapterDataV1Args, WriteExternalPluginAdapterDataV1Args,
};

/// Instructions supported by the mpl-core program.
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub(crate) enum MplAssetInstruction {
    /// Create a new mpl-core Asset.
    /// This function creates the initial Asset, with or without plugins.
    #[account(0, writable, signer, name="asset", desc = "The address of the new asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, optional, signer, name="authority", desc = "The authority signing for creation")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, name="owner", desc = "The owner of the new asset. Defaults to the authority if not present.")]
    #[account(5, optional, name="update_authority", desc = "The authority on the new asset")]
    #[account(6, name="system_program", desc = "The system program")]
    #[account(7, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    CreateV1(CreateV1Args),

    /// Create a new mpl-core Collection.
    /// This function creates the initial Collection, with or without plugins.
    #[account(0, writable, signer, name="collection", desc = "The address of the new asset")]
    #[account(1, optional, name="update_authority", desc = "The authority of the new asset")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    CreateCollectionV1(CreateCollectionV1Args),

    /// Add a plugin to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddPluginV1(AddPluginV1Args),

    /// Add a plugin to an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddCollectionPluginV1(AddCollectionPluginV1Args),

    /// Remove a plugin from an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemovePluginV1(RemovePluginV1Args),

    /// Remove a plugin from an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemoveCollectionPluginV1(RemoveCollectionPluginV1Args),

    /// Update a plugin of an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdatePluginV1(UpdatePluginV1Args),

    /// Update a plugin of an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateCollectionPluginV1(UpdateCollectionPluginV1Args),

    /// Approve an authority to an mpl-core plugin.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    ApprovePluginAuthorityV1(ApprovePluginAuthorityV1Args),

    /// Approve an authority to an mpl-core plugin.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    ApproveCollectionPluginAuthorityV1(ApproveCollectionPluginAuthorityV1Args),

    /// Revoke an authority from an mpl-core plugin.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RevokePluginAuthorityV1(RevokePluginAuthorityV1Args),

    /// Revoke an authority from an mpl-core plugin.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RevokeCollectionPluginAuthorityV1(RevokeCollectionPluginAuthorityV1Args),

    /// Burn an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, optional, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    BurnV1(BurnV1Args),

    /// Burn an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    BurnCollectionV1(BurnCollectionV1Args),

    // Transfer an asset.
    /// Transfer an asset by changing its owner.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="new_owner", desc = "The new owner to which to transfer the asset")]
    #[account(5, optional, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    TransferV1(TransferV1Args),

    /// Update an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The update authority or update authority delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateV1(UpdateV1Args),

    /// Update an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The update authority or update authority delegate of the asset")]
    #[account(3, optional, name="new_update_authority", desc = "The new update authority of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateCollectionV1(UpdateCollectionV1Args),

    /// Compress an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account receiving the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    CompressV1(CompressV1Args),

    /// Decompress an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    DecompressV1(DecompressV1Args),

    /// Collect
    /// This function creates the initial mpl-core
    #[account(0, writable, name="recipient1", desc = "The address of the recipient 1")]
    #[account(1, writable, name="recipient2", desc = "The address of the recipient 2")]
    Collect,

    /// Create a new mpl-core Asset V2.
    /// This function creates the initial Asset, with or without internal/external plugin adapters.
    #[account(0, writable, signer, name="asset", desc = "The address of the new asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, optional, signer, name="authority", desc = "The authority signing for creation")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, name="owner", desc = "The owner of the new asset. Defaults to the authority if not present.")]
    #[account(5, optional, name="update_authority", desc = "The authority on the new asset")]
    #[account(6, name="system_program", desc = "The system program")]
    #[account(7, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    CreateV2(CreateV2Args),

    /// Create a new mpl-core Collection V2.
    /// This function creates the initial Collection, with or without internal/external plugin adapters.
    #[account(0, writable, signer, name="collection", desc = "The address of the new asset")]
    #[account(1, optional, name="update_authority", desc = "The authority of the new asset")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    CreateCollectionV2(CreateCollectionV2Args),

    /// Add an external plugin adapter to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddExternalPluginAdapterV1(AddExternalPluginAdapterV1Args),

    /// Add an external plugin adapter to an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddCollectionExternalPluginAdapterV1(AddCollectionExternalPluginAdapterV1Args),

    /// Remove an external plugin adapter from an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemoveExternalPluginAdapterV1(RemoveExternalPluginAdapterV1Args),

    /// Remove an external plugin adapter from an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemoveCollectionExternalPluginAdapterV1(RemoveCollectionExternalPluginAdapterV1Args),

    /// Update the data for an external plugin adapter of an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateExternalPluginAdapterV1(UpdateExternalPluginAdapterV1Args),

    /// Update the data for an external plugin adapter of an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateCollectionExternalPluginAdapterV1(UpdateCollectionExternalPluginAdapterV1Args),

    /// Add an external plugin adapter to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The Data Authority of the External Plugin Adapter")]
    #[account(4, optional, name="buffer", desc = "The buffer to write to the external plugin")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    WriteExternalPluginAdapterDataV1(WriteExternalPluginAdapterDataV1Args),

    /// Add an external plugin adapter to an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The Data Authority of the External Plugin Adapter")]
    #[account(3, optional, name="buffer", desc = "The buffer to write to the external plugin")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    WriteCollectionExternalPluginAdapterDataV1(WriteCollectionExternalPluginAdapterDataV1Args),

    /// Update an mpl-core V2.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The update authority or update authority delegate of the asset")]
    #[account(4, optional, writable, name="new_collection", desc = "A new collection to which to move the asset")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateV2(UpdateV2Args),

    /// Execute an instruction on behalf of the owner.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, name="asset_signer", desc = "The signing PDA for the asset")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, name="program_id", desc = "The program id of the instruction")]
    ExecuteV1(ExecuteV1Args),

    /// Update mpl-core collection info (can only be called by Bubblegum program).
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, signer, name="bubblegum_signer", desc = "Bubblegum PDA signer")]
    UpdateCollectionInfoV1(UpdateCollectionInfoV1Args),
    
    // Group operations
    /// Create a new Group account.
    #[account(0, writable, signer, name="payer", desc = "The account paying for storage fees")]
    #[account(1, writable, name="group", desc = "The address of the new group account")]
    #[account(2, signer, name="update_authority", desc = "The update authority for the group")]
    #[account(3, name="system_program", desc = "The system program")]
    CreateGroup(CreateGroupArgs),

    /// Close an existing Group, returning its lamports to the update authority.
    #[account(0, writable, signer, name="update_authority", desc = "The update authority receiving lamports")]
    #[account(1, writable, name="group", desc = "The group account to close")]
    CloseGroup,

    /// Update metadata or authority fields on a Group.
    #[account(0, signer, name="authority", desc = "The current update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account to update")]
    UpdateGroupMetadata(UpdateGroupMetadataArgs),

    /// Add collections to a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account")]
    // Additional collection + authority accounts are passed after
    AddCollectionsToGroup(AddCollectionsToGroupArgs),

    /// Remove collections from a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account")]
    RemoveCollectionsFromGroup(RemoveCollectionsFromGroupArgs),

    /// Add assets to a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account")]
    // Additional asset + authority accounts are passed after
    AddAssetsToGroup(AddAssetsToGroupArgs),

    /// Remove assets from a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account")]
    RemoveAssetsFromGroup(RemoveAssetsFromGroupArgs),

    /// Add child Groups to a parent Group.
    #[account(0, signer, name="authority", desc = "The parent group update authority or delegated authority")]
    #[account(1, writable, name="parent_group", desc = "The parent group account")]
    AddGroupsToGroup(AddGroupsToGroupArgs),

    /// Remove child Groups from a parent Group.
    #[account(0, signer, name="authority", desc = "The parent group update authority or delegated authority")]
    #[account(1, writable, name="parent_group", desc = "The parent group account")]
    RemoveGroupsFromGroup(RemoveGroupsFromGroupArgs),

    /// Add a plugin to a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account")]
    #[account(2, writable, name="plugin", desc = "The plugin account to create")]
    #[account(3, name="plugin_program", desc = "The plugin program id")]
    #[account(4, name="system_program", desc = "The system program")]
    AddGroupPlugin(AddGroupPluginArgs),

    /// Remove a plugin from a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, writable, name="group", desc = "The group account")]
    #[account(2, writable, name="plugin", desc = "The plugin account to remove")]
    RemoveGroupPlugin,

    /// Update a plugin attached to a Group.
    #[account(0, signer, name="authority", desc = "The plugin authority as defined by plugin config")]
    #[account(1, name="group", desc = "The group account")]
    #[account(2, writable, name="plugin", desc = "The plugin account to update")]
    UpdateGroupPlugin(UpdateGroupPluginArgs),

    /// Approve an existing plugin on a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, name="group", desc = "The group account")]
    #[account(2, writable, name="plugin", desc = "The plugin account to approve")]
    ApproveGroupPlugin(ApproveGroupPluginArgs),

    /// Revoke (disable) a plugin on a Group.
    #[account(0, signer, name="authority", desc = "The group update authority or delegated authority")]
    #[account(1, name="group", desc = "The group account")]
    #[account(2, writable, name="plugin", desc = "The plugin account to revoke")]
    RevokeGroupPlugin,
}
