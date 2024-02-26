import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  Asset,
  compress,
  create,
  DataState,
  fetchAsset,
  fetchHashedAsset,
  getAssetAccountDataSerializer,
  getHashedAssetSchemaSerializer,
  hash,
  HashedAssetSchema,
} from '../src';
import { createUmi } from './_setup';
// import bs58 from 'bs58';

test('it can compress an asset without any plugins as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  // And when we compress the asset.
  await compress(umi, {
    assetAddress: assetAddress.publicKey,
    owner: umi.identity,
  }).sendAndConfirm(umi);
  // console.log('Compress signature: ', bs58.encode(tx.signature));

  // And the asset is now compressed as a hashed asset.
  const afterAsset = await fetchHashedAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);

  // And the hash matches the expected value.
  const hashedAssetSchema: HashedAssetSchema = {
    assetHash: hash(getAssetAccountDataSerializer().serialize(beforeAsset)),
    pluginHashes: [],
  };

  const hashedAsset = hash(
    getHashedAssetSchemaSerializer().serialize(hashedAssetSchema)
  );
  t.deepEqual(afterAsset.hash, hashedAsset);
});

test('it cannot compress an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const attacker = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  const result = compress(umi, {
    assetAddress: assetAddress.publicKey,
    owner: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  const afterAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});
