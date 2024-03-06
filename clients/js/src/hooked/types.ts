import { PublicKey } from '@metaplex-foundation/umi';
import {
  Asset,
  Burn,
  Freeze,
  PluginHeader,
  Royalties,
  Transfer,
  UpdateDelegate,
} from '../generated';

export type BaseAuthorities = {
  none?: boolean;
  owner?: boolean;
  update?: boolean;
  pubkey?: Array<PublicKey>;
  permanent?: Array<PublicKey>;
};

export type BasePlugin = {
  authorities: BaseAuthorities;
};

export type ReservedPlugin = BasePlugin;
export type RoyaltiesPlugin = BasePlugin & Royalties;
export type FreezePlugin = BasePlugin & Freeze;
export type BurnPlugin = BasePlugin & Burn;
export type TransferPlugin = BasePlugin & Transfer;
export type UpdateDelegatePlugin = BasePlugin & UpdateDelegate;

export type PluginsList = {
  reserved?: ReservedPlugin;
  royalties?: RoyaltiesPlugin;
  freeze?: FreezePlugin;
  burn?: BurnPlugin;
  transfer?: TransferPlugin;
  updateDelegate?: UpdateDelegatePlugin;
};

export type AssetWithPluginsTest = Asset &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };
