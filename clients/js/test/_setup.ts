/* eslint-disable import/no-extraneous-dependencies */
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import { Assertions } from 'ava';
import {
  PublicKey,
  Signer,
  Umi,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi';
import {
  DataState,
  Key,
  create,
  fetchAsset,
  fetchCollection,
  mplCore,
  createCollection as baseCreateCollection,
  Collection,
  Asset,
  UpdateAuthority,
  PluginsList,
  PluginAuthorityPairArgs,
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
  // TODO use PluginList type here
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
  const updateAuthority = publicKey(input.updateAuthority || payer);
  await create(umi, {
    owner,
    payer,
    dataState: input.dataState || DataState.AccountState,
    asset,
    updateAuthority,
    name: input.name || DEFAULT_ASSET.name,
    uri: input.uri || DEFAULT_ASSET.uri,
    plugins: input.plugins,
    collection: input.collection,
    authority: input.authority,
  }).sendAndConfirm(umi);

  return fetchAsset(umi, publicKey(asset));
};

export type CreateCollectionHelperArgs = {
  payer?: Signer;
  collection?: Signer;
  name?: string;
  uri?: string;
  updateAuthority?: PublicKey | Signer;
  // TODO use CollectionPluginList type here
  plugins?: PluginAuthorityPairArgs[];
};

export const createCollection = async (
  umi: Umi,
  input: CreateCollectionHelperArgs
) => {
  const payer = input.payer || umi.identity;
  const collection = input.collection || generateSigner(umi);
  const updateAuthority = publicKey(input.updateAuthority || payer);
  await baseCreateCollection(umi, {
    ...DEFAULT_COLLECTION,
    collection,
    payer,
    updateAuthority,
    plugins: input.plugins,
  }).sendAndConfirm(umi);

  return fetchCollection(umi, publicKey(collection));
};

export const createAssetWithCollection: (
  umi: Umi,
  assetInput: CreateAssetHelperArgs & { collection?: PublicKey | Signer },
  collectionInput?: CreateCollectionHelperArgs
) => Promise<{
  asset: Asset;
  collection: Collection;
}> = async (umi, assetInput, collectionInput = {}) => {
  const collection = assetInput.collection
    ? await fetchCollection(umi, publicKey(assetInput.collection))
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
  } & PluginsList
) => {
  const { asset, owner, name, uri, ...rest } = input;
  const assetAddress = publicKey(input.asset);
  const assetWithPlugins = await fetchAsset(umi, assetAddress);

  // Name.
  if (typeof name === 'string') t.is(assetWithPlugins.name, name);
  else if (name !== undefined) t.regex(assetWithPlugins.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(assetWithPlugins.uri, uri);
  else if (uri !== undefined) t.regex(assetWithPlugins.uri, uri);

  const testObj = <Asset>{
    key: Key.Asset,
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
  } & PluginsList
) => {
  const { collection, name, uri, updateAuthority, ...rest } = input;

  const collectionAddress = publicKey(collection);
  const collectionWithPlugins = await fetchCollection(umi, collectionAddress);

  // Name.
  if (typeof name === 'string') t.is(collectionWithPlugins.name, name);
  else if (name !== undefined) t.regex(collectionWithPlugins.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(collectionWithPlugins.uri, uri);
  else if (uri !== undefined) t.regex(collectionWithPlugins.uri, uri);

  const testObj = <Collection>{
    key: Key.Collection,
    publicKey: collectionAddress,
    ...rest,
  };

  if (updateAuthority) {
    testObj.updateAuthority = publicKey(updateAuthority);
  }

  t.like(collectionWithPlugins, testObj);
};
