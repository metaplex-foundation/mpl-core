mod external;
mod external_plugin_adapters;
mod internal;
mod lifecycle;
mod plugin_header;
mod plugin_registry;
mod utils;

pub use external::*;
pub use external_plugin_adapters::*;
pub use internal::*;
pub use lifecycle::*;
pub use plugin_header::*;
pub use plugin_registry::*;
pub use utils::*;

use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::ToPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use strum::{EnumCount, EnumIter};

use crate::{
    error::MplCoreError,
    state::{Authority, Compressible, DataBlob},
};

/// Definition of the plugin variants, each containing a link to the plugin struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Plugin {
    /// Royalties plugin.
    Royalties(Royalties),
    /// Freeze Delegate plugin.
    FreezeDelegate(FreezeDelegate),
    /// Burn Delegate plugin.
    BurnDelegate(BurnDelegate),
    /// Transfer Delegate plugin.
    TransferDelegate(TransferDelegate),
    /// Update Delegate plugin.
    UpdateDelegate(UpdateDelegate),
    /// Permanent Freeze Delegate authority which allows the creator to freeze
    PermanentFreezeDelegate(PermanentFreezeDelegate),
    /// Attributes plugin for arbitrary Key-Value pairs.
    Attributes(Attributes),
    /// Permanent Transfer Delegate authority which allows the creator of an asset to become the person who can transfer an Asset
    PermanentTransferDelegate(PermanentTransferDelegate),
    /// Permanent Burn Delegate authority allows the creator of an asset to become the person who can burn an Asset
    PermanentBurnDelegate(PermanentBurnDelegate),
    /// Edition plugin allows creators to add an edition number to the asset
    Edition(Edition),
    /// Master Edition plugin allows creators to specify the max supply and master edition details
    MasterEdition(MasterEdition),
    /// AddBlocker plugin. Prevents plugins from being added.
    AddBlocker(AddBlocker),
    /// ImmutableMetadata plugin. Makes metadata of the asset immutable.
    ImmutableMetadata(ImmutableMetadata),
    /// VerifiedCreators plugin allows update auth to specify verified creators and additional creators to sign
    VerifiedCreators(VerifiedCreators),
    /// Autograph plugin allows anybody to add their signature to the asset with an optional message
    Autograph(Autograph),
    /// The Bubblegum V2 plugin allows a Core collection to contain Compressed NFTs (cNFTs) from the Bubblegum program.
    BubblegumV2(BubblegumV2),
    /// Freeze Execute plugin.
    FreezeExecute(FreezeExecute),
}

impl Plugin {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn manager(&self) -> Authority {
        PluginType::from(self).manager()
    }

    /// Load and deserialize a plugin from an offset in the account.
    pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::DeserializationError.into()
        })
    }

    /// Save and serialize a plugin to an offset in the account.
    pub fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::SerializationError.into()
        })
    }

    /// Extract the inner plugin value from the plugin.
    pub(crate) fn inner(&self) -> &dyn PluginValidation {
        match &self {
            Plugin::Royalties(inner) => inner,
            Plugin::FreezeDelegate(inner) => inner,
            Plugin::BurnDelegate(inner) => inner,
            Plugin::TransferDelegate(inner) => inner,
            Plugin::UpdateDelegate(inner) => inner,
            Plugin::PermanentFreezeDelegate(inner) => inner,
            Plugin::Attributes(inner) => inner,
            Plugin::PermanentTransferDelegate(inner) => inner,
            Plugin::PermanentBurnDelegate(inner) => inner,
            Plugin::Edition(inner) => inner,
            Plugin::MasterEdition(inner) => inner,
            Plugin::AddBlocker(inner) => inner,
            Plugin::ImmutableMetadata(inner) => inner,
            Plugin::VerifiedCreators(inner) => inner,
            Plugin::Autograph(inner) => inner,
            Plugin::BubblegumV2(inner) => inner,
            Plugin::FreezeExecute(inner) => inner,
        }
    }
}

impl Compressible for Plugin {}

impl DataBlob for Plugin {
    fn len(&self) -> usize {
        1 // The discriminator
            + match self {
                Plugin::Royalties(royalties) => royalties.len(),
                Plugin::FreezeDelegate(freeze_delegate) => freeze_delegate.len(),
                Plugin::BurnDelegate(burn_delegate) => burn_delegate.len(),
                Plugin::TransferDelegate(transfer_delegate) => transfer_delegate.len(),
                Plugin::UpdateDelegate(update_delegate) => update_delegate.len(),
                Plugin::PermanentFreezeDelegate(permanent_freeze_delegate) => {
                    permanent_freeze_delegate.len()
                }
                Plugin::Attributes(attributes) => attributes.len(),
                Plugin::PermanentTransferDelegate(permanent_transfer_delegate) => {
                    permanent_transfer_delegate.len()
                }
                Plugin::PermanentBurnDelegate(permanent_burn_delegate) => {
                    permanent_burn_delegate.len()
                }
                Plugin::Edition(edition) => edition.len(),
                Plugin::MasterEdition(master_edition) => master_edition.len(),
                Plugin::AddBlocker(add_blocker) => add_blocker.len(),
                Plugin::ImmutableMetadata(immutable_metadata) => immutable_metadata.len(),
                Plugin::VerifiedCreators(verified_creators) => verified_creators.len(),
                Plugin::Autograph(autograph) => autograph.len(),
                Plugin::BubblegumV2(bubblegum_v2) => bubblegum_v2.len(),
                Plugin::FreezeExecute(freeze_execute) => freeze_execute.len(),
            }
    }
}

/// List of first party plugin types.
#[repr(C)]
#[derive(
    Clone,
    Copy,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Eq,
    Hash,
    PartialEq,
    ToPrimitive,
    EnumCount,
    PartialOrd,
    Ord,
    EnumIter,
)]
pub enum PluginType {
    /// Royalties plugin.
    Royalties,
    /// Freeze Delegate plugin.
    FreezeDelegate,
    /// Burn Delegate plugin.
    BurnDelegate,
    /// Transfer Delegate plugin.
    TransferDelegate,
    /// Update Delegate plugin.
    UpdateDelegate,
    /// The Permanent Freeze Delegate plugin.
    PermanentFreezeDelegate,
    /// The Attributes plugin.
    Attributes,
    /// The Permanent Transfer Delegate plugin.
    PermanentTransferDelegate,
    /// The Permanent Burn Delegate plugin.
    PermanentBurnDelegate,
    /// The Edition plugin.
    Edition,
    /// The Master Edition plugin.
    MasterEdition,
    /// AddBlocker plugin.
    AddBlocker,
    /// ImmutableMetadata plugin.
    ImmutableMetadata,
    /// VerifiedCreators plugin.
    VerifiedCreators,
    /// Autograph plugin.
    Autograph,
    /// Bubblegum V2 plugin.
    BubblegumV2,
    /// Freeze Execute plugin.
    FreezeExecute,
}

impl PluginType {
    /// A u8 enum discriminator.
    const BASE_LEN: usize = 1;
}

/// The list of permanent delegate types.
pub const PERMANENT_DELEGATES: [PluginType; 3] = [
    PluginType::PermanentFreezeDelegate,
    PluginType::PermanentTransferDelegate,
    PluginType::PermanentBurnDelegate,
];

impl DataBlob for PluginType {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl From<&Plugin> for PluginType {
    fn from(plugin: &Plugin) -> Self {
        match plugin {
            Plugin::AddBlocker(_) => PluginType::AddBlocker,
            Plugin::ImmutableMetadata(_) => PluginType::ImmutableMetadata,
            Plugin::Royalties(_) => PluginType::Royalties,
            Plugin::FreezeDelegate(_) => PluginType::FreezeDelegate,
            Plugin::BurnDelegate(_) => PluginType::BurnDelegate,
            Plugin::TransferDelegate(_) => PluginType::TransferDelegate,
            Plugin::UpdateDelegate(_) => PluginType::UpdateDelegate,
            Plugin::PermanentFreezeDelegate(_) => PluginType::PermanentFreezeDelegate,
            Plugin::Attributes(_) => PluginType::Attributes,
            Plugin::PermanentTransferDelegate(_) => PluginType::PermanentTransferDelegate,
            Plugin::PermanentBurnDelegate(_) => PluginType::PermanentBurnDelegate,
            Plugin::Edition(_) => PluginType::Edition,
            Plugin::MasterEdition(_) => PluginType::MasterEdition,
            Plugin::VerifiedCreators(_) => PluginType::VerifiedCreators,
            Plugin::Autograph(_) => PluginType::Autograph,
            Plugin::BubblegumV2(_) => PluginType::BubblegumV2,
            Plugin::FreezeExecute(_) => PluginType::FreezeExecute,
        }
    }
}

impl PluginType {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn manager(&self) -> Authority {
        match self {
            PluginType::AddBlocker => Authority::UpdateAuthority,
            PluginType::ImmutableMetadata => Authority::UpdateAuthority,
            PluginType::Royalties => Authority::UpdateAuthority,
            PluginType::FreezeDelegate => Authority::Owner,
            PluginType::BurnDelegate => Authority::Owner,
            PluginType::TransferDelegate => Authority::Owner,
            PluginType::UpdateDelegate => Authority::UpdateAuthority,
            PluginType::PermanentFreezeDelegate => Authority::UpdateAuthority,
            PluginType::Attributes => Authority::UpdateAuthority,
            PluginType::PermanentTransferDelegate => Authority::UpdateAuthority,
            PluginType::PermanentBurnDelegate => Authority::UpdateAuthority,
            PluginType::Edition => Authority::UpdateAuthority,
            PluginType::MasterEdition => Authority::UpdateAuthority,
            PluginType::VerifiedCreators => Authority::UpdateAuthority,
            PluginType::Autograph => Authority::Owner,
            PluginType::BubblegumV2 => Authority::Address {
                address: mpl_bubblegum::ID,
            },
            PluginType::FreezeExecute => Authority::Owner,
        }
    }
}

/// A pair of a plugin type and an optional authority.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct PluginAuthorityPair {
    pub(crate) plugin: Plugin,
    pub(crate) authority: Option<Authority>,
}

#[cfg(test)]
mod test {
    use solana_program::pubkey::Pubkey;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_plugin_empty_size() {
        //TODO: Implement Default for all plugins in a separate PR.
        let plugins = vec![
            Plugin::Royalties(Royalties {
                basis_points: 0,
                creators: vec![],
                rule_set: RuleSet::None,
            }),
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Plugin::BurnDelegate(BurnDelegate {}),
            Plugin::TransferDelegate(TransferDelegate {}),
            Plugin::UpdateDelegate(UpdateDelegate {
                additional_delegates: vec![],
            }),
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
            Plugin::Attributes(Attributes {
                attribute_list: vec![],
            }),
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Plugin::Edition(Edition { number: 0 }),
            Plugin::MasterEdition(MasterEdition {
                max_supply: None,
                name: None,
                uri: None,
            }),
            Plugin::AddBlocker(AddBlocker {}),
            Plugin::ImmutableMetadata(ImmutableMetadata {}),
            Plugin::VerifiedCreators(VerifiedCreators { signatures: vec![] }),
            Plugin::Autograph(Autograph { signatures: vec![] }),
            Plugin::BubblegumV2(BubblegumV2 {}),
            Plugin::FreezeExecute(FreezeExecute { frozen: false }),
        ];

        assert_eq!(
            plugins.len(),
            PluginType::COUNT,
            "All plugins should be tested"
        );

        for fixture in plugins {
            let serialized = fixture.try_to_vec().unwrap();
            assert_eq!(
                serialized.len(),
                fixture.len(),
                "Serialized {:?} should match size returned by len()",
                fixture
            );
        }
    }

    #[test]
    fn test_plugin_different_size() {
        //TODO: Implement Default for all plugins in a separate PR.
        let plugins: Vec<Vec<Plugin>> = vec![
            vec![
                Plugin::Royalties(Royalties {
                    basis_points: 0,
                    creators: vec![],
                    rule_set: RuleSet::None,
                }),
                Plugin::Royalties(Royalties {
                    basis_points: 1,
                    creators: vec![Creator {
                        address: Pubkey::default(),
                        percentage: 1,
                    }],
                    rule_set: RuleSet::ProgramAllowList(vec![]),
                }),
                Plugin::Royalties(Royalties {
                    basis_points: 2,
                    creators: vec![
                        Creator {
                            address: Pubkey::default(),
                            percentage: 2,
                        },
                        Creator {
                            address: Pubkey::default(),
                            percentage: 3,
                        },
                    ],
                    rule_set: RuleSet::ProgramDenyList(vec![Pubkey::default()]),
                }),
                Plugin::Royalties(Royalties {
                    basis_points: 3,
                    creators: vec![
                        Creator {
                            address: Pubkey::default(),
                            percentage: 3,
                        },
                        Creator {
                            address: Pubkey::default(),
                            percentage: 4,
                        },
                        Creator {
                            address: Pubkey::default(),
                            percentage: 5,
                        },
                    ],
                    rule_set: RuleSet::ProgramDenyList(vec![Pubkey::default(), Pubkey::default()]),
                }),
            ],
            vec![Plugin::FreezeDelegate(FreezeDelegate { frozen: true })],
            vec![Plugin::BurnDelegate(BurnDelegate {})],
            vec![Plugin::TransferDelegate(TransferDelegate {})],
            vec![Plugin::UpdateDelegate(UpdateDelegate {
                additional_delegates: vec![Pubkey::default(), Pubkey::default()],
            })],
            vec![Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                frozen: true,
            })],
            vec![Plugin::Attributes(Attributes {
                attribute_list: vec![
                    Attribute {
                        key: "test".to_string(),
                        value: "test".to_string(),
                    },
                    Attribute {
                        key: "test2".to_string(),
                        value: "test2".to_string(),
                    },
                ],
            })],
            vec![Plugin::PermanentTransferDelegate(
                PermanentTransferDelegate {},
            )],
            vec![Plugin::PermanentBurnDelegate(PermanentBurnDelegate {})],
            vec![Plugin::Edition(Edition { number: 1 })],
            vec![Plugin::MasterEdition(MasterEdition {
                max_supply: Some(1),
                name: Some("test".to_string()),
                uri: Some("test".to_string()),
            })],
            vec![Plugin::AddBlocker(AddBlocker {})],
            vec![Plugin::ImmutableMetadata(ImmutableMetadata {})],
            vec![Plugin::VerifiedCreators(VerifiedCreators {
                signatures: vec![VerifiedCreatorsSignature {
                    address: Pubkey::default(),
                    verified: true,
                }],
            })],
            vec![Plugin::Autograph(Autograph {
                signatures: vec![AutographSignature {
                    address: Pubkey::default(),
                    message: "test".to_string(),
                }],
            })],
            vec![Plugin::BubblegumV2(BubblegumV2 {})],
            vec![Plugin::FreezeExecute(FreezeExecute { frozen: true })],
        ];

        assert_eq!(
            plugins.len(),
            PluginType::COUNT,
            "All plugins should be tested"
        );

        for fixtures in plugins {
            for fixture in fixtures {
                let serialized = fixture.try_to_vec().unwrap();
                assert_eq!(
                    serialized.len(),
                    fixture.len(),
                    "Serialized {:?} should match size returned by len()",
                    fixture
                );
            }
        }
    }

    #[test]
    fn test_plugin_type_size() {
        for fixture in PluginType::iter() {
            let serialized = fixture.try_to_vec().unwrap();
            assert_eq!(
                serialized.len(),
                fixture.len(),
                "Serialized {:?} should match size returned by len()",
                fixture
            );
        }
    }
}
