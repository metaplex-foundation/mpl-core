#[cfg(all(feature = "anchor", not(feature = "anchor-0-32")))]
extern crate anchor_lang_0_31 as anchor_lang;
#[cfg(all(feature = "anchor", feature = "anchor-0-32"))]
extern crate anchor_lang_0_32 as anchor_lang;
#[cfg(not(feature = "anchor"))]
extern crate borsh1 as borsh;
#[cfg(not(feature = "anchor"))]
extern crate solana_program_v3 as solana_program;

mod generated;
mod hooked;
mod indexable_asset;

pub use generated::programs::MPL_CORE_ID as ID;
pub use generated::*;
pub use hooked::*;
pub use indexable_asset::*;

impl Copy for generated::types::Key {}
