import { PublicKey } from '@metaplex-foundation/umi';
import {
  PluginAuthority,
  Burn,
  Freeze,
  PermanentFreeze,
  Royalties,
  Transfer,
  UpdateDelegate,
  Attributes,
  PermanentTransfer,
  UpdateAuthority,
} from './generated';

export type BaseAuthority = {
  type: PluginAuthorityType | UpdateAuthorityType;
  address?: PublicKey;
};

export type UpdateAuthorityType = UpdateAuthority['__kind'];
export type PluginAuthorityType = PluginAuthority['__kind'];

export type BasePlugin = {
  authority: BaseAuthority;
  offset?: bigint;
};

export type RoyaltiesPlugin = BasePlugin & Royalties;
export type FreezePlugin = BasePlugin & Freeze;
export type BurnPlugin = BasePlugin & Burn;
export type TransferPlugin = BasePlugin & Transfer;
export type UpdateDelegatePlugin = BasePlugin & UpdateDelegate;
export type PermanentFreezePlugin = BasePlugin & PermanentFreeze;
export type AttributesPlugin = BasePlugin & Attributes;
export type PermanentTransferPlugin = BasePlugin & PermanentTransfer;

export type PluginsList = {
  royalties?: RoyaltiesPlugin;
  freeze?: FreezePlugin;
  burn?: BurnPlugin;
  transfer?: TransferPlugin;
  updateDelegate?: UpdateDelegatePlugin;
  permanentFreeze?: PermanentFreezePlugin;
  attributes?: AttributesPlugin;
  permanentTransfer?: PermanentTransferPlugin;
};
