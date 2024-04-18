import { PublicKey, publicKey } from '@metaplex-foundation/umi';
import { AssetV1, CollectionV1 } from '../generated';
import { deriveAssetPlugins, isAssetOwner } from './state';
import { PluginAuthority } from '../plugins';

/**
 * Check if the given pubkey has the Address authority for the plugin.
 * @param {PublicKey | string} pubkey Pubkey
 * @param {PluginAuthority} authority Plugin authority
 * @returns {boolean} True if the pubkey has the authority
 */
export function hasPluginAddressAuthority(
  pubkey: PublicKey | string,
  authority: PluginAuthority
): boolean {
  return (
    authority.type === 'Address' && authority.address === publicKey(pubkey)
  );
}
/**
 * Check if the given pubkey has the Owner authority for the plugin.
 * @param {PublicKey | string} pubkey Pubkey
 * @param {PluginAuthority} authority Plugin authority
 * @param {AssetV1} asset Asset
 * @returns {boolean} True if the pubkey has the authority
 */
export function hasPluginOwnerAuthority(
  pubkey: PublicKey | string,
  authority: PluginAuthority,
  asset: AssetV1
): boolean {
  return authority.type === 'Owner' && isAssetOwner(pubkey, asset);
}
/**
 * Check if the given pubkey has the UpdateAuthority authority for the plugin.
 * @param {PublicKey | string} pubkey Pubkey
 * @param {PluginAuthority} authority Plugin authority
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function hasPluginUpdateAuthority(
  pubkey: PublicKey | string,
  authority: PluginAuthority,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  return (
    authority.type === 'UpdateAuthority' &&
    hasAssetUpdateAuthority(pubkey, asset, collection)
  );
}

/**
 * Check if the given pubkey has the update authority for the asset.
 * If the asset specifies a collection as the update authority, the collection's update authority is checked.
 * If there are update delegates, they are also checked
 * @param {string | PublicKey} pubkey Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey is the update authority
 */
export function hasAssetUpdateAuthority(
  pubkey: string | PublicKey,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  const key = publicKey(pubkey);
  const dAsset = deriveAssetPlugins(asset, collection);
  if (
    dAsset.updateAuthority.type === 'Collection' &&
    dAsset.updateAuthority.address !== collection?.publicKey
  ) {
    throw Error('Collection mismatch');
  }

  // check if pubkey matches asset update auth or collection or derived delegate plugin
  if (
    (dAsset.updateAuthority.type === 'Address' &&
      dAsset.updateAuthority.address === key) ||
    (dAsset.updateDelegate?.authority.type === 'Address' &&
      dAsset.updateDelegate?.authority.address === key) ||
    (dAsset.updateDelegate?.authority.type === 'Owner' &&
      dAsset.owner === key) ||
    (dAsset.updateAuthority.type === 'Collection' &&
      collection?.updateAuthority === key)
  ) {
    return true;
  }

  return false;
}
/**
 * CHeck if the given pubkey has update authority for the collection.
 * @param {string | PublicKey} pubkey Pubkey
 * @param {CollectionV1} collection Collection
 * @returns {boolean} True if the pubkey is the update authority
 */

export function hasCollectionUpdateAuthority(
  pubkey: string | PublicKey,
  collection: CollectionV1
): boolean {
  const key = publicKey(pubkey);
  if (
    collection.updateAuthority === key ||
    (collection.updateDelegate?.authority.type === 'Address' &&
      collection.updateDelegate?.authority.address === key)
  ) {
    return true;
  }
  return false;
}
