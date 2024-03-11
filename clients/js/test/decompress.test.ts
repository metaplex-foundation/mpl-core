import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  Asset,
  compress,
  create,
  DataState,
  decompress,
  fetchAsset,
  fetchHashedAsset,
  getAssetAccountDataSerializer,
  getHashedAssetSchemaSerializer,
  hash,
  HashedAssetSchema,
  updateAuthority,
} from '../src';
import { createUmi } from './_setup';

test.skip('it can decompress a previously compressed asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  // And when we compress the asset.
  await compress(umi, {
    asset: assetAddress.publicKey,
    authority: umi.identity,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  // And the asset is now compressed as a hashed asset.
  const afterCompressedAsset = await fetchHashedAsset(
    umi,
    assetAddress.publicKey
  );

  // And the hash matches the expected value.
  const hashedAssetSchema: HashedAssetSchema = {
    assetHash: hash(getAssetAccountDataSerializer().serialize(beforeAsset)),
    pluginHashes: [],
  };

  const hashedAsset = hash(
    getHashedAssetSchemaSerializer().serialize(hashedAssetSchema)
  );
  t.deepEqual(afterCompressedAsset.hash, hashedAsset);

  // And when we decompress the asset.
  await decompress(umi, {
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
  const afterDecompressedAsset = await fetchAsset(umi, assetAddress.publicKey);

  t.like(afterDecompressedAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});
