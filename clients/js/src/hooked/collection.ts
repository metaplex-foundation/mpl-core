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
  deserializeBaseCollection,
  getBaseCollectionAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
} from '../generated';
import {
  Collection,
  PluginsList,
  registryRecordsToPluginsList,
} from '.';


export function deserializeCollection(rawAccount: RpcAccount): Collection {
  const collection = deserializeBaseCollection(rawAccount);
  const collectionData =
    getBaseCollectionAccountDataSerializer().serialize(collection);

  let pluginHeader: PluginHeaderAccountData | undefined;
  let pluginRegistry: PluginRegistryAccountData | undefined;
  let pluginsList: PluginsList | undefined;

  if (rawAccount.data.length !== collectionData.length) {
    [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
      rawAccount.data,
      collectionData.length
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
    ...collection,
  };
}

export async function safeFetchCollection(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<Collection | null> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  return maybeAccount.exists ? deserializeCollection(maybeAccount) : null;
}

export async function fetchCollection(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<Collection> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'Collection');
  return deserializeCollection(maybeAccount);
}
