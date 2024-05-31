import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { getAssetV1AccountDataSerializer } from '../src/hooked';
import {
  AssetV1,
  compressV1,
  createV1,
  DataState,
  decompressV1,
  fetchAssetV1,
  fetchHashedAssetV1,
  getHashedAssetSchemaSerializer,
  hash,
  HashedAssetSchema,
  updateAuthority,
} from '../src';
import { createUmi } from './_setupRaw';

test.skip('it can decompress a previously compressed asset as the owner', async (t) => {
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
  await compressV1(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  // And the asset is now compressed as a hashed asset.
  const afterCompressedAsset = await fetchHashedAssetV1(
    umi,
    assetAddress.publicKey
  );

  // And the hash matches the expected value.
  const hashedAssetSchema: HashedAssetSchema = {
    assetHash: hash(getAssetV1AccountDataSerializer().serialize(beforeAsset)),
    pluginHashes: [],
  };

  const hashedAsset = hash(
    getHashedAssetSchemaSerializer().serialize(hashedAssetSchema)
  );
  t.deepEqual(afterCompressedAsset.hash, hashedAsset);

  // And when we decompress the asset.
  await decompressV1(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    compressionProof: {
      updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
      owner: umi.identity.publicKey,
      name: 'Test Bread',
      uri: 'https://example.com/bread',
      seq: 1,
      plugins: [],
    },
  }).sendAndConfirm(umi);

  // Then the asset is now decompressed into an asset.
  const afterDecompressedAsset = await fetchAssetV1(
    umi,
    assetAddress.publicKey
  );

  t.like(afterDecompressedAsset, <AssetV1>{
    publicKey: assetAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
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

  const result = decompressV1(umi, {
    asset: assetAddress.publicKey,
    systemProgram: fakeSystemProgram.publicKey,
    compressionProof: {
      owner: umi.identity.publicKey,
      updateAuthority: {
        __kind: 'None',
      },
      name: '',
      uri: '',
      seq: 0,
      plugins: [],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program', async (t) => {
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

  const result = decompressV1(umi, {
    asset: assetAddress.publicKey,
    logWrapper: fakeLogWrapper.publicKey,
    compressionProof: {
      owner: umi.identity.publicKey,
      updateAuthority: {
        __kind: 'None',
      },
      name: '',
      uri: '',
      seq: 0,
      plugins: [],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});
