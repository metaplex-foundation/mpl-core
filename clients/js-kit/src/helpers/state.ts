import { type Address, address } from '@solana/addresses';
import { type AssetV1, type CollectionV1 } from '../generated';
import {
  type AssetPluginsList,
  type CollectionPluginsList,
} from '../plugins';

/**
 * Find the collection address for the given asset if it is part of a collection.
 * @param {AssetV1} asset Asset
 * @returns {Address | undefined} Collection address
 */
export function collectionAddress(
  asset: AssetV1 & { updateAuthority: { type: string; address?: Address } }
): Address | undefined {
  if (asset.updateAuthority.type === 'Collection') {
    return asset.updateAuthority.address;
  }

  return undefined;
}

/**
 * Check if the asset is frozen.
 * @param {AssetV1 & AssetPluginsList} asset Asset with plugins
 * @param {CollectionV1 & CollectionPluginsList | undefined} collection Collection with plugins
 * @returns {boolean} True if the asset is frozen
 */
export function isFrozen(
  asset: AssetV1 & AssetPluginsList,
  collection?: CollectionV1 & CollectionPluginsList
): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);
  return (
    (dAsset as AssetPluginsList).freezeDelegate?.frozen ||
    (dAsset as AssetPluginsList).permanentFreezeDelegate?.frozen ||
    false
  );
}

/**
 * Check if the given address is the owner of the asset.
 * @param {string | Address} addr Address
 * @param {AssetV1} asset Asset
 * @returns {boolean} True if the address is the owner
 */
export function isAssetOwner(addr: string | Address, asset: AssetV1): boolean {
  const key = address(addr);
  return key === asset.owner;
}

/**
 * Derive the asset plugins from the asset and collection. Plugins on the asset take precedence over plugins on the collection.
 * @param {AssetV1 & AssetPluginsList} asset Asset with plugins
 * @param {CollectionV1 & CollectionPluginsList | undefined} collection Collection with plugins
 * @returns {AssetV1 & AssetPluginsList} Asset with derived plugins
 */
export function deriveAssetPlugins<T extends AssetV1 & AssetPluginsList>(
  asset: T,
  collection?: CollectionV1 & CollectionPluginsList
): T {
  if (!collection) {
    return asset;
  }

  // Merge plugins from collection into asset, asset takes precedence
  const mergedPlugins: Partial<AssetPluginsList> = {};

  // List of common plugin keys that can be inherited from collection
  const inheritablePluginKeys: (keyof AssetPluginsList)[] = [
    'attributes',
    'royalties',
    'updateDelegate',
    'permanentFreezeDelegate',
    'permanentTransferDelegate',
    'permanentBurnDelegate',
    'addBlocker',
    'immutableMetadata',
    'autograph',
    'verifiedCreators',
    'freezeExecute',
    'permanentFreezeExecute',
  ];

  for (const key of inheritablePluginKeys) {
    const assetPlugin = (asset as AssetPluginsList)[key];
    const collectionPlugin = (collection as CollectionPluginsList)[
      key as keyof CollectionPluginsList
    ];

    if (assetPlugin) {
      (mergedPlugins as Record<string, unknown>)[key] = assetPlugin;
    } else if (collectionPlugin) {
      (mergedPlugins as Record<string, unknown>)[key] = collectionPlugin;
    }
  }

  return {
    ...asset,
    ...mergedPlugins,
  };
}
