import { Serializer } from '@metaplex-foundation/umi/serializers';

import {
  Key,
  PluginHeaderV1,
  PluginHeaderV1AccountData,
  getPluginHeaderV1AccountDataSerializer,
} from '../generated';
import {
  AssetV1AccountData as GenAssetV1AccountData,
  AssetV1AccountDataArgs as GenAssetV1AccountDataArgs,
  getAssetV1AccountDataSerializer as genGetAssetV1AccountDataSerializer,
} from '../generated/types/assetV1AccountData';
import { BaseUpdateAuthority, PluginsList } from '../types';
import { registryRecordsToPluginsList } from '../plugins';
import {
  PluginRegistryV1AccountData,
  getPluginRegistryV1AccountDataSerializer,
} from './pluginRegistryV1Data';

export type AssetV1AccountData = Omit<
  GenAssetV1AccountData,
  'updateAuthority'
> &
  PluginsList & {
    pluginHeader?: Omit<PluginHeaderV1, 'publicKey' | 'header'>;
    updateAuthority: BaseUpdateAuthority;
  };

export type AssetV1AccountDataArgs = Omit<
  GenAssetV1AccountDataArgs,
  'updateAuthority'
> &
  PluginsList & {
    pluginHeader?: Omit<PluginHeaderV1, 'publicKey' | 'header'>;
    updateAuthority: BaseUpdateAuthority;
  };

export const getAssetV1AccountDataSerializer = (): Serializer<
  AssetV1AccountDataArgs,
  AssetV1AccountData
> => ({
  description: 'AssetAccountData',
  fixedSize: null,
  maxSize: null,
  serialize: () => {
    throw new Error('Operation not supported.');
  },
  deserialize: (
    buffer: Uint8Array,
    offset = 0
  ): [AssetV1AccountData, number] => {
    // Account.
    const [asset, assetOffset] =
      genGetAssetV1AccountDataSerializer().deserialize(buffer, offset);
    if (asset.key !== Key.AssetV1) {
      throw new Error(`Expected an Asset account, got key: ${asset.key}`);
    }

    let pluginHeader: PluginHeaderV1AccountData | undefined;
    let pluginRegistry: PluginRegistryV1AccountData | undefined;
    let pluginsList: PluginsList | undefined;
    let finalOffset = assetOffset;

    if (buffer.length !== assetOffset) {
      [pluginHeader] = getPluginHeaderV1AccountDataSerializer().deserialize(
        buffer,
        assetOffset
      );

      [pluginRegistry, finalOffset] =
        getPluginRegistryV1AccountDataSerializer().deserialize(
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
