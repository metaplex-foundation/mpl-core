/* eslint-disable import/no-extraneous-dependencies */
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import { Assertions } from 'ava';
import { PublicKey, Signer, Umi, generateSigner, publicKey } from '@metaplex-foundation/umi';
import {
  DataState,
  Key,
  create,
  fetchAssetWithPlugins,
  fetchCollectionWithPlugins,
  mplCore,
  PluginArgs,
  createCollection as baseCreateCollection,
  CollectionWithPlugins,
  AssetWithPlugins,
  UpdateAuthority,
  PluginWithAuthority,
} from '../src';

export const createUmi = async () => (await basecreateUmi()).use(mplCore());

export type CreateAssetHelperArgs = {
  owner?: PublicKey | Signer;
  payer?: Signer;
  asset?: Signer
  dataState?: DataState;
  name?: string;
  uri?: string;
  authority?: Signer;
  updateAuthority?: PublicKey | Signer
  collection?: PublicKey
  // TODO use PluginList type here
  plugins?: PluginArgs[]
}

export const DEFAULT_ASSET = {
  name: 'Test Asset',
  uri: 'https://example.com/asset',
}

export const DEFAULT_COLLECTION = {
  name: 'Test Collection',
  uri: 'https://example.com/collection',
}

export const createAsset = async (
  umi: Umi,
  input: CreateAssetHelperArgs
) => {
  const payer = input.payer || umi.identity;
  const owner = publicKey(input.owner || input.payer || umi.identity);
  const asset = input.asset || generateSigner(umi)
  const updateAuthority = publicKey(input.updateAuthority || payer)
  await create(umi, {
    owner,
    payer,
    dataState: input.dataState || DataState.AccountState,
    asset,
    updateAuthority,
    name: input.name || DEFAULT_ASSET.name,
    uri: input.uri || DEFAULT_ASSET.uri,
    plugins: input.plugins || [],
    collection: input.collection,
    authority: input.authority
  }).sendAndConfirm(umi);

  return fetchAssetWithPlugins(umi, publicKey(asset));
};

export type CreateCollectionHelperArgs = {
  payer?: Signer;
  collection?: Signer
  name?: string;
  uri?: string;
  updateAuthority?: PublicKey | Signer
  // TODO use CollectionPluginList type here 
  plugins?: PluginArgs[]
}

export const createCollection = async (umi: Umi, input:CreateCollectionHelperArgs) => {
  const payer = input.payer || umi.identity;
  const collection = input.collection || generateSigner(umi);
  const updateAuthority = publicKey(input.updateAuthority || payer);
  await baseCreateCollection(umi, {
    ...DEFAULT_COLLECTION,
    collection,
    payer,
    updateAuthority,
    plugins: input.plugins || []
  }).sendAndConfirm(umi);

  return fetchCollectionWithPlugins(umi, publicKey(collection));
}

export const createAssetWithCollection: (
  umi: Umi, 
  assetInput: CreateAssetHelperArgs & { collection?: PublicKey | Signer}, 
  collectionInput?: CreateCollectionHelperArgs
) => Promise<{ asset: AssetWithPlugins; collection: CollectionWithPlugins }> = async (umi, assetInput, collectionInput = {}) => {
  const collection = assetInput.collection ? await fetchCollectionWithPlugins(umi, publicKey(assetInput.collection)) : await createCollection(umi, {
    payer: assetInput.payer,
    updateAuthority: assetInput.updateAuthority,
    ...collectionInput
  });

  const asset = await createAsset(umi, {...assetInput, collection: collection.publicKey});

  return {
    asset,
    collection
  }
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
    // TODO replace with remapped PluginList type
    plugins?: PluginWithAuthority[]
  }
) => {
  const assetAddress = publicKey(input.asset);
  const owner = publicKey(input.owner);
  const { name, uri, updateAuthority, plugins } = input;
  const asset = await fetchAssetWithPlugins(umi, assetAddress);

  // Name.
  if (typeof name === 'string') t.is(asset.name, name);
  else if (name !== undefined) t.regex(asset.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(asset.uri, uri);
  else if (uri !== undefined) t.regex(asset.uri, uri);

  const testObj = <AssetWithPlugins>{
    key: Key.Asset,
    publicKey: assetAddress,
    owner,
  }

  if (plugins) {
    testObj.plugins = plugins;
  }

  if (updateAuthority) {
    testObj.updateAuthority = updateAuthority;
  }

  t.like(asset, testObj);

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
    // TODO replace with remapped PluginList type
    plugins?: PluginWithAuthority[]
  }
) => {
  const collectionAddress = publicKey(input.collection);
  const { name, uri, numMinted, currentSize, updateAuthority } = input;
  const collection = await fetchCollectionWithPlugins(umi, collectionAddress);

  // Name.
  if (typeof name === 'string') t.is(collection.name, name);
  else if (name !== undefined) t.regex(collection.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(collection.uri, uri);
  else if (uri !== undefined) t.regex(collection.uri, uri);

  const testObj = <CollectionWithPlugins>{
    key: Key.Collection,
    publicKey: collectionAddress,
    updateAuthority,
    plugins: input.plugins || [],
  }

  if(numMinted !== undefined) {
    testObj.numMinted = numMinted;
  };

  if(currentSize !== undefined) {
    testObj.currentSize = currentSize;
  }

  if(updateAuthority) {
    testObj.updateAuthority = publicKey(updateAuthority);
  }

  t.like(collection, testObj);

};
