mod generated;
mod hooked;
mod indexable_asset;

pub use generated::programs::MPL_CORE_ID as ID;
pub use generated::*;
pub use hooked::*;
pub use indexable_asset::*;

impl Copy for generated::types::Key {}
