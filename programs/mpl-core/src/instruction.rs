#![allow(missing_docs)]
use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    AddCollectionExternalPluginV1Args, AddCollectionPluginV1Args, AddExternalPluginV1Args,
    AddPluginV1Args, AllocateCollectionExternalPluginDataV1Args, AllocateExternalPluginDataV1Args,
    ApproveCollectionPluginAuthorityV1Args, ApprovePluginAuthorityV1Args, BurnCollectionV1Args,
    BurnV1Args, ClearCollectionExternalPluginDataV1Args, ClearExternalPluginDataV1Args,
    CompressV1Args, CreateCollectionV1Args, CreateCollectionV2Args, CreateV1Args, CreateV2Args,
    DecompressV1Args, RemoveCollectionExternalPluginV1Args, RemoveCollectionPluginV1Args,
    RemoveExternalPluginV1Args, RemovePluginV1Args, RevokeCollectionPluginAuthorityV1Args,
    RevokePluginAuthorityV1Args, TransferV1Args, UpdateCollectionExternalPluginV1Args,
    UpdateCollectionPluginV1Args, UpdateCollectionV1Args, UpdateExternalPluginV1Args,
    UpdatePluginV1Args, UpdateV1Args, WriteCollectionExternalPluginDataV1Args,
    WriteExternalPluginDataV1Args,
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
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
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
    /// This function creates the initial Asset, with or without internal/external plugins.
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
    /// This function creates the initial Collection, with or without internal/external plugins.
    #[account(0, writable, signer, name="collection", desc = "The address of the new asset")]
    #[account(1, optional, name="update_authority", desc = "The authority of the new asset")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    CreateCollectionV2(CreateCollectionV2Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddExternalPluginV1(AddExternalPluginV1Args),

    /// Add an external plugin to an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddCollectionExternalPluginV1(AddCollectionExternalPluginV1Args),

    /// Remove an external plugin from an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemoveExternalPluginV1(RemoveExternalPluginV1Args),

    /// Remove an external plugin from an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemoveCollectionExternalPluginV1(RemoveCollectionExternalPluginV1Args),

    /// Update the data for an external plugin of an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateExternalPluginV1(UpdateExternalPluginV1Args),

    /// Update the data for an external plugin of an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateCollectionExternalPluginV1(UpdateCollectionExternalPluginV1Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AllocateExternalPluginDataV1(AllocateExternalPluginDataV1Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AllocateCollectionExternalPluginDataV1(AllocateCollectionExternalPluginDataV1Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    WriteExternalPluginDataV1(WriteExternalPluginDataV1Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    WriteCollectionExternalPluginDataV1(WriteCollectionExternalPluginDataV1Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    ClearExternalPluginDataV1(ClearExternalPluginDataV1Args),

    /// Add an external plugin to an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(2, optional, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    ClearCollectionExternalPluginDataV1(ClearCollectionExternalPluginDataV1Args),
}
