import { createDecoder, type Decoder } from '@solana/codecs';
import {
  Key,
  PluginHeaderV1,
  getPluginHeaderV1Decoder,
  CollectionV1,
  getCollectionV1Decoder,
} from '../generated';
import { getPluginRegistryV1AccountDataDecoder } from './pluginRegistryV1Data';
import { type CollectionPluginsList as PluginsCollectionPluginsList } from '../plugins/types';

// Re-export for internal use
type CollectionPluginsList = PluginsCollectionPluginsList;

export type CollectionV1AccountData = CollectionV1 &
  CollectionPluginsList & {
    pluginHeader?: PluginHeaderV1;
  };

export function getCollectionV1AccountDataDecoder(): Decoder<CollectionV1AccountData> {
  return createDecoder({
    read: (bytes, offset) => {
      const [collection, collectionOffset] = getCollectionV1Decoder().read(bytes, offset);
      if (collection.key !== Key.CollectionV1) {
        throw new Error(`Expected a Collection account, got key: ${collection.key}`);
      }

      let pluginHeader: PluginHeaderV1 | undefined;
      let finalOffset = collectionOffset;

      if (bytes.length !== collectionOffset) {
        [pluginHeader] = getPluginHeaderV1Decoder().read(bytes, collectionOffset);

        const [, registryOffset] = getPluginRegistryV1AccountDataDecoder().read(
          bytes,
          Number(pluginHeader.pluginRegistryOffset)
        );
        finalOffset = registryOffset;

        // TODO: Parse plugins when plugins module is ported
      }

      return [
        {
          pluginHeader,
          ...collection,
        },
        finalOffset,
      ];
    },
  });
}
