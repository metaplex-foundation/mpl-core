import {
  assertAccountExists,
  generateSigner,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  Asset,
  DataState,
  create,
  fetchAsset,
  burn,
  Key,
  updateAuthority,
} from '../../../src';
import { createUmi } from '../../_setup';

test('it can burn an asset as the owner', async (t) => {
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

  await burn(umi, {
    asset: assetAddress.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  // And the asset address still exists but was resized to 1.
  const afterAsset = await umi.rpc.getAccount(assetAddress.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.deepEqual(afterAsset.lamports, sol(0.00089784 + 0.0015));
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});

test('it cannot burn an asset if not the owner', async (t) => {
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

  const result = burn(umi, {
    asset: assetAddress.publicKey,
    compressionProof: null,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  const afterAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});
