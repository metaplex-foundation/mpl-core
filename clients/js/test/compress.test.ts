import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  Asset,
  baseAsset,
  compress,
  create,
  DataState,
  fetchAsset,
  fetchHashedAsset,
  getBaseAssetAccountDataSerializer,
  getHashedAssetSchemaSerializer,
  hash,
  HashedAssetSchema,
} from '../src';
import { createAsset, createUmi } from './_setup';

test.skip('it can compress an asset without any plugins as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  // And when we compress the asset.
  await compress(umi, {
    asset: asset.publicKey,
    authority: umi.identity,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);
  // console.log('Compress signature: ', bs58.encode(tx.signature));

  // And the asset is now compressed as a hashed asset.
  const afterAsset = await fetchHashedAsset(umi, asset.publicKey);
  // console.log("Account State:", afterAsset);

  // And the hash matches the expected value.
  const hashedAssetSchema: HashedAssetSchema = {
    assetHash: hash(getBaseAssetAccountDataSerializer().serialize(baseAsset(asset))),
    pluginHashes: [],
  };

  const hashedAsset = hash(
    getHashedAssetSchemaSerializer().serialize(hashedAssetSchema)
  );
  t.deepEqual(afterAsset.hash, hashedAsset);
});

test.skip('it cannot compress an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const attacker = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  const result = compress(umi, {
    asset: assetAddress.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  const afterAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});

test('it cannot compress an asset because it is not available', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  // And when we compress the asset.
  const result = compress(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NotAvailable' });
});
