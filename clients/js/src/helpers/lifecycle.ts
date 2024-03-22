import { PublicKey } from '@metaplex-foundation/umi';
import { AssetV1, CollectionV1, PluginType } from '../generated';
import { deriveAssetPlugins, isFrozen } from './state';
import { checkPluginAuthorities } from './plugin';
import { hasAssetUpdateAuthority } from './authority';

/**
 * Check if the given authority is eligible to transfer the asset.
 * This does NOT check if the asset's roylaty rule sets.
 * @param {PublicKey | string} authority Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function canTransfer(
  authority: PublicKey | string,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);
  const permaTransferDelegate = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.PermanentTransferDelegate],
    asset: dAsset,
    collection,
  });
  if (permaTransferDelegate.some((d) => d)) {
    return true;
  }

  if (!isFrozen(asset, collection)) {
    if (dAsset.owner === authority) {
      return true;
    }
    const transferDelegates = checkPluginAuthorities({
      authority,
      pluginTypes: [PluginType.TransferDelegate],
      asset: dAsset,
      collection,
    });
    return transferDelegates.some((d) => d);
  }
  return false;
}

/**
 * Check if the given pubkey is eligible to burn the asset.
 * @param {PublicKey | string} authority Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function canBurn(
  authority: PublicKey | string,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);
  const permaBurnDelegate = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.PermanentBurnDelegate],
    asset: dAsset,
    collection,
  });
  if (permaBurnDelegate.some((d) => d)) {
    return true;
  }

  if (!isFrozen(asset, collection)) {
    if (dAsset.owner === authority) {
      return true;
    }
    const burnDelegates = checkPluginAuthorities({
      authority,
      pluginTypes: [PluginType.BurnDelegate],
      asset,
      collection,
    });
    return burnDelegates.some((d) => d);
  }
  return false;
}

/**
 * Check if the given pubkey is eligible to update the asset.
 * @param {PublicKey | string} authority Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function canUpdate(
  authority: PublicKey | string,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  return hasAssetUpdateAuthority(authority, asset, collection);
}
