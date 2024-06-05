import {
  BurnDelegate,
  FreezeDelegate,
  PermanentFreezeDelegate,
  TransferDelegate,
  UpdateDelegate,
  Attributes,
  PermanentTransferDelegate,
  PermanentBurnDelegate,
  Edition,
  basePluginAuthority as pluginAuthority,
  baseUpdateAuthority as updateAuthority,
  baseRuleSet as ruleSet,
  FreezeDelegateArgs,
  UpdateDelegateArgs,
  AttributesArgs,
  PermanentFreezeDelegateArgs,
  EditionArgs,
  BasePluginAuthority,
  BaseRoyaltiesArgs,
  BaseMasterEditionArgs,
  AddBlocker,
  ImmutableMetadata,
  AutographArgs,
  VerifiedCreatorsArgs,
  Autograph,
  VerifiedCreators,
} from '../generated';
import { RoyaltiesArgs, RoyaltiesPlugin } from './royalties';
import { PluginAuthority } from './pluginAuthority';
import { MasterEdition, MasterEditionArgs } from './masterEdition';

// for backwards compatibility
export { pluginAuthority, updateAuthority, ruleSet };

export type BasePlugin = {
  authority: PluginAuthority;
  offset?: bigint;
};

export type PluginAuthorityPairHelperArgs = CreatePluginArgs & {
  authority?: BasePluginAuthority;
};

export type CreatePluginArgs =
  | {
      type: 'Royalties';
      data: BaseRoyaltiesArgs;
    }
  | {
      type: 'FreezeDelegate';
      data: FreezeDelegateArgs;
    }
  | {
      type: 'BurnDelegate';
    }
  | {
      type: 'TransferDelegate';
    }
  | {
      type: 'UpdateDelegate';
      data?: UpdateDelegateArgs;
    }
  | {
      type: 'Attributes';
      data: AttributesArgs;
    }
  | {
      type: 'PermanentFreezeDelegate';
      data: PermanentFreezeDelegateArgs;
    }
  | {
      type: 'PermanentTransferDelegate';
    }
  | {
      type: 'PermanentBurnDelegate';
    }
  | {
      type: 'Edition';
      data: EditionArgs;
    }
  | {
      type: 'MasterEdition';
      data: BaseMasterEditionArgs;
    }
  | {
      type: 'ImmutableMetadata';
    }
  | {
      type: 'AddBlocker';
    };

export type AuthorityArgsV2 = {
  authority?: PluginAuthority;
};

export type CreateOnlyPluginArgsV2 =
  | ({
      type: 'PermanentFreezeDelegate';
    } & PermanentFreezeDelegateArgs)
  | {
      type: 'PermanentTransferDelegate';
    }
  | {
      type: 'PermanentBurnDelegate';
    }
  | ({
      type: 'Edition';
    } & EditionArgs);

export type OwnerManagedPluginArgsV2 =
  | ({
      type: 'FreezeDelegate';
    } & FreezeDelegateArgs)
  | {
      type: 'BurnDelegate';
    }
  | {
      type: 'TransferDelegate';
    }
  | ({
      type: 'Autograph';
    } & AutographArgs);

export type AuthorityManagedPluginArgsV2 =
  | ({
      type: 'Royalties';
    } & RoyaltiesArgs)
  | ({
      type: 'UpdateDelegate';
    } & UpdateDelegateArgs)
  | ({
      type: 'Attributes';
    } & AttributesArgs)
  | ({
      type: 'MasterEdition';
    } & MasterEditionArgs)
  | {
      type: 'ImmutableMetadata';
    }
  | {
      type: 'AddBlocker';
    }
  | ({
      type: 'VerifiedCreators';
    } & VerifiedCreatorsArgs);

export type AssetAddablePluginArgsV2 =
  | OwnerManagedPluginArgsV2
  | AuthorityManagedPluginArgsV2;
export type AssetAllPluginArgsV2 =
  | AssetAddablePluginArgsV2
  | CreateOnlyPluginArgsV2;
export type AssetPluginAuthorityPairArgsV2 = AssetAllPluginArgsV2 &
  AuthorityArgsV2;
export type AssetAddablePluginAuthorityPairArgsV2 = AssetAddablePluginArgsV2 &
  AuthorityArgsV2;

export type CollectionAddablePluginArgsV2 = AuthorityManagedPluginArgsV2;
export type CollectionAllPluginArgsV2 =
  | CreateOnlyPluginArgsV2
  | CollectionAddablePluginArgsV2;
export type CollectionPluginAuthorityPairArgsV2 = CollectionAllPluginArgsV2 &
  AuthorityArgsV2;
export type CollectionAddablePluginAuthorityPairArgsV2 =
  CollectionAddablePluginArgsV2 & AuthorityArgsV2;

export type FreezeDelegatePlugin = BasePlugin & FreezeDelegate;
export type BurnDelegatePlugin = BasePlugin & BurnDelegate;
export type TransferDelegatePlugin = BasePlugin & TransferDelegate;
export type UpdateDelegatePlugin = BasePlugin & UpdateDelegate;
export type PermanentFreezeDelegatePlugin = BasePlugin &
  PermanentFreezeDelegate;
export type AttributesPlugin = BasePlugin & Attributes;
export type PermanentTransferDelegatePlugin = BasePlugin &
  PermanentTransferDelegate;
export type PermanentBurnDelegatePlugin = BasePlugin & PermanentBurnDelegate;
export type EditionPlugin = BasePlugin & Edition;
export type MasterEditionPlugin = BasePlugin & MasterEdition;
export type AddBlockerPlugin = BasePlugin & AddBlocker;
export type ImmutableMetadataPlugin = BasePlugin & ImmutableMetadata;
export type VerifiedCreatorsPlugin = BasePlugin & VerifiedCreators;
export type AutographPlugin = BasePlugin & Autograph;

export type CommonPluginsList = {
  attributes?: AttributesPlugin;
  royalties?: RoyaltiesPlugin;
  updateDelegate?: UpdateDelegatePlugin;
  permanentFreezeDelegate?: PermanentFreezeDelegatePlugin;
  permanentTransferDelegate?: PermanentTransferDelegatePlugin;
  permanentBurnDelegate?: PermanentBurnDelegatePlugin;
  addBlocker?: AddBlockerPlugin;
  immutableMetadata?: ImmutableMetadataPlugin;
  autograph?: AutographPlugin;
  verifiedCreators?: VerifiedCreatorsPlugin;
};

export type AssetPluginsList = {
  freezeDelegate?: FreezeDelegatePlugin;
  burnDelegate?: BurnDelegatePlugin;
  transferDelegate?: TransferDelegatePlugin;
  edition?: EditionPlugin;
} & CommonPluginsList;

export type CollectionPluginsList = {
  masterEdition?: MasterEditionPlugin;
} & CommonPluginsList;

export type PluginsList = AssetPluginsList & CollectionPluginsList;
