import {
  Context,
  OptionOrNullable,
  Pda,
  PublicKey,
  RpcAccount,
  RpcGetAccountOptions,
  assertAccountExists,
  gpaBuilder,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  option,
  publicKey as publicKeySerializer,
  string,
  u64,
} from '@metaplex-foundation/umi/serializers';
import {
  BaseAsset,
  KeyArgs,
  PluginHeaderAccountData,
  PluginRegistryAccountData,
  UpdateAuthorityArgs,
  deserializeBaseAsset,
  getBaseAssetAccountDataSerializer,
  getKeySerializer,
  getPluginHeaderAccountDataSerializer,
  getPluginRegistryAccountDataSerializer,
  getUpdateAuthoritySerializer,
  updateAuthority,
} from '../generated';
import { Asset, Collection, PluginsList } from './types';
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
  const updateAuth = {
    type: asset.updateAuthority.__kind,
    address: asset.updateAuthority.__kind ==='None' ? undefined : asset.updateAuthority.fields[0],
  }

  return {
    pluginHeader,
    ...pluginsList,
    ...asset,
    updateAuthority: updateAuth,
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

export function getAssetGpaBuilder(context: Pick<Context, 'rpc' | 'programs'>) {
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'
  );
  return gpaBuilder(context, programId)
    .registerFields<{
      key: KeyArgs;
      owner: PublicKey;
      updateAuthority: UpdateAuthorityArgs;
      name: string;
      uri: string;
      seq: OptionOrNullable<number | bigint>;
    }>({
      key: [0, getKeySerializer()],
      owner: [1, publicKeySerializer()],
      updateAuthority: [33, getUpdateAuthoritySerializer()],
      name: [null, string()],
      uri: [null, string()],
      seq: [null, option(u64())],
    })
    .deserializeUsing<Asset>((account) => deserializeAsset(account));
}

export function deriveAssetPlugins(
  asset: Asset,
  collection?: Collection
): Asset {
  if (!collection) return asset;
  if (asset.updateAuthority.type !== 'Collection') return asset;
  if (asset.updateAuthority.address !== collection?.updateAuthority) {
    throw new Error('Collection address does not match asset update authority');
  }
  const {
    name,
    uri,
    updateAuthority: updateAuth,
    currentSize,
    header,
    key,
    numMinted,
    publicKey,
    ...rest
  } = collection;
  return {
    ...rest,
    ...asset,
  };
}

// hax
export function baseAsset(asset: Asset): BaseAsset {
  return {
    ...asset,
    updateAuthority:
      updateAuthority(asset.updateAuthority.type as any, asset.updateAuthority.address? [asset.updateAuthority.address] : undefined as any)
  }
}