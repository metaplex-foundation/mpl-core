import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import {
  SPL_SYSTEM_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
} from '@metaplex-foundation/mpl-toolbox';
import {
  AssetWithPlugins,
  DataState,
  MPL_CORE_PROGRAM_ID,
  create,
  fetchAssetWithPlugins,
  plugin,
  ruleSet,
  transfer,
  updateAuthority,
} from '../../../src';
import { createUmi } from '../../_setup';

test('it can transfer an asset with royalties', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: newOwner,
    name: 'Owner',
    uri: '',
    plugins: [],
  }).sendAndConfirm(umi);

  // Creating a new asset to transfer.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      plugin('Royalties', [
        {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('None'),
        },
      ]),
    ],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await transfer(umi, {
    asset: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  const afterAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: newOwner.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});

test('it can transfer an asset with royalties to an allowlisted program address', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: newOwner,
    name: 'Owner',
    uri: '',
    plugins: [],
  }).sendAndConfirm(umi);

  // Creating a new asset to transfer.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      plugin('Royalties', [
        {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [[MPL_CORE_PROGRAM_ID]]),
        },
      ]),
    ],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await transfer(umi, {
    asset: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  const afterAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: newOwner.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});

test('it cannot transfer an asset with royalties to a program address not on the allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const newerOwner = generateSigner(umi);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: newOwner,
    name: 'Owner',
    uri: '',
    plugins: [],
  }).sendAndConfirm(umi);

  // Create a second one because allowlist needs both to be off the allowlist.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: newerOwner,
    name: 'Owner',
    uri: '',
    plugins: [],
  }).sendAndConfirm(umi);

  // Creating a new asset to transfer.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      plugin('Royalties', [
        {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
        },
      ]),
    ],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await transfer(umi, {
    asset: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset: assetAddress.publicKey,
    newOwner: newerOwner.publicKey,
    authority: newOwner,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it can transfer an asset with royalties to a program address not on the denylist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: newOwner,
    name: 'Owner',
    uri: '',
    plugins: [],
  }).sendAndConfirm(umi);

  // Creating a new asset to transfer.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      plugin('Royalties', [
        {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
        },
      ]),
    ],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await transfer(umi, {
    asset: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  const afterAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", afterAsset);
  t.like(afterAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: newOwner.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });
});

test('it cannot transfer an asset with royalties to a denylisted program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: newOwner,
    name: 'Owner',
    uri: '',
    plugins: [],
  }).sendAndConfirm(umi);

  // Creating a new asset to transfer.
  await create(umi, {
    dataState: DataState.AccountState,
    asset: assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    plugins: [
      plugin('Royalties', [
        {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
        },
      ]),
    ],
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const beforeAsset = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log("Account State:", beforeAsset);
  t.like(beforeAsset, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  const result = transfer(umi, {
    asset: assetAddress.publicKey,
    newOwner: newOwner.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});
