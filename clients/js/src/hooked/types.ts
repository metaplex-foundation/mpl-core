import { PublicKey } from '@metaplex-foundation/umi';
import {
  Asset,
  Burn,
  Collection,
  Freeze,
  PluginHeader,
  Royalties,
  Transfer,
  UpdateDelegate,
} from '../generated';

export type BaseAuthority = {
  none?: boolean;
  owner?: boolean;
  update?: boolean;
  pubkey?: Array<PublicKey>;
  permanent?: Array<PublicKey>;
};

export type BasePlugin = {
  authority: BaseAuthority;
  offset?: bigint;
};

export type RoyaltiesPlugin = BasePlugin & Royalties;
export type FreezePlugin = BasePlugin & Freeze;
export type BurnPlugin = BasePlugin & Burn;
export type TransferPlugin = BasePlugin & Transfer;
export type UpdateDelegatePlugin = BasePlugin & UpdateDelegate;

export type PluginsList = {
  royalties?: RoyaltiesPlugin;
  freeze?: FreezePlugin;
  burn?: BurnPlugin;
  transfer?: TransferPlugin;
  updateDelegate?: UpdateDelegatePlugin;
};

export type AssetWithPlugins = Asset &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };

export type CollectionWithPlugins = Collection &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };
