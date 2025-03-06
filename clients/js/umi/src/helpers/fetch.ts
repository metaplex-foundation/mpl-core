import {
  Context,
  publicKey,
  PublicKey,
  RpcGetAccountOptions,
  Signer,
} from '@metaplex-foundation/umi';
import {
  AssetV1,
  CollectionV1,
  fetchAllAssetV1,
  fetchAllCollectionV1,
  fetchAssetV1,
  fetchCollectionV1,
  getAssetV1GpaBuilder,
  getCollectionV1GpaBuilder,
  Key,
} from '../generated';
import { updateAuthority } from '../plugins';
import { collectionAddress, deriveAssetPlugins } from './state';

/**
 * Helper function to derive plugins for a list of assets while fetching relevant collections.
 *
 * @param umi Context
 * @param assets Assets to derive plugins for
 * @returns Promise of a list of `AssetV1`
 */
export const deriveAssetPluginsWithFetch = async (
  umi: Context,
  assets: AssetV1[]
): Promise<AssetV1[]> => {
  const collectionKeys = Array.from(
    new Set(assets.map((asset) => collectionAddress(asset)))
  ).filter((collection): collection is PublicKey => !!collection);
  const collections = await fetchAllCollectionV1(umi, collectionKeys);

  const collectionMap = collections.reduce(
    (map, collection) => {
      map[collection.publicKey] = collection;
      return map;
    },
    {} as { [key: string]: CollectionV1 }
  );

  return assets.map((asset) => {
    const collection = collectionAddress(asset);
    if (!collection) {
      return asset;
    }
    return deriveAssetPlugins(asset, collectionMap[collection]);
  });
};

/**
 * Helper function to fetch assets by owner using GPA. For more filters, use the `getAssetV1GpaBuilder` directly.
 * For faster performance or more flexible queries, use DAS with `mpl-core-das` package.
 *
 * @param umi Context
 * @param owner Owner of the assets
 * @param options Options, `skipDerivePlugins` plugins from collection is false by default
 * @returns Promise of a list of `AssetV1`
 */
export const fetchAssetsByOwner = async (
  umi: Context,
  owner: PublicKey | Signer | string,
  options: { skipDerivePlugins?: boolean } = {}
): Promise<AssetV1[]> => {
  const assets = await getAssetV1GpaBuilder(umi)
    .whereField('key', Key.AssetV1)
    .whereField('owner', publicKey(owner))
    .getDeserialized();

  if (options.skipDerivePlugins) {
    return assets;
  }

  return deriveAssetPluginsWithFetch(umi, assets);
};

/**
 * Helper function to fetch assets by collection using GPA. For more filters, use the `getAssetV1GpaBuilder` directly.
 * For faster performance or more flexible queries, use DAS with `mpl-core-das` package.
 *
 * @param umi Context
 * @param collection Assets that belong to this collection
 * @param options Options, `skipDerivePlugins` plugins from collection is false by default
 * @returns Promise of a list of `AssetV1`
 */
export const fetchAssetsByCollection = async (
  umi: Context,
  collection: PublicKey | string,
  options: { skipDerivePlugins?: boolean } = {}
): Promise<AssetV1[]> => {
  const assets = await getAssetV1GpaBuilder(umi)
    .whereField('key', Key.AssetV1)
    .whereField(
      'updateAuthority',
      updateAuthority('Collection', [publicKey(collection)])
    )
    .getDeserialized();

  if (options.skipDerivePlugins) {
    return assets;
  }
  return deriveAssetPluginsWithFetch(umi, assets);
};

/**
 * Helper function to fetch collections by update authority using GPA. For more filters, use the `getCollectionV1GpaBuilder` directly.
 * For faster performance or more flexible queries, use DAS with `mpl-core-das` package.
 *
 * @param umi Context
 * @param authority Update authority of the collections
 * @returns
 */
export const fetchCollectionsByUpdateAuthority = async (
  umi: Context,
  authority: PublicKey | string
) =>
  getCollectionV1GpaBuilder(umi)
    .whereField('key', Key.CollectionV1)
    .whereField('updateAuthority', publicKey(authority))
    .getDeserialized();

/**
 * Helper function to fetch an asset and derive plugins from the collection if applicable.
 *
 * @param umi
 * @param asset
 * @param options
 */
export const fetchAsset = async (
  umi: Context,
  asset: PublicKey | string,
  options: { skipDerivePlugins?: boolean } & RpcGetAccountOptions = {}
): Promise<AssetV1> => {
  const assetV1 = await fetchAssetV1(umi, publicKey(asset));

  if (options.skipDerivePlugins) {
    return assetV1;
  }

  const collection = collectionAddress(assetV1);
  if (!collection) {
    return assetV1;
  }
  return deriveAssetPlugins(assetV1, await fetchCollectionV1(umi, collection));
};

/**
 * Helper function to fetch multiple assets and derive plugins from their collections if applicable.
 *
 * @param umi Context
 * @param assets Array of asset addresses to fetch
 * @param options Options, `skipDerivePlugins` plugins from collection is false by default; `chunksize` how many assets to fetch in a single rpc call.
 * @returns Promise of a list of `AssetV1`
 */
export const fetchAllAssets = async (
  umi: Context,
  assets: Array<PublicKey | string>,
  options: {
    skipDerivePlugins?: boolean;
    chunkSize?: number;
  } & RpcGetAccountOptions = {}
): Promise<AssetV1[]> => {
  const chunkSize = options.chunkSize ?? 1000;
  const assetChunks = [];
  for (let i = 0; i < assets.length; i += chunkSize) {
    assetChunks.push(assets.slice(i, i + chunkSize));
  }

  const assetV1s = (
    await Promise.all(
      assetChunks.map((chunk) =>
        fetchAllAssetV1(
          umi,
          chunk.map((asset) => publicKey(asset))
        )
      )
    )
  ).flat();

  if (options.skipDerivePlugins) {
    return assetV1s;
  }

  const collectionKeys = Array.from(
    new Set(assetV1s.map((asset) => collectionAddress(asset)))
  ).filter((collection): collection is PublicKey => !!collection);

  const collections = await fetchAllCollectionV1(umi, collectionKeys);
  const collectionMap = collections.reduce(
    (map, collection) => {
      map[collection.publicKey] = collection;
      return map;
    },
    {} as { [key: string]: CollectionV1 }
  );

  return assetV1s.map((assetV1) => {
    const collection = collectionAddress(assetV1);
    if (!collection) {
      return assetV1;
    }
    return deriveAssetPlugins(assetV1, collectionMap[collection]);
  });
};

/**
 * Helper function to fetch a collection.
 *
 * @param umi
 * @param collection
 */

export const fetchCollection = async (
  umi: Context,
  collection: PublicKey | string,
  options?: RpcGetAccountOptions
): Promise<CollectionV1> =>
  fetchCollectionV1(umi, publicKey(collection), options);
