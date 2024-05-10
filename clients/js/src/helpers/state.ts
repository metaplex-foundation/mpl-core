import { PublicKey, publicKey } from '@metaplex-foundation/umi';
import { AssetV1, CollectionV1 } from '../generated';
import { ExternalPluginAdaptersList } from '../plugins';
import { OracleInitInfoArgs, OraclePlugin } from '../plugins/oracle';
import { DataStoreInitInfoArgs, DataStorePlugin } from '../plugins/dataStore';
import {
  LifecycleHookInitInfoArgs,
  LifecycleHookPlugin,
} from '../plugins/lifecycleHook';

/**
 * Find the collection address for the given asset if it is part of a collection.
 * @param {AssetV1} asset Asset
 * @returns {PublicKey | undefined} Collection address
 */
export function collectionAddress(asset: AssetV1): PublicKey | undefined {
  if (asset.updateAuthority.type === 'Collection') {
    return asset.updateAuthority.address;
  }

  return undefined;
}

const externalPluginAdapterKeys: (keyof ExternalPluginAdaptersList)[] = [
  'oracles',
  'dataStores',
  'lifecycleHooks',
];
export const getExternalPluginAdapterKeyAsString = (
  plugin:
    | OraclePlugin
    | DataStorePlugin
    | LifecycleHookPlugin
    | OracleInitInfoArgs
    | LifecycleHookInitInfoArgs
    | DataStoreInitInfoArgs
) => {
  switch (plugin.type) {
    case 'Oracle':
      return `${plugin.type}-${plugin.baseAddress}`;
    case 'DataStore':
      return `${plugin.type}-${plugin.dataAuthority.type}${
        plugin.dataAuthority.address ? `-${plugin.dataAuthority.address}` : ''
      }`;
    case 'LifecycleHook':
    default:
      return `${plugin.type}-${plugin.hookedProgram}`;
  }
};

export const deriveExternalPluginAdapters = (
  asset: ExternalPluginAdaptersList,
  collection?: ExternalPluginAdaptersList
) => {
  if (!collection) {
    return asset;
  }
  const externalPluginAdapters: ExternalPluginAdaptersList = {};
  externalPluginAdapterKeys.forEach((key) => {
    const set = new Set();
    if (asset[key] || collection[key]) {
      externalPluginAdapters[key] = [];
    }
    asset[key]?.forEach(
      (plugin: OraclePlugin | DataStorePlugin | LifecycleHookPlugin) => {
        set.add(getExternalPluginAdapterKeyAsString(plugin));
        externalPluginAdapters[key]?.push(plugin as any);
      }
    );

    collection[key]?.forEach(
      (plugin: OraclePlugin | DataStorePlugin | LifecycleHookPlugin) => {
        if (!set.has(getExternalPluginAdapterKeyAsString(plugin))) {
          externalPluginAdapters[key]?.push(plugin as any);
        }
      }
    );
  });

  return externalPluginAdapters;
};
/**
 * Derive the asset plugins from the asset and collection. Plugins on the asset take precedence over plugins on the collection.
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {AssetV1} Asset with plugins
 */
export function deriveAssetPlugins(
  asset: AssetV1,
  collection?: CollectionV1
): AssetV1 {
  if (!collection) {
    return asset;
  }
  const externalPluginAdapters = deriveExternalPluginAdapters(asset, collection);

  return {
    ...collection,
    ...asset,
    ...externalPluginAdapters,
  };
}

/**
 * Check if the asset is frozen.
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the asset is frozen
 */
export function isFrozen(asset: AssetV1, collection?: CollectionV1): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);
  return (
    dAsset.freezeDelegate?.frozen ||
    dAsset.permanentFreezeDelegate?.frozen ||
    false
  );
}

/**
 * Check if the given pubkey is the owner of the asset.
 * @param {string | PublicKey} pubkey Pubkey
 * @param {AssetV1} asset Asset
 * @returns {boolean} True if the pubkey is the owner
 */
export function isAssetOwner(
  pubkey: string | PublicKey,
  asset: AssetV1
): boolean {
  const key = publicKey(pubkey);
  return key === asset.owner;
}
