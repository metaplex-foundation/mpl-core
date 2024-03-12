import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  SPL_SYSTEM_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
} from '@metaplex-foundation/mpl-toolbox';
import {
  MPL_CORE_PROGRAM_ID,
  pluginAuthorityPair,
  ruleSet,
  transfer,
  updateAuthority,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createUmi,
} from '../../_setup';

test('it can transfer an asset with royalties', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('None'),
        },
      }),
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

test('it can transfer an asset with collection royalties', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
  })

  await transfer(umi, {
    collection: collection.publicKey,
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
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [[MPL_CORE_PROGRAM_ID]]),
        },
      }),
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
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwned.publicKey,
  });
});

test('it can transfer an asset with collection royalties to an allowlisted program address', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [[MPL_CORE_PROGRAM_ID]]),
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [[MPL_CORE_PROGRAM_ID]]),
    },
  })

  await transfer(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: programOwned.publicKey,
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
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
        },
      }),
    ],
  });

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
    }
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

test('it cannot transfer an asset with collection royalties to a program address not on allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const programOwner = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(umi, {
    owner: programOwner.publicKey,
  }, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);
  // Create a second one because allowlist needs both to be off the allowlist.
  const programOwned2 = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: programOwner.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
    },
  })

  await transfer(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: programOwned2.publicKey,
    authority: programOwner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwned.publicKey,
  });
});

test('it can transfer an asset with royalties to a program address not on the denylist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Creating a new asset to transfer.
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
        },
      }),
    ],
  });

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
      ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
    }
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


test('it can transfer an asset with collection royalties to a program address not on the denylist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
    },
  })

  await transfer(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
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
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
        },
      }),
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

test('it cannot transfer an asset with collection royalties to a program address on the denylist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {}, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
    },
  })

  const result = transfer(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
  });
});