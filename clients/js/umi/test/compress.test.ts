import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { getAssetV1AccountDataSerializer } from '../src/hooked';
import {
  AssetV1,
  compressV1,
  createV1,
  DataState,
  fetchAssetV1,
  fetchHashedAssetV1,
  getHashedAssetSchemaSerializer,
  hash,
  HashedAssetSchema,
} from '../src';

import { createAsset, createUmi } from './_setupRaw';

test.skip('it can compress an asset without any plugins as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi);

  // And when we compress the asset.
  await compressV1(umi, {
    asset: asset.publicKey,
    authority: umi.identity,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);
  // console.log('Compress signature: ', bs58.encode(tx.signature));

  // And the asset is now compressed as a hashed asset.
  const afterAsset = await fetchHashedAssetV1(umi, asset.publicKey);
  // console.log("Account State:", afterAsset);

  // And the hash matches the expected value.
  const hashedAssetSchema: HashedAssetSchema = {
    assetHash: hash(getAssetV1AccountDataSerializer().serialize(asset)),
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
  await createV1(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetV1(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetV1>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  const result = compressV1(umi, {
    asset: assetAddress.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  const afterAsset = await fetchAssetV1(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <AssetV1>{
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
  await createV1(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetV1(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetV1>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  // And when we compress the asset.
  const result = compressV1(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NotAvailable' });
});

test('it cannot use an invalid system program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const fakeSystemProgram = generateSigner(umi);

  // When we create a new account.
  await createV1(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetV1(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetV1>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  // And when we compress the asset.
  const result = compressV1(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    systemProgram: fakeSystemProgram.publicKey,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid log wrapper program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const fakeLogWrapper = generateSigner(umi);

  // When we create a new account.
  await createV1(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetV1(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetV1>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  // And when we compress the asset.
  const result = compressV1(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});
