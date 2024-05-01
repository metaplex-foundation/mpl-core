mod generated;
mod indexable_asset;
mod plugins;
mod unified;

pub use generated::programs::MPL_CORE_ID as ID;
pub use generated::*;
pub use indexable_asset::*;
pub use plugins::*;
pub use unified::*;

impl Copy for generated::types::Key {}
