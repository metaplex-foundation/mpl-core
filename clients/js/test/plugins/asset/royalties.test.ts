import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  SPL_SYSTEM_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
} from '@metaplex-foundation/mpl-toolbox';
import {
  MPL_CORE_PROGRAM_ID,
  plugin,
  ruleSet,
  transfer,
  updateAuthority,
} from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it can transfer an asset with royalties', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      {
        plugin: plugin('Royalties', [
          {
            percentage: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('None'),
          },
        ]),
        authority: null,
      },
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),

    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwned.publicKey,
  });
});

test('it can transfer an asset with royalties to an allowlisted program address', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        plugin: plugin('Royalties', [
          {
            percentage: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramAllowList', [[MPL_CORE_PROGRAM_ID]]),
          },
        ]),
        authority: null,
      },
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [[MPL_CORE_PROGRAM_ID]]),
    },
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwned.publicKey,
  });
});

test('it cannot transfer an asset with royalties to a program address not on the allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const programOwner = generateSigner(umi);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi, {
    owner: programOwner.publicKey,
  });

  // Create a second one because allowlist needs both to be off the allowlist.
  const programOwned2 = await createAsset(umi);

  // Creating a new asset to transfer.
  const asset = await createAsset(umi, {
    plugins: [
      {
        plugin: plugin('Royalties', [
          {
            percentage: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
          },
        ]),
        authority: null,
      },
    ],
  });

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset: asset.publicKey,
    newOwner: programOwned2.publicKey,
    authority: programOwner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it can transfer an asset with royalties to a program address not on the denylist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Creating a new asset to transfer.
  const asset = await createAsset(umi, {
    plugins: [
      {
        plugin: plugin('Royalties', [
          {
            percentage: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
          },
        ]),
        authority: null,
      },
    ],
  });

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwned.publicKey,
  });
});

test('it cannot transfer an asset with royalties to a denylisted program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Creating a new asset to transfer.
  const asset = await createAsset(umi, {
    plugins: [
      {
        plugin: plugin('Royalties', [
          {
            percentage: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
          },
        ]),
        authority: null,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  const result = transfer(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});
