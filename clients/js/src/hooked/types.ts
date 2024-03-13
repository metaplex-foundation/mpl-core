import { PublicKey } from '@metaplex-foundation/umi';
import {
  BaseAsset,
  Authority,
  Burn,
  BaseCollection,
  Freeze,
  PermanentFreeze,
  PluginHeader,
  Royalties,
  Transfer,
  UpdateDelegate,
  Attributes,
} from '../generated';

export type BaseAuthority = {
  type: PluginAuthorityType;
  address?: PublicKey;
};

export type PluginAuthorityType = Authority['__kind'];

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

export type PluginsList = {
  royalties?: RoyaltiesPlugin;
  freeze?: FreezePlugin;
  burn?: BurnPlugin;
  transfer?: TransferPlugin;
  updateDelegate?: UpdateDelegatePlugin;
  permanentFreeze?: PermanentFreezePlugin;
  attributes?: AttributesPlugin;
};

export type Asset = BaseAsset &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };

export type Collection = BaseCollection &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };
