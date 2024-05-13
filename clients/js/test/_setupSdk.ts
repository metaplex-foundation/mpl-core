/* eslint-disable import/no-extraneous-dependencies */
import {
  PublicKey,
  Signer,
  Umi,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi';
import {
  DataState,
  fetchAssetV1,
  fetchCollectionV1,
  CollectionV1,
  AssetV1,
  AssetPluginAuthorityPairArgsV2,
  ExternalPluginAdapterInitInfoArgs,
  create,
  createCollection as baseCreateCollection,
  CollectionPluginAuthorityPairArgsV2,
} from '../src';
import { DEFAULT_ASSET, DEFAULT_COLLECTION } from './_setupRaw';

export type CreateAssetHelperArgs = {
  owner?: PublicKey | Signer;
  payer?: Signer;
  asset?: Signer;
  dataState?: DataState;
  name?: string;
  uri?: string;
  authority?: Signer;
  updateAuthority?: PublicKey | Signer;
  collection?: PublicKey | CollectionV1;
  plugins?: (
    | ExternalPluginAdapterInitInfoArgs
    | AssetPluginAuthorityPairArgsV2
  )[];
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

  const col =
    typeof input.collection === 'string'
      ? await fetchCollectionV1(umi, input.collection as PublicKey)
      : (input.collection as CollectionV1 | undefined);

  await create(umi, {
    owner,
    payer,
    dataState: input.dataState,
    asset,
    updateAuthority,
    name: input.name || DEFAULT_ASSET.name,
    uri: input.uri || DEFAULT_ASSET.uri,
    plugins: input.plugins,
    collection: col,
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
  plugins?: (
    | ExternalPluginAdapterInitInfoArgs
    | CollectionPluginAuthorityPairArgsV2
  )[];
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
    collection,
  });

  return {
    asset,
    collection,
  };
};
