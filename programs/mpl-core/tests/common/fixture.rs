//! Create-through-the-program fixtures on top of `MolluskContext`.
//!
//! A [`Fixture`] keeps an account store between instructions (successful
//! results persist, failed ones do not, program and sysvar accounts are
//! hydrated automatically), so a test reads:
//!
//! ```ignore
//! let f = Fixture::new();
//! let asset = f.create_asset(CreateAssetArgs::default());
//! let r = f.run(&ix::transfer_v1(asset, None, payer, None, new_owner, None, None));
//! assert_ok(&r);
//! assert_eq!(f.asset(&asset).owner, new_owner);
//! ```
//!
//! This mirrors `clients/js/test/_setupRaw.ts` (`createAsset`,
//! `createCollection`, `createAssetWithCollection`).

use {
    super::{
        accounts::{
            empty_account, payer_account, ACCOUNT_LAMPORTS, DEFAULT_ASSET_NAME,
            DEFAULT_COLLECTION_NAME, DEFAULT_URI,
        },
        assert::{assert_account_burned, assert_ok},
        core_mollusk, ix,
        read::{parse_any, read_asset, read_collection, ParsedAccount},
    },
    mollusk_svm::{result::InstructionResult, MolluskContext},
    mpl_core::types::{ExternalPluginAdapterInitInfo, PluginAuthorityPair},
    mpl_core_program::state::{AssetV1, CollectionV1},
    solana_account::Account,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    std::collections::HashMap,
};

/// A Mollusk context with the mpl-core program and the auxiliary builtins.
pub struct Fixture {
    /// The underlying context; use it directly for `process_and_validate_*`.
    pub ctx: MolluskContext<HashMap<Pubkey, Account>>,
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Arguments for [`Fixture::create_asset`]; every field has a default.
#[derive(Clone, Debug)]
pub struct CreateAssetArgs {
    /// The payer (default: a freshly funded account).
    pub payer: Option<Pubkey>,
    /// The `owner` account (default: absent, so the program uses the update authority / payer).
    pub owner: Option<Pubkey>,
    /// The `update_authority` account (default: absent, so the program uses the payer).
    pub update_authority: Option<Pubkey>,
    /// The `authority` signer (default: absent, so the payer signs).
    pub authority: Option<Pubkey>,
    /// The collection to mint into.
    pub collection: Option<Pubkey>,
    /// The asset name.
    pub name: String,
    /// The asset URI.
    pub uri: String,
    /// Internal plugins to initialize.
    pub plugins: Vec<PluginAuthorityPair>,
    /// External adapters to initialize.
    pub adapters: Vec<ExternalPluginAdapterInitInfo>,
    /// Remaining accounts (e.g. an agent identity PDA marked as signer).
    pub remaining: Vec<AccountMeta>,
}

impl Default for CreateAssetArgs {
    fn default() -> Self {
        Self {
            payer: None,
            owner: None,
            update_authority: None,
            authority: None,
            collection: None,
            name: DEFAULT_ASSET_NAME.to_string(),
            uri: DEFAULT_URI.to_string(),
            plugins: vec![],
            adapters: vec![],
            remaining: vec![],
        }
    }
}

/// Arguments for [`Fixture::create_collection`]; every field has a default.
#[derive(Clone, Debug)]
pub struct CreateCollectionArgs {
    /// The payer (default: a freshly funded account).
    pub payer: Option<Pubkey>,
    /// The `update_authority` account (default: absent, so the program uses the payer).
    pub update_authority: Option<Pubkey>,
    /// The collection name.
    pub name: String,
    /// The collection URI.
    pub uri: String,
    /// Internal plugins to initialize.
    pub plugins: Vec<PluginAuthorityPair>,
    /// External adapters to initialize.
    pub adapters: Vec<ExternalPluginAdapterInitInfo>,
    /// Remaining accounts.
    pub remaining: Vec<AccountMeta>,
}

impl Default for CreateCollectionArgs {
    fn default() -> Self {
        Self {
            payer: None,
            update_authority: None,
            name: DEFAULT_COLLECTION_NAME.to_string(),
            uri: DEFAULT_URI.to_string(),
            plugins: vec![],
            adapters: vec![],
            remaining: vec![],
        }
    }
}

fn non_empty<T>(items: Vec<T>) -> Option<Vec<T>> {
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}

impl Fixture {
    /// A fresh context from [`core_mollusk`] with an empty account store.
    pub fn new() -> Self {
        Self {
            ctx: core_mollusk().with_context(HashMap::new()),
        }
    }

    /// Stores a new system account holding `lamports` and returns its key.
    pub fn fund(&self, lamports: u64) -> Pubkey {
        let key = Pubkey::new_unique();
        self.store(key, payer_account(lamports));
        key
    }

    /// Stores `account` under `key`, replacing any previous value.
    pub fn store(&self, key: Pubkey, account: Account) {
        self.ctx.account_store.borrow_mut().insert(key, account);
    }

    /// The stored account for `key`; panics if there is none.
    pub fn account(&self, key: &Pubkey) -> Account {
        self.ctx
            .account_store
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or_else(|| panic!("account {key} is not in the fixture store"))
    }

    /// Whether `key` has a stored account.
    pub fn has_account(&self, key: &Pubkey) -> bool {
        self.ctx.account_store.borrow().contains_key(key)
    }

    /// Runs one instruction; the store is updated only on success.
    pub fn run(&self, ix: &Instruction) -> InstructionResult {
        self.ctx.process_instruction(ix)
    }

    /// Runs one instruction and asserts it succeeded.
    pub fn run_ok(&self, ix: &Instruction) -> InstructionResult {
        let result = self.run(ix);
        assert_ok(&result);
        result
    }

    /// Runs instructions in sequence, stopping at the first failure.
    pub fn run_chain(&self, ixs: &[Instruction]) -> InstructionResult {
        self.ctx.process_instruction_chain(ixs)
    }

    /// Creates an asset through `CreateV2` and returns its key.
    pub fn create_asset(&self, args: CreateAssetArgs) -> Pubkey {
        let payer = args.payer.unwrap_or_else(|| self.fund(ACCOUNT_LAMPORTS));
        let asset = Pubkey::new_unique();
        self.store(asset, empty_account());
        let ix = ix::create_v2(
            asset,
            args.collection,
            args.authority,
            payer,
            args.owner,
            args.update_authority,
            None,
            &args.name,
            &args.uri,
            non_empty(args.plugins),
            non_empty(args.adapters),
            &args.remaining,
        );
        self.run_ok(&ix);
        asset
    }

    /// Creates a collection through `CreateCollectionV2` and returns its key.
    pub fn create_collection(&self, args: CreateCollectionArgs) -> Pubkey {
        let payer = args.payer.unwrap_or_else(|| self.fund(ACCOUNT_LAMPORTS));
        let collection = Pubkey::new_unique();
        self.store(collection, empty_account());
        let ix = ix::create_collection_v2(
            collection,
            args.update_authority,
            payer,
            &args.name,
            &args.uri,
            non_empty(args.plugins),
            non_empty(args.adapters),
            &args.remaining,
        );
        self.run_ok(&ix);
        collection
    }

    /// The stored `AssetV1` for `key`.
    pub fn asset(&self, key: &Pubkey) -> AssetV1 {
        read_asset(&self.account(key))
    }

    /// The stored `CollectionV1` for `key`.
    pub fn collection(&self, key: &Pubkey) -> CollectionV1 {
        read_collection(&self.account(key))
    }

    /// The parsed plugin metadata of the stored asset or collection.
    pub fn parsed(&self, key: &Pubkey) -> ParsedAccount {
        parse_any(&self.account(key).data)
    }

    /// The stored lamports of `key`.
    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).lamports
    }

    /// Asserts the stored account for `key` was burned.
    pub fn assert_burned(&self, key: &Pubkey) {
        assert_account_burned(key, &self.account(key));
    }
}
