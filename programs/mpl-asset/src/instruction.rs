use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    BurnArgs, CompressArgs, CreateArgs, DecompressArgs, DelegateArgs, FreezeArgs, RevokeArgs,
    ThawArgs, TransferArgs, UpdateArgs,
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MplAssetInstruction {
    /// Create a new mpl-asset.
    /// This function creates the initial mpl-asset
    #[account(0, writable, signer, name="asset_address", desc = "The address of the new asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, optional, name="update_authority", desc = "The authority of the new asset")]
    #[account(3, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, name="owner", desc = "The owner of the new asset. Defaults to the authority if not present.")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Create(CreateArgs),

    /// Delegate an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="owner", desc = "The owner of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="delegate", desc = "The new simple delegate for the asset")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Delegate(DelegateArgs),

    /// Delegate an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="owner", desc = "The owner of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="delegate", desc = "The delegate to be revoked for the asset")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Revoke(RevokeArgs),

    //TODO: Implement this instruction
    /// Burn an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, optional, writable, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Burn(BurnArgs),

    // Transfer an asset.
    // danenbm WIP
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, signer, name="authority", desc = "The owner or delegate of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="new_owner", desc = "The new owner to which to transfer the asset")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Transfer(TransferArgs),

    //TODO: Implement this instruction
    /// Update an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, signer, name="authority", desc = "The update authority or update authority delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, name="new_update_authority", desc = "The new update authority of the asset")]
    #[account(4, name="system_program", desc = "The system program")]
    #[account(5, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Update(UpdateArgs),

    /// Freeze an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, signer, name="delegate", desc = "The delegate of the asset")]
    #[account(2, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Freeze(FreezeArgs),

    /// Thaw an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, signer, name="delegate", desc = "The delegate of the asset")]
    #[account(2, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Thaw(ThawArgs),

    //TODO: Implement this instruction
    /// Create a new mpl-asset.
    /// This function creates the initial mpl-asset
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, signer, name="owner", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account receiving the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Compress(CompressArgs),

    //TODO: Implement this instruction
    /// Create a new mpl-asset.
    /// This function creates the initial mpl-asset
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, signer, name="owner", desc = "The owner or delegate of the asset")]
    #[account(2, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, name="system_program", desc = "The system program")]
    #[account(4, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Decompress(DecompressArgs),
}
