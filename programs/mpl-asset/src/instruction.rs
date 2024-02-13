use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MplAssetInstruction {
    /// Create a new mpl-asset.
    /// This function creates the initial mpl-asset
    #[account(0, writable, signer, name="asset_address", desc = "The address of the new asset")]
    #[account(1, optional, name="authority", desc = "The authority of the new asset")]
    #[account(2, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(3, optional, name="owner", desc = "The owner of the new asset. Defaults to the authority if not present.")]
    #[account(4, name="system_program", desc = "The system program")]
    Create(CreateArgs),
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateArgs {
    pub name: String,
    pub uri: String,
}
