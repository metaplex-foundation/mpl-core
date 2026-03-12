import { createDecoder, type Decoder } from '@solana/codecs';
import {
  Key,
  PluginHeaderV1,
  getPluginHeaderV1Decoder,
  AssetV1,
  getAssetV1Decoder,
} from '../generated';
import { getPluginRegistryV1AccountDataDecoder } from './pluginRegistryV1Data';

// Import from plugins module
import { type AssetPluginsList as PluginsAssetPluginsList, type UpdateAuthority } from '../plugins';

// Re-export for internal use
type AssetPluginsList = PluginsAssetPluginsList;

export type AssetV1AccountData = Omit<AssetV1, 'updateAuthority'> &
  AssetPluginsList & {
    pluginHeader?: PluginHeaderV1;
    updateAuthority: UpdateAuthority;
  };

export function getAssetV1AccountDataDecoder(): Decoder<AssetV1AccountData> {
  return createDecoder({
    read: (bytes, offset) => {
      const [asset, assetOffset] = getAssetV1Decoder().read(bytes, offset);
      if (asset.key !== Key.AssetV1) {
        throw new Error(`Expected an Asset account, got key: ${asset.key}`);
      }

      let pluginHeader: PluginHeaderV1 | undefined;
      let finalOffset = assetOffset;

      if (bytes.length !== assetOffset) {
        [pluginHeader] = getPluginHeaderV1Decoder().read(bytes, assetOffset);

        const [, registryOffset] = getPluginRegistryV1AccountDataDecoder().read(
          bytes,
          Number(pluginHeader.pluginRegistryOffset)
        );
        finalOffset = registryOffset;

        // TODO: Parse plugins when plugins module is ported
      }

      const updateAuth: UpdateAuthority = {
        type: asset.updateAuthority.__kind,
        address:
          asset.updateAuthority.__kind === 'None'
            ? undefined
            : asset.updateAuthority.fields[0],
      };

      return [
        {
          pluginHeader,
          ...asset,
          updateAuthority: updateAuth,
        },
        finalOffset,
      ];
    },
  });
}
