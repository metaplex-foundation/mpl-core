import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import { DataState, create, compress /* fetchAsset, fetchHashedAsset */ } from '../src';
import { createUmi } from './_setup';

test('fetch account info for account state', async (t) => {
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

  // Print the size of the account.
  const account = await umi.rpc.getAccount(assetAddress.publicKey);
  if (account.exists) {
    console.log(`Account Size ${account.data.length} bytes`);
  }

  // Then an account was created with the correct data.
  // const nonFungible = await fetchAsset(umi, assetAddress.publicKey);
  // console.log(nonFungible);

  t.pass();
});

test('HELLO EHLO  HLEOOO  fetch account info for ledger state', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
    plugins: [],
  }).sendAndConfirm(umi);

  // And when we compress the asset.
  await compress(umi, {
    asset: assetAddress.publicKey,
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  // Print the size of the account.
  const account = await umi.rpc.getAccount(assetAddress.publicKey);
  if (account.exists) {
    console.log(`Account Size ${account.data.length} bytes`);
  }

  // Then an account was created with the correct data.
  // const nonFungible = await fetchHashedAsset(umi, assetAddress.publicKey);
  // console.log(nonFungible);

  t.pass();
});
