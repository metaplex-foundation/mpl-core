import {
  Context,
  Pda,
  PublicKey,
  RpcGetAccountOptions,
  assertAccountExists,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  Collection,
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  deserializeCollection,
  getCollectionAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
  getPluginSerializer,
} from '../generated';
import { PluginList, PluginWithAuthority } from '.';

export type CollectionWithPlugins = Collection & PluginList;

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
  let plugins: PluginWithAuthority[] | undefined;
  if (maybeAccount.data.length !== collectionData.length) {
    [pluginHeader] = getPluginHeaderAccountDataSerializer().deserialize(
      maybeAccount.data,
      collectionData.length
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

  const collectionWithPlugins: CollectionWithPlugins = {
    pluginHeader,
    plugins,
    pluginRegistry,
    ...collection,
  };

  return collectionWithPlugins;
}
