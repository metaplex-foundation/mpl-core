import { Serializer } from '@metaplex-foundation/umi/serializers';

import {
  Key,
  PluginHeader,
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
} from '../generated';
import {
  AssetAccountData as GenAssetAccountData,
  AssetAccountDataArgs as GenAssetAccountDataArgs,
  getAssetAccountDataSerializer as genGetAssetAccountDataSerializer,
} from '../generated/types/assetAccountData';
import { BaseAuthority, PluginsList } from '../types';
import { registryRecordsToPluginsList } from '../plugins';

export type AssetAccountData = Omit<GenAssetAccountData, 'updateAuthority'> &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
    updateAuthority: BaseAuthority;
  };

export type AssetAccountDataArgs = Omit<
  GenAssetAccountDataArgs,
  'updateAuthority'
> &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
    updateAuthority: BaseAuthority;
  };

export const getAssetAccountDataSerializer = (): Serializer<
  AssetAccountDataArgs,
  AssetAccountData
> => ({
  description: 'AssetAccountData',
  fixedSize: null,
  maxSize: null,
  serialize: () => {
    throw new Error('Operation not supported.');
  },
  deserialize: (buffer: Uint8Array, offset = 0): [AssetAccountData, number] => {
    // Account.
    const [asset, assetOffset] = genGetAssetAccountDataSerializer().deserialize(
      buffer,
      offset
    );
    if (asset.key !== Key.Asset) {
      throw new Error(`Expected an Asset account, got key: ${asset.key}`);
    }

    let pluginHeader: PluginHeaderAccountData | undefined;
    let pluginRegistry: PluginRegistryAccountData | undefined;
    let pluginsList: PluginsList | undefined;
    let finalOffset = assetOffset;

    if (buffer.length !== assetOffset) {
      [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
        buffer,
        assetOffset
      );

      [pluginRegistry, finalOffset] =
        getPluginRegistryAccountDataSerializer().deserialize(
          buffer,
          Number(pluginHeader.pluginRegistryOffset)
        );

      pluginsList = registryRecordsToPluginsList(
        pluginRegistry.registry,
        buffer
      );
    }
    const updateAuth = {
      type: asset.updateAuthority.__kind,
      address:
        asset.updateAuthority.__kind === 'None'
          ? undefined
          : asset.updateAuthority.fields[0],
    };

    return [
      {
        pluginHeader,
        ...pluginsList,
        ...asset,
        updateAuthority: updateAuth,
      },
      finalOffset,
    ];
  },
});
