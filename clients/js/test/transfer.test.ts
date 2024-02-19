import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import { Asset, DataState, create, fetchAsset, transfer } from '../src';
import { createUmi } from './_setup';

test('it can transfer an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
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

  await transfer(umi, {
    assetAddress: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null
  }).sendAndConfirm(umi);

  const afterAsset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: newOwner.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});

test('it cannot transfer an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const attacker = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
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

  const result = transfer(umi, {
    assetAddress: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' })

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
