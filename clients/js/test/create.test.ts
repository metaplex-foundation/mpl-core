import { generateSigner, publicKey, sol } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  Asset,
  AssetWithPlugins,
  DataState,
  create,
  fetchAsset,
  fetchAssetWithPlugins,
  fetchHashedAsset,
  getAssetAccountDataSerializer,
  updateAuthority,
} from '../src';
import { DEFAULT_ASSET, assertAsset, createAsset, createUmi } from './_setup';

test('it can create a new asset in account state', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  await createAsset(umi, {
    asset: assetAddress,
  })

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetAddress.publicKey,
    owner: umi.identity,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  })
});

test('it can create a new asset with a different payer', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const payer = await generateSignerWithSol(umi, sol(1));
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    owner: umi.identity.publicKey,
    payer,
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority("Address", [payer.publicKey]),
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
    plugins: [{ __kind: 'Freeze', fields: [{ frozen: false }] }],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(120),
    },
    pluginRegistry: {
      key: 4,
      registry: [
        {
          pluginType: 2,
          offset: BigInt(118),
          authority: { __kind: 'Owner' },
        },
      ],
    },
    plugins: [
      {
        authority: { __kind: 'Owner' },
        plugin: {
          __kind: 'Freeze',
          fields: [{ frozen: false }],
        },
      },
    ],
  });
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
