import { generateSigner, publicKey, sol } from '@metaplex-foundation/umi';
import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { createAccount } from '@metaplex-foundation/mpl-toolbox';
import { getAssetAccountDataSerializer } from '../src/hooked';
import {
  Asset,
  create,
  DataState,
  fetchHashedAsset,
  pluginAuthorityPair,
} from '../src';
import { assertAsset, createAsset, createUmi, DEFAULT_ASSET } from './_setup';

test('it can create a new asset in account state', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  await createAsset(umi, {
    asset: assetAddress,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetAddress.publicKey,
    owner: umi.identity,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it can create a new asset with a different payer', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const payer = await generateSignerWithSol(umi, sol(1));
  const assetAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    asset: assetAddress,
    owner: umi.identity,
    payer,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: payer.publicKey },
  });
});

test.skip('it can create a new asset in ledger state', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  const txResult = await create(umi, {
    dataState: DataState.LedgerState,
    asset: assetAddress,
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
      updateAuthority: { type: 'Address', address: umi.identity.publicKey },
      owner: umi.identity.publicKey,
      name: 'Test Bread',
      uri: 'https://example.com/bread',
    });
  }
});

test('it cannot create a new asset in ledger state because it is not available', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  const result = create(umi, {
    dataState: DataState.LedgerState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NotAvailable' });
});

test('it can create a new asset in account state with plugins', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      pluginAuthorityPair({
        type: 'Freeze',
        data: { frozen: false },
      }),
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: assetAddress.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    // pluginHeader: formPluginHeader(BigInt(120)),
    freeze: {
      authority: {
        type: 'Owner',
      },
      offset: BigInt(119),
      frozen: false,
    },
  });
});

test('it can create a new asset in account state with a different update authority', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  await createAsset(umi, {
    asset: assetAddress,
    updateAuthority: updateAuth.publicKey,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetAddress.publicKey,
    owner: umi.identity,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
  });
});

test('it can create a new asset in account state with plugins with a different update authority', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  await createAsset(umi, {
    asset: assetAddress,
    updateAuthority: updateAuth.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'Freeze',
        data: { frozen: false },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetAddress.publicKey,
    owner: umi.identity,
    updateAuthority: { type: 'Address', address: updateAuth.publicKey },
    freeze: {
      authority: { type: 'Owner' },
      frozen: false,
    },
  });
});

test('it cannot create a new asset if the address is already in use', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);

  await createAsset(umi, {
    asset: assetAddress,
  });

  const result = create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: DEFAULT_ASSET.name,
    uri: DEFAULT_ASSET.uri,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot create a new asset if the address is not owned by the system program', async (t) => {
  // Given an Umi instance and a new signer.
  const umi = await createUmi();
  const newAccount = generateSigner(umi);

  // Then a new account with a non-system program as an owner is created using the new signer.
  await createAccount(umi, {
    newAccount,
    lamports: sol(0.1),
    space: 42,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  // The invalid system program error is expected.
  const result = create(umi, {
    ...DEFAULT_ASSET,
    dataState: DataState.AccountState,
    asset: newAccount,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

// TODO: Add test
test.skip('it can create a new asset in ledger state with plugins', async (t) => {
  t.pass();
});

// TODO: Add test
test.skip('it can create a new asset in ledger state with a different update authority', async (t) => {
  t.pass();
});

// TODO: Add test
test.skip('it can create a new asset in ledger state with plugins with a different update authority', async (t) => {
  t.pass();
});
