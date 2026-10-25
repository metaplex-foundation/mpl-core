import {
  AttributesArgs,
  BaseMasterEditionArgs,
  BasePluginAuthority,
  BaseRoyaltiesArgs,
  EditionArgs,
  FreezeDelegateArgs,
  FreezeExecuteArgs,
  GroupsArgs,
  PermanentFreezeDelegateArgs,
  PermanentFreezeExecuteArgs,
  UpdateDelegateArgs,
  basePluginAuthority as pluginAuthority,
  baseRuleSet as ruleSet,
  baseUpdateAuthority as updateAuthority,
} from '../generated';
import { PluginAuthority } from './pluginAuthority';

// The plugin type aliases, V2 argument unions, and plugin lists are generated
// from the IDL + program-derived plugin manifest.
export * from '../generated/plugins/internal';

// for backwards compatibility
export { pluginAuthority, ruleSet, updateAuthority };

export type BasePlugin = {
  authority: PluginAuthority;
  offset?: bigint;
};

export type PluginAuthorityPairHelperArgs = CreatePluginArgs & {
  authority?: BasePluginAuthority;
};

/**
 * @deprecated Use the V2 plugin argument unions (e.g. `AssetAllPluginArgsV2`)
 * with the `create`/`addPlugin` helpers instead.
 */
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
    }
  | {
      type: 'BubblegumV2';
    }
  | {
      type: 'FreezeExecute';
      data: FreezeExecuteArgs;
    }
  | {
      type: 'Groups';
      data: GroupsArgs;
    }
  | {
      type: 'PermanentFreezeExecute';
      data: PermanentFreezeExecuteArgs;
    };
