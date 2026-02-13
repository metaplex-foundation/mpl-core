/* eslint-disable import/no-extraneous-dependencies */
import {
  assertAccountExists,
  deserializeAccount,
  generateSigner,
  PublicKey,
  publicKey,
  Signer,
  Umi,
} from '@metaplex-foundation/umi';
import { createUmi as basecreateUmi } from '@metaplex-foundation/umi-bundle-tests';
import { Assertions } from 'ava';
import {
  AssetPluginsList,
  AssetV1,
  createCollectionV1 as baseCreateCollection,
  CollectionPluginsList,
  CollectionV1,
  createGroupV1,
  createV1,
  DataState,
  ExternalPluginAdaptersList,
  fetchAsset,
  fetchAssetV1,
  fetchCollectionV1,
  fetchGroupV1,
  GroupPluginsList,
  GroupV1,
  Key,
  mplCore,
  PluginAuthorityPairArgs,
  UpdateAuthority,
} from '../src';
import { getGroupV1AccountDataSerializer } from '../src/hooked';

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

export const DEFAULT_GROUP = {
  name: 'Test Group',
  uri: 'https://example.com/group',
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

export const createGroup = async (
  umi: Umi,
  input: {
    name?: string;
    uri?: string;
    payer?: Signer;
    group?: Signer;
    updateAuthority?: PublicKey | Signer;
  } = {}
) => {
  const payer = input.payer || umi.identity;
  const group = input.group || generateSigner(umi);

  // Determine if the provided updateAuthority is a signer or just a public key.
  const providedUpdateAuth = input.updateAuthority;

  let updateAuthoritySigner: Signer | undefined;
  let updateAuthorityPubkey: PublicKey | undefined;

  if (providedUpdateAuth) {
    // Heuristic: If the value has a `secretKey` property, we treat it as a Signer.
    if (
      typeof providedUpdateAuth === 'object' &&
      'secretKey' in providedUpdateAuth
    ) {
      updateAuthoritySigner = providedUpdateAuth as Signer;
      updateAuthorityPubkey = publicKey(updateAuthoritySigner);
    } else {
      updateAuthorityPubkey = publicKey(providedUpdateAuth);
    }
  }

  // Step 1: create the group. If we have a signer for the update authority, pass it now.
  const createGroupArgs: Parameters<typeof createGroupV1>[1] = {
    name: input.name || DEFAULT_GROUP.name,
    uri: input.uri || DEFAULT_GROUP.uri,
    group,
    payer,
    relationships: [],
  };
  if (updateAuthoritySigner) {
    // Account field type now allows Signer.
    (createGroupArgs as any).updateAuthority = updateAuthoritySigner;
  }

  await createGroupV1(umi, createGroupArgs).sendAndConfirm(umi);

  // Step 2: If the desired update authority was provided as a public key (non-signer),
  // update the group to set that new update authority.
  if (updateAuthorityPubkey && !updateAuthoritySigner) {
    const { updateGroup } = await import('../src');
    await updateGroup(umi, {
      group: group.publicKey,
      payer,
      authority: payer,
      newUpdateAuthority: updateAuthorityPubkey,
      newName: null,
      newUri: null,
    }).sendAndConfirm(umi);
  }

  // If we have a signer for the update authority, delegate permissions to the payer
  if (updateAuthoritySigner) {
    const { plugin } = await import('../src/generated/types/plugin');
    const { addGroupPluginV1 } = await import('../src/generated');

    await addGroupPluginV1(umi, {
      group: group.publicKey,
      payer,
      authority: updateAuthoritySigner,
      plugin: plugin('UpdateDelegate', [
        {
          additionalDelegates: [publicKey(payer)],
        },
      ] as any),
      initAuthority: null,
    }).sendAndConfirm(umi);
  }

  return fetchGroupV1(umi, publicKey(group));
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

export const assertGroup = async (
  t: Assertions,
  umi: Umi,
  input: {
    group: PublicKey | Signer;
    updateAuthority?: PublicKey | Signer;
    name?: string | RegExp;
    uri?: string | RegExp;
    assets?: PublicKey[];
    collections?: PublicKey[];
    groups?: PublicKey[];
    parentGroups?: PublicKey[];
  } & GroupPluginsList &
    ExternalPluginAdaptersList
) => {
  const { group, name, uri, updateAuthority, ...rest } = input;

  const groupAddress = publicKey(group);
  const maybeGroupAccount = await umi.rpc.getAccount(groupAddress);
  assertAccountExists(maybeGroupAccount, 'GroupV1');
  const groupWithPlugins = deserializeAccount(
    maybeGroupAccount,
    getGroupV1AccountDataSerializer()
  );

  // Name.
  if (typeof name === 'string') t.is(groupWithPlugins.name, name);
  else if (name !== undefined) t.regex(groupWithPlugins.name, name);

  // Uri.
  if (typeof uri === 'string') t.is(groupWithPlugins.uri, uri);
  else if (uri !== undefined) t.regex(groupWithPlugins.uri, uri);

  const testObj = <GroupV1>{
    key: Key.GroupV1,
    publicKey: groupAddress,
    ...rest,
  };

  if (updateAuthority) {
    testObj.updateAuthority = publicKey(updateAuthority);
  }

  t.like(groupWithPlugins, testObj);
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
