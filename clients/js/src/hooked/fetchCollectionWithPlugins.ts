import {
  Context,
  Pda,
  PublicKey,
  RpcGetAccountOptions,
  assertAccountExists,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  CollectionData,
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  deserializeCollectionData,
  getCollectionDataAccountDataSerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
  getPluginSerializer,
} from '../generated';
import { PluginList, PluginWithAuthorities } from '.';

export type CollectionWithPlugins = CollectionData & PluginList;

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
  const collection = deserializeCollectionData(maybeAccount);
  const collectionData = getCollectionDataAccountDataSerializer().serialize(collection);

  let pluginHeader: PluginHeaderAccountData | undefined;
  let pluginRegistry: PluginRegistryAccountData | undefined;
  let plugins: PluginWithAuthorities[] | undefined;
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
      authorities: record.authorities,
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
