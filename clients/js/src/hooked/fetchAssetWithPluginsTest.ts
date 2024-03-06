import {
  Context,
  Pda,
  PublicKey,
  RpcGetAccountOptions,
  assertAccountExists,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  deserializeAsset,
  getAssetAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
} from '../generated';
import { AssetWithPluginsTest, PluginsList } from './types';
import { registryRecordsToPluginsList } from './pluginHelpers';

export async function fetchAssetWithPluginsTest(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<AssetWithPluginsTest> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'Asset');
  const asset = deserializeAsset(maybeAccount);
  const assetData = getAssetAccountDataSerializer().serialize(asset);

  let pluginHeader: PluginHeaderAccountData | undefined;
  let pluginRegistry: PluginRegistryAccountData | undefined;
  let pluginsList: PluginsList | undefined;

  if (maybeAccount.data.length !== assetData.length) {
    [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
      maybeAccount.data,
      assetData.length
    );

    [pluginRegistry] = getPluginRegistryAccountDataSerializer().deserialize(
      maybeAccount.data,
      Number(pluginHeader.pluginRegistryOffset)
    );

    pluginsList = registryRecordsToPluginsList(
      pluginRegistry.registry,
      maybeAccount.data
    );
  }

  return {
    pluginHeader,
    ...pluginsList,
    ...asset,
  };
}
