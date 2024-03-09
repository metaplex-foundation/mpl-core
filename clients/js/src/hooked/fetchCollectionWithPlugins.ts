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
  deserializeCollection,
  getCollectionAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
} from '../generated';
import {
  CollectionWithPlugins,
  PluginsList,
  registryRecordsToPluginsList,
} from '.';

export async function fetchCollectionWithPlugins(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<CollectionWithPlugins> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'Collection');
  const collection = deserializeCollection(maybeAccount);
  const collectionData =
    getCollectionAccountDataSerializer().serialize(collection);

  let pluginHeader: PluginHeaderAccountData | undefined;
  let pluginRegistry: PluginRegistryAccountData | undefined;
  let pluginsList: PluginsList | undefined;

  if (maybeAccount.data.length !== collectionData.length) {
    [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
      maybeAccount.data,
      collectionData.length
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
    ...collection,
  };
}
