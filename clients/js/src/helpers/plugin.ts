import { PublicKey, publicKey } from '@metaplex-foundation/umi';
import { capitalizeFirstLetter, lowercaseFirstLetter } from '../utils';
import { AssetV1, CollectionV1, PluginType } from '../generated';
import { collectionAddress, deriveAssetPlugins, isAssetOwner } from './state';
import {
  hasAssetUpdateAuthority,
  hasPluginAddressAuthority,
} from './authority';
import { AssetPluginsList } from '../plugins';

export type AssetPluginKey = keyof AssetPluginsList;

/**
 * Convert a plugin type to a key for the asset plugins.
 * @param {AssetV1} pluginType Asset
 * @returns {AssetPluginKey}
 */
export function assetPluginKeyFromType(pluginType: PluginType): AssetPluginKey {
  return lowercaseFirstLetter(PluginType[pluginType]) as AssetPluginKey;
}

/**
 * Convert a plugin key to a type.
 * @param {AssetPluginKey} key Asset plugin key
 * @returns {PluginType}
 */
export function pluginTypeFromAssetPluginKey(key: AssetPluginKey): PluginType {
  return PluginType[capitalizeFirstLetter(key) as keyof typeof PluginType];
}

export type CheckPluginAuthoritiesArgs = {
  authority: PublicKey | string;
  pluginTypes: PluginType[];
  asset: AssetV1;
  collection?: CollectionV1;
};
/**
 * Check the authority for the given plugin types on an asset.
 * @param {CheckPluginAuthoritiesArgs} args Arguments
 * @returns {boolean[]} Array of booleans indicating if the authority matches the plugin authority
 */

export function checkPluginAuthorities({
  authority,
  pluginTypes,
  asset,
  collection,
}: CheckPluginAuthoritiesArgs): boolean[] {
  const cAddress = collectionAddress(asset);
  if (cAddress && cAddress !== collection?.publicKey) {
    throw new Error('Collection mismatch');
  }

  const dAsset = deriveAssetPlugins(asset, collection);
  const auth = publicKey(authority);
  const isUpdateAuth = hasAssetUpdateAuthority(auth, asset, collection);
  const isOwner = isAssetOwner(auth, asset);
  return pluginTypes.map((type) => {
    const plugin = dAsset[assetPluginKeyFromType(type)];
    if (plugin) {
      if (
        hasPluginAddressAuthority(auth, plugin.authority) ||
        (plugin.authority.type === 'UpdateAuthority' && isUpdateAuth) ||
        (plugin.authority.type === 'Owner' && isOwner)
      ) {
        return true;
      }
    }
    return false;
  });
}
