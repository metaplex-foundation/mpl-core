import {
  Context,
  Pda,
  PublicKey,
  RpcAccount,
  RpcGetAccountOptions,
  assertAccountExists,
  gpaBuilder,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  publicKey as publicKeySerializer,
  string,
  u32,
} from '@metaplex-foundation/umi/serializers';
import {
  KeyArgs,
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  deserializeBaseCollection,
  getBaseCollectionAccountDataSerializer,
  getKeySerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
} from '../generated';
import { Collection, PluginsList, registryRecordsToPluginsList } from '.';

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

export function getCollectionGpaBuilder(
  context: Pick<Context, 'rpc' | 'programs'>
) {
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'
  );
  return gpaBuilder(context, programId)
    .registerFields<{
      key: KeyArgs;
      updateAuthority: PublicKey;
      name: string;
      uri: string;
      numMinted: number;
      currentSize: number;
    }>({
      key: [0, getKeySerializer()],
      updateAuthority: [1, publicKeySerializer()],
      name: [33, string()],
      uri: [null, string()],
      numMinted: [null, u32()],
      currentSize: [null, u32()],
    })
    .deserializeUsing<Collection>((account) => deserializeCollection(account));
}
