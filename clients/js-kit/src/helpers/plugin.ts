import { type Address, address } from '@solana/addresses';
import { capitalizeFirstLetter, lowercaseFirstLetter } from '../utils';
import { type AssetV1, type CollectionV1, PluginType } from '../generated';
import { collectionAddress, deriveAssetPlugins, isAssetOwner } from './state';
import {
  hasAssetUpdateAuthority,
  hasPluginAddressAuthority,
} from './authority';
import { type AssetPluginsList, type UpdateAuthority } from '../plugins';

export type AssetPluginKey = keyof AssetPluginsList;

/**
 * Convert a plugin type to a key for the asset plugins.
 * @param {PluginType} pluginType Plugin type
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
  authority: Address | string;
  pluginTypes: PluginType[];
  asset: AssetV1 & AssetPluginsList & { updateAuthority: UpdateAuthority };
  collection?: CollectionV1 & AssetPluginsList;
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
  if (cAddress && collection && cAddress !== collection.updateAuthority) {
    throw new Error('Collection mismatch');
  }

  const dAsset = deriveAssetPlugins(asset, collection);
  const auth = address(authority);
  const isUpdateAuth = hasAssetUpdateAuthority(auth, asset, collection);
  const isOwner = isAssetOwner(auth, asset);
  return pluginTypes.map((type) => {
    const plugin = (dAsset as AssetPluginsList)[assetPluginKeyFromType(type)];
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
