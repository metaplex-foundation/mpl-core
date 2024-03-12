#![allow(missing_docs)]
use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    AddCollectionPluginArgs, AddPluginArgs, ApproveCollectionPluginAuthorityArgs,
    ApprovePluginAuthorityArgs, BurnArgs, BurnCollectionArgs, CompressArgs, CreateArgs,
    CreateCollectionArgs, DecompressArgs, RemoveCollectionPluginArgs, RemovePluginArgs,
    RevokeCollectionPluginAuthorityArgs, RevokePluginAuthorityArgs, TransferArgs, UpdateArgs,
    UpdateCollectionArgs, UpdateCollectionPluginArgs, UpdatePluginArgs,
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
    Create(CreateArgs),

    /// Create a new mpl-core Collection.
    /// This function creates the initial Collection, with or without plugins.
    #[account(0, writable, signer, name="collection", desc = "The address of the new asset")]
    #[account(1, optional, name="update_authority", desc = "The authority of the new asset")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    CreateCollection(CreateCollectionArgs),

    /// Add a plugin to an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddPlugin(AddPluginArgs),

    /// Add a plugin to an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    AddCollectionPlugin(AddCollectionPluginArgs),

    /// Remove a plugin from an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemovePlugin(RemovePluginArgs),

    /// Remove a plugin from an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RemoveCollectionPlugin(RemoveCollectionPluginArgs),

    /// Update a plugin of an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdatePlugin(UpdatePluginArgs),

    /// Update a plugin of an mpl-core Collection.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateCollectionPlugin(UpdateCollectionPluginArgs),

    /// Approve an authority to an mpl-core plugin.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    ApprovePluginAuthority(ApprovePluginAuthorityArgs),

    /// Approve an authority to an mpl-core plugin.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    ApproveCollectionPluginAuthority(ApproveCollectionPluginAuthorityArgs),

    /// Revoke an authority from an mpl-core plugin.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RevokePluginAuthority(RevokePluginAuthorityArgs),

    /// Revoke an authority from an mpl-core plugin.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    RevokeCollectionPluginAuthority(RevokeCollectionPluginAuthorityArgs),

    /// Burn an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Burn(BurnArgs),

    /// Burn an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    BurnCollection(BurnCollectionArgs),

    // Transfer an asset.
    /// Transfer an asset by changing its owner.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="new_owner", desc = "The new owner to which to transfer the asset")]
    #[account(5, optional, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Transfer(TransferArgs),

    /// Update an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The update authority or update authority delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, name="new_update_authority", desc = "The new update authority of the asset")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Update(UpdateArgs),

    /// Update an mpl-core.
    #[account(0, writable, name="collection", desc = "The address of the asset")]
    #[account(1, writable, signer, name="authority", desc = "The update authority or update authority delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, name="new_update_authority", desc = "The new update authority of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    UpdateCollection(UpdateCollectionArgs),

    /// Compress an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account receiving the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Compress(CompressArgs),

    /// Decompress an mpl-core.
    #[account(0, writable, name="asset", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Decompress(DecompressArgs),

    /// Collect
    /// This function creates the initial mpl-core
    #[account(0, writable, name="recipient", desc = "The address of the recipient")]
    Collect,
}
