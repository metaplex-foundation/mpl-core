use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

use crate::processor::{
    BurnArgs, CompressArgs, CreateArgs, DecompressArgs, DelegateArgs, FreezeArgs, MigrateArgs,
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

    //TODO: Implement this instruction
    // keith WIP
    /// Migrate an mpl-token-metadata asset to an mpl-asset.
    #[account(0, writable, signer, name="asset_address", desc = "The address of the new asset")]
    #[account(1, optional, signer, name="owner", desc = "The authority of the new asset")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    // Dependent on how migration is handled
    #[account(3, optional, writable, name="collection", desc="mpl-token-metadata collection metadata or mpl-asset collection")]
    #[account(4, writable, name="token", desc="Token account")]
    #[account(5, writable, name="mint", desc="Mint of token asset")]
    #[account(6, writable, name="metadata", desc="Metadata (pda of ['metadata', program id, mint id])")]
    #[account(7, writable, name="edition", desc="Edition of token asset")]
    #[account(8, optional, writable, name="owner_token_record", desc="Owner token record account")]
    #[account(9, name="spl_token_program", desc="SPL Token Program")]
    #[account(10, name="spl_ata_program", desc="SPL Associated Token Account program")]
    #[account(11, name="system_program", desc = "The system program")]
    #[account(12, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    #[account(13, optional, name="authorization_rules_program", desc="Token Authorization Rules Program")]
    #[account(14, optional, name="authorization_rules", desc="Token Authorization Rules account")]
    Migrate(MigrateArgs),

    /// Delegate an mpl-asset.
    #[account(0, writable, name="asset_address", desc = "The address of the asset")]
    #[account(1, optional, name="collection", desc = "The collection to which the asset belongs")]
    #[account(2, writable, signer, name="owner", desc = "The owner of the asset")]
    #[account(3, optional, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(4, name="delegate", desc = "The new simple delegate for the asset")]
    #[account(5, name="system_program", desc = "The system program")]
    #[account(6, optional, name="log_wrapper", desc = "The SPL Noop Program")]
    Delegate(DelegateArgs),

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
