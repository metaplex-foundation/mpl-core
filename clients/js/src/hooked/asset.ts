import {
  Context,
  Pda,
  PublicKey,
  RpcAccount,
  RpcGetAccountOptions,
  assertAccountExists,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  deserializeBaseAsset,
  getBaseAssetAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
} from '../generated';
import { Asset, PluginsList } from './types';
import { registryRecordsToPluginsList } from './plugin';


export function deserializeAsset(rawAccount: RpcAccount): Asset {
  const asset = deserializeBaseAsset(rawAccount);
  const assetData = getBaseAssetAccountDataSerializer().serialize(asset);

  let pluginHeader: PluginHeaderAccountData | undefined;
  let pluginRegistry: PluginRegistryAccountData | undefined;
  let pluginsList: PluginsList | undefined;

  if (rawAccount.data.length !== assetData.length) {
    [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
      rawAccount.data,
      assetData.length
    );

    [pluginRegistry] = getPluginRegistryAccountDataSerializer().deserialize(
      rawAccount.data,
      Number(pluginHeader.pluginRegistryOffset)
    );

    pluginsList = registryRecordsToPluginsList(
      pluginRegistry.registry,
      rawAccount.data
    );
  }

  return {
    pluginHeader,
    ...pluginsList,
    ...asset,
  };
}

export async function safeFetchAsset(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<Asset | null> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  return maybeAccount.exists ? deserializeAsset(maybeAccount) : null;
}

export async function fetchAsset(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<Asset> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'Asset');
  return deserializeAsset(maybeAccount);
}
