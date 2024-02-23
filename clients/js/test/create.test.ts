import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  Asset,
  DataState,
  create,
  fetchAsset,
  fetchHashedAsset,
  getAssetAccountDataSerializer,
} from '../src';
import { createUmi } from './_setup';

test('it can create a new asset in account state', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});

test('it can create a new asset in ledger state', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  const txResult = await create(umi, {
    dataState: DataState.LedgerState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchHashedAsset(umi, assetAddress.publicKey);
  // console.log(asset);
  t.like(asset, <Asset>{
    publicKey: assetAddress.publicKey,
  });

  const tx = await umi.rpc.getTransaction(txResult.signature);
  if (tx && tx.meta.innerInstructions) {
    // console.log(tx.meta.innerInstructions[0].instructions);
    const { data } = tx.meta.innerInstructions[0].instructions[0];
    // console.log(base58.deserialize(data));
    const parsed = getAssetAccountDataSerializer().deserialize(data)[0];
    // console.log("Ledger State:", parsed);
    t.like(parsed, <Asset>{
      updateAuthority: umi.identity.publicKey,
      owner: umi.identity.publicKey,
      name: 'Test Bread',
      uri: 'https://example.com/bread',
    });
  }
});
