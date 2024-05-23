import { PublicKey, publicKey } from '@metaplex-foundation/umi';
import { AssetLinkedLifecycleHookPlugin } from 'src/plugins/assetLinkedLifecycleHook';
import { AssetV1, CollectionV1 } from '../generated';
import { ExternalPluginAdapters, ExternalPluginAdaptersList } from '../plugins';
import { OraclePlugin } from '../plugins/oracle';
import { SecureDataStorePlugin } from '../plugins/secureDataStore';
import { LifecycleHookPlugin } from '../plugins/lifecycleHook';
import { DataSectionPlugin } from '../plugins/dataSection';
import { AssetLinkedSecureDataStorePlugin } from '../plugins/assetLinkedSecureDataStore';

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
  'secureDataStores',
  'lifecycleHooks',
  'dataSections',
  'assetLinkedSecureDataStores',
];
export const getExternalPluginAdapterKeyAsString = (
  plugin:
    | Pick<OraclePlugin, 'type' | 'baseAddress'>
    | Pick<SecureDataStorePlugin, 'type' | 'dataAuthority'>
    | Pick<LifecycleHookPlugin, 'type' | 'hookedProgram'>
    | Pick<AssetLinkedSecureDataStorePlugin, 'type' | 'dataAuthority'>
    | Pick<AssetLinkedLifecycleHookPlugin, 'type' | 'hookedProgram'>
    | Pick<DataSectionPlugin, 'type' | 'parentKey'>
): string => {
  switch (plugin.type) {
    case 'Oracle':
      return `${plugin.type}-${plugin.baseAddress}`;
    case 'SecureDataStore':
      return `${plugin.type}-${plugin.dataAuthority.type}${
        plugin.dataAuthority.address ? `-${plugin.dataAuthority.address}` : ''
      }`;
    case 'LifecycleHook':
      return `${plugin.type}-${plugin.hookedProgram}`;
    case 'AssetLinkedSecureDataStore':
      return `${plugin.type}-${plugin.dataAuthority.type}${
        plugin.dataAuthority.address ? `-${plugin.dataAuthority.address}` : ''
      }`;
    case 'DataSection':
      return `${plugin.type}-${getExternalPluginAdapterKeyAsString(plugin.parentKey)}`;
    default:
      throw new Error('Unknown ExternalPluginAdapter type');
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
    asset[key]?.forEach((plugin: ExternalPluginAdapters) => {
      set.add(getExternalPluginAdapterKeyAsString(plugin));
      externalPluginAdapters[key]?.push(plugin as any);
    });

    collection[key]?.forEach((plugin: ExternalPluginAdapters) => {
      if (!set.has(getExternalPluginAdapterKeyAsString(plugin))) {
        externalPluginAdapters[key]?.push(plugin as any);
      }
    });
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
  const externalPluginAdapters = deriveExternalPluginAdapters(
    asset,
    collection
  );

  // TODO copy linked data section

  return {
    ...{
      ...collection,
      // the following plugins can only be on the collection
      masterEdition: undefined,
      assetLinkedSecureDataStores: undefined,
    },
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
