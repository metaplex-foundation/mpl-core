import { PublicKey } from '@metaplex-foundation/umi';
import {
  PluginAuthority,
  BurnDelegate,
  FreezeDelegate,
  PermanentFreezeDelegate,
  Royalties,
  TransferDelegate,
  UpdateDelegate,
  Attributes,
  PermanentTransferDelegate,
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
export type FreezeDelegatePlugin = BasePlugin & FreezeDelegate;
export type BurnDelegatePlugin = BasePlugin & BurnDelegate;
export type TransferDelegatePlugin = BasePlugin & TransferDelegate;
export type UpdateDelegatePlugin = BasePlugin & UpdateDelegate;
export type PermanentFreezeDelegatePlugin = BasePlugin &
  PermanentFreezeDelegate;
export type AttributesPlugin = BasePlugin & Attributes;
export type PermanentTransferDelegatePlugin = BasePlugin &
  PermanentTransferDelegate;

export type PluginsList = {
  royalties?: RoyaltiesPlugin;
  freezeDelegate?: FreezeDelegatePlugin;
  burnDelegate?: BurnDelegatePlugin;
  transferDelegate?: TransferDelegatePlugin;
  updateDelegate?: UpdateDelegatePlugin;
  permanentFreezeDelegate?: PermanentFreezeDelegatePlugin;
  attributes?: AttributesPlugin;
  permanentTransferDelegate?: PermanentTransferDelegatePlugin;
};
