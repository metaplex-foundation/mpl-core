import { PublicKey } from '@metaplex-foundation/umi';
import { AssetV1, CollectionV1, PluginType } from '../generated';
import { deriveAssetPlugins, isFrozen } from './state';
import { checkPluginAuthorities } from './plugin';

export function canTransfer(authority: PublicKey | string, asset: AssetV1, collection?: CollectionV1): boolean {
  if (!isFrozen(asset, collection)) {
    const dAsset = deriveAssetPlugins(asset, collection);
    if (dAsset.owner === authority) {
      return true;
    }
    const transferDelegates = checkPluginAuthorities({
      authority,
      pluginTypes: [PluginType.TransferDelegate, PluginType.PermanentTransferDelegate],
      asset: dAsset,
      collection,
    });
    return transferDelegates.some((d) => d);
  }
  return false;
}

export function canBurn(authority: PublicKey | string, asset: AssetV1, collection?: CollectionV1): boolean {
  if (!isFrozen(asset, collection)) {
    const dAsset = deriveAssetPlugins(asset, collection);
    if (dAsset.owner === authority) {
      return true;
    }
    const burnDelegates = checkPluginAuthorities({
      authority,
      pluginTypes: [PluginType.BurnDelegate, PluginType.PermanentBurnDelegate],
      asset,
      collection,
    });
    return burnDelegates.some((d) => d);
  }
  return false;
}

export function canUpdate(authority: PublicKey | string, asset: AssetV1, collection?: CollectionV1): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);
  if (dAsset.updateAuthority.type === 'Address' && dAsset.updateAuthority.address === authority) {
    return true;
  }
  const updateDelegates = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.UpdateDelegate],
    asset: dAsset,
    collection,
  });
  return updateDelegates.some((d) => d);
}
