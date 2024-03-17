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
  CollectionAccountData as GenCollectionAccountData,
  CollectionAccountDataArgs as GenCollectionAccountDataArgs,
  getCollectionAccountDataSerializer as genGetCollectionAccountDataSerializer,
} from '../generated/types/collectionAccountData';
import { PluginsList } from '../types';
import { registryRecordsToPluginsList } from '../plugins';

export type CollectionAccountData = GenCollectionAccountData &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };

export type CollectionAccountDataArgs = Omit<
  GenCollectionAccountDataArgs,
  'updateAuthority'
> &
  PluginsList & {
    pluginHeader?: Omit<PluginHeader, 'publicKey' | 'header'>;
  };

export const getCollectionAccountDataSerializer = (): Serializer<
  CollectionAccountDataArgs,
  CollectionAccountData
> => ({
  description: 'CollectionAccountData',
  fixedSize: null,
  maxSize: null,
  serialize: () => {
    throw new Error('Operation not supported.');
  },
  deserialize: (
    buffer: Uint8Array,
    offset = 0
  ): [CollectionAccountData, number] => {
    // Account.
    const [collection, collectionOffset] =
      genGetCollectionAccountDataSerializer().deserialize(buffer, offset);
    if (collection.key !== Key.Collection) {
      throw new Error(
        `Expected an Collection account, got key: ${collection.key}`
      );
    }

    let pluginHeader: PluginHeaderAccountData | undefined;
    let pluginRegistry: PluginRegistryAccountData | undefined;
    let pluginsList: PluginsList | undefined;
    let finalOffset = collectionOffset;

    if (buffer.length !== collectionOffset) {
      [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
        buffer,
        collectionOffset
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

    return [
      {
        pluginHeader,
        ...pluginsList,
        ...collection,
      },
      finalOffset,
    ];
  },
});
