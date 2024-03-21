import { PublicKey, publicKey } from '@metaplex-foundation/umi';
import { AssetV1, CollectionV1 } from '../generated';

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

  return {
    ...collection,
    ...asset,
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
