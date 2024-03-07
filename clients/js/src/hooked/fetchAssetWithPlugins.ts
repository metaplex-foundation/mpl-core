import {
  Context,
  Pda,
  PublicKey,
  RpcGetAccountOptions,
  assertAccountExists,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  Asset,
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  deserializeAsset,
  getAssetAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
  getPluginSerializer,
} from '../generated';
import { PluginList, PluginWithAuthority } from '.';

export type AssetWithPlugins = Asset & PluginList;

export async function fetchAssetWithPlugins(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<AssetWithPlugins> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'Asset');
  const asset = deserializeAsset(maybeAccount);
  const assetData = getAssetAccountDataSerializer().serialize(asset);

  let pluginHeader: PluginHeaderAccountData | undefined;
  let pluginRegistry: PluginRegistryAccountData | undefined;
  let plugins: PluginWithAuthority[] | undefined;
  if (maybeAccount.data.length !== assetData.length) {
    [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
      maybeAccount.data,
      assetData.length
    );
    [pluginRegistry] = getPluginRegistryAccountDataSerializer().deserialize(
      maybeAccount.data,
      Number(pluginHeader.pluginRegistryOffset)
    );
    plugins = pluginRegistry.registry.map((record) => ({
      plugin: getPluginSerializer().deserialize(
        maybeAccount.data,
        Number(record.offset)
      )[0],
      authority: record.authority,
    }));
  }

  const assetWithPlugins: AssetWithPlugins = {
    pluginHeader,
    plugins,
    pluginRegistry,
    ...asset,
  };

  return assetWithPlugins;
}
