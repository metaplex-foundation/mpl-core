import { generateSigner, publicKey, sol } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  Asset,
  DataState,
  create,
  fetchHashedAsset,
  getAssetAccountDataSerializer,
  updateAuthority,
  plugin,
} from '../src';
import { DEFAULT_ASSET, assertAsset, createAsset, createUmi } from './_setup';

test('it can create a new asset in account state', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  await createAsset(umi, {
    asset: assetAddress,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetAddress.publicKey,
    owner: umi.identity,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });
});

test('it can create a new asset with a different payer', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = await generateSignerWithSol(umi, sol(1));
  const assetAddress = generateSigner(umi);

  create(umi, {
    ...DEFAULT_ASSET,
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    plugins: []
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner: umi.identity,
    payer,
  })

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [payer.publicKey]),
  });
});

test.skip('it can create a new asset in ledger state', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  const txResult = await create(umi, {
    dataState: DataState.LedgerState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
    plugins: [],
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
      updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
      owner: umi.identity.publicKey,
      name: 'Test Bread',
      uri: 'https://example.com/bread',
    });
  }
});

test('it cannot create a new asset in ledger state because it is not available', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  const result = create(umi, {
    dataState: DataState.LedgerState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
    plugins: [],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NotAvailable' });
});

test('it can create a new asset in account state with plugins', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [plugin('Freeze', [{ frozen: false }])],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: assetAddress.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    // pluginHeader: formPluginHeader(BigInt(120)),
    freeze: {
      authority: {
        type: 'Owner',
      },
      offset: BigInt(118),
      frozen: false,
    },
  })
});

// TODO: Add test
test('it can create a new asset in account state with a different update authority', async (t) => {
  t.pass();
});

// TODO: Add test
test('it can create a new asset in account state with plugins with a different update authority', async (t) => {
  t.pass();
});

// TODO: Add test
test('it can create a new asset in ledger state with plugins', async (t) => {
  t.pass();
});

// TODO: Add test
test('it can create a new asset in ledger state with a different update authority', async (t) => {
  t.pass();
});

// TODO: Add test
test('it can create a new asset in ledger state with plugins with a different update authority', async (t) => {
  t.pass();
});
