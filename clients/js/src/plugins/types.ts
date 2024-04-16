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
  BaseRoyalties,
  FreezeDelegateArgs,
  UpdateDelegateArgs,
  AttributesArgs,
  PermanentFreezeDelegateArgs,
  EditionArgs,
  BasePluginAuthority,
  BaseRoyaltiesArgs,
  BaseExternalPluginKey,
} from '../generated';
import { RoyaltiesArgs, RoyaltiesPlugin } from './royalties';
import { PluginAuthority } from './pluginAuthority';
import { OracleInitInfoArgs, OraclePlugin, OracleUpdateInfoArgs } from './oracle';
import { LifecycleChecksContainer } from './lifecycleChecks';
import { DataStoreInitInfoArgs, DataStorePlugin, DataStoreUpdateInfoArgs } from './dataStore';
import { LifecycleHookInitInfoArgs, LifecycleHookPlugin, LifecycleHookUpdateInfoArgs } from './lifecycleHook';

// for backwards compatibility
export {
  pluginAuthority,
  updateAuthority,
  ruleSet,
}

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
    };

export type PluginAuthorityPairHelperArgsV2 = CreatePluginArgsV2 & {
  authority?: PluginAuthority;
};

export type CreatePluginArgsV2 =
  | ({
      type: 'Royalties';
    } & RoyaltiesArgs)
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
      type: 'UpdateDelegate';
    } & UpdateDelegateArgs)
  | ({
      type: 'Attributes';
    } & AttributesArgs)
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

export type PluginsList = {
  // for backwards compatibility
  royalties?: RoyaltiesPlugin | (BasePlugin & BaseRoyalties);
  freezeDelegate?: FreezeDelegatePlugin;
  burnDelegate?: BurnDelegatePlugin;
  transferDelegate?: TransferDelegatePlugin;
  updateDelegate?: UpdateDelegatePlugin;
  attributes?: AttributesPlugin;
  permanentFreezeDelegate?: PermanentFreezeDelegatePlugin;
  permanentTransferDelegate?: PermanentTransferDelegatePlugin;
  permanentBurnDelegate?: PermanentBurnDelegatePlugin;
  edition?: EditionPlugin;
};