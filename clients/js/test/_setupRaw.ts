/* eslint-disable import/no-extraneous-dependencies */
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import { Assertions } from 'ava';
import {
  PublicKey,
  Signer,
  Umi,
  assertAccountExists,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi';
import {
  DataState,
  Key,
  createV1,
  fetchAssetV1,
  fetchCollectionV1,
  mplCore,
  createCollectionV1 as baseCreateCollection,
  CollectionV1,
  AssetV1,
  PluginAuthorityPairArgs,
  UpdateAuthority,
  ExternalPluginAdaptersList,
  AssetPluginsList,
  CollectionPluginsList,
  fetchAsset,
} from '../src';

export const createUmi = async () => (await basecreateUmi()).use(mplCore());

export type CreateAssetHelperArgs = {
  owner?: PublicKey | Signer;
  payer?: Signer;
  asset?: Signer;
  dataState?: DataState;
  name?: string;
  uri?: string;
  authority?: Signer;
  updateAuthority?: PublicKey | Signer;
  collection?: PublicKey;
  plugins?: PluginAuthorityPairArgs[];
};

export const DEFAULT_ASSET = {
  name: 'Test Asset',
  uri: 'https://example.com/asset',
};

export const DEFAULT_COLLECTION = {
  name: 'Test Collection',
  uri: 'https://example.com/collection',
};

export const createAsset = async (
  umi: Umi,
  input: CreateAssetHelperArgs = {}
) => {
  const payer = input.payer || umi.identity;
  const owner = publicKey(input.owner || input.payer || umi.identity);
  const asset = input.asset || generateSigner(umi);
  const updateAuthority = input.updateAuthority
    ? publicKey(input.updateAuthority)
    : undefined;

  await createV1(umi, {
    owner,
    payer,
    dataState: input.dataState,
    asset,
    updateAuthority,
    name: input.name || DEFAULT_ASSET.name,
    uri: input.uri || DEFAULT_ASSET.uri,
    plugins: input.plugins,
    collection: input.collection,
    authority: input.authority,
  }).sendAndConfirm(umi);

  return fetchAssetV1(umi, publicKey(asset));
};

export type CreateCollectionHelperArgs = {
  payer?: Signer;
  collection?: Signer;
  name?: string;
  uri?: string;
  updateAuthority?: PublicKey | Signer;
  plugins?: PluginAuthorityPairArgs[];
};

export const createCollection = async (
  umi: Umi,
  input: CreateCollectionHelperArgs = {}
) => {
  const payer = input.payer || umi.identity;
  const collection = input.collection || generateSigner(umi);
  const updateAuthority = publicKey(input.updateAuthority || payer);
  await baseCreateCollection(umi, {
    name: input.name || DEFAULT_COLLECTION.name,
    uri: input.uri || DEFAULT_COLLECTION.uri,
    collection,
    payer,
    updateAuthority,
    plugins: input.plugins,
  }).sendAndConfirm(umi);

  return fetchCollectionV1(umi, publicKey(collection));
};

export const createAssetWithCollection: (
  umi: Umi,
  assetInput: CreateAssetHelperArgs & { collection?: PublicKey | Signer },
  collectionInput?: CreateCollectionHelperArgs
) => Promise<{
  asset: AssetV1;
  collection: CollectionV1;
}> = async (umi, assetInput, collectionInput = {}) => {
  const collection = assetInput.collection
    ? await fetchCollectionV1(umi, publicKey(assetInput.collection))
    : await createCollection(umi, {
        payer: assetInput.payer,
        updateAuthority: assetInput.updateAuthority,
        ...collectionInput,
      });

  const asset = await createAsset(umi, {
    ...assetInput,
    collection: collection.publicKey,
  });

  return {
    asset,
    collection,
  };
};

export const assertAsset = async (
  t: Assertions,
  umi: Umi,
  input: {
    asset: PublicKey | Signer;
    owner: PublicKey | Signer;
    updateAuthority?: UpdateAuthority;
    name?: string | RegExp;
    uri?: string | RegExp;
  } & AssetPluginsList &
    ExternalPluginAdaptersList,
  options: { derivePlugins: boolean } = {
    derivePlugins: false,
  }
) => {
  const { asset, owner, name, uri, ...rest } = input;
  const assetAddress = publicKey(input.asset);
  const assetWithPlugins = options.derivePlugins
    ? await fetchAsset(umi, assetAddress)
    : await fetchAssetV1(umi, assetAddress);

  // Name.
  if (typeof name === 'string') t.is(assetWithPlugins.name, name);
  else if (name !== undefined) t.regex(assetWithPlugins.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(assetWithPlugins.uri, uri);
  else if (uri !== undefined) t.regex(assetWithPlugins.uri, uri);

  const testObj = <AssetV1>{
    key: Key.AssetV1,
    publicKey: assetAddress,
    owner: publicKey(owner),
    ...rest,
  };

  t.like(assetWithPlugins, testObj);
};

export const assertCollection = async (
  t: Assertions,
  umi: Umi,
  input: {
    collection: PublicKey | Signer;
    updateAuthority?: PublicKey | Signer;
    name?: string | RegExp;
    uri?: string | RegExp;
    numMinted?: number;
    currentSize?: number;
  } & CollectionPluginsList &
    ExternalPluginAdaptersList
) => {
  const { collection, name, uri, updateAuthority, ...rest } = input;

  const collectionAddress = publicKey(collection);
  const collectionWithPlugins = await fetchCollectionV1(umi, collectionAddress);

  // Name.
  if (typeof name === 'string') t.is(collectionWithPlugins.name, name);
  else if (name !== undefined) t.regex(collectionWithPlugins.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(collectionWithPlugins.uri, uri);
  else if (uri !== undefined) t.regex(collectionWithPlugins.uri, uri);

  const testObj = <CollectionV1>{
    key: Key.CollectionV1,
    publicKey: collectionAddress,
    ...rest,
  };

  if (updateAuthority) {
    testObj.updateAuthority = publicKey(updateAuthority);
  }

  t.like(collectionWithPlugins, testObj);
};

export const assertBurned = async (
  t: Assertions,
  umi: Umi,
  asset: PublicKey
) => {
  const account = await umi.rpc.getAccount(asset);
  t.true(account.exists);
  assertAccountExists(account);
  t.is(account.data.length, 1);
  t.is(account.data[0], Key.Uninitialized);
  return account;
};
