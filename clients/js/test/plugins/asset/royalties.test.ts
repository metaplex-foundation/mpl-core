import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  SPL_SYSTEM_PROGRAM_ID,
  SPL_TOKEN_PROGRAM_ID,
} from '@metaplex-foundation/mpl-toolbox';
import {
  MPL_CORE_PROGRAM_ID,
  addPluginV1,
  createPlugin,
  pluginAuthorityPair,
  ruleSet,
  transferV1,
  updatePluginV1,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createUmi,
} from '../../_setupRaw';

test('it can transfer an asset with royalties', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
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
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },

    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
  });

  await transferV1(umi, {
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
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('None'),
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
  });

  await transferV1(umi, {
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
          basisPoints: 5,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('ProgramAllowList', [
            [SPL_SYSTEM_PROGRAM_ID, MPL_CORE_PROGRAM_ID],
          ]),
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
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [
        [SPL_SYSTEM_PROGRAM_ID, MPL_CORE_PROGRAM_ID],
      ]),
    },
  });

  await transferV1(umi, {
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
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramAllowList', [
              [SPL_SYSTEM_PROGRAM_ID, MPL_CORE_PROGRAM_ID],
            ]),
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [
        [SPL_SYSTEM_PROGRAM_ID, MPL_CORE_PROGRAM_ID],
      ]),
    },
  });

  await transferV1(umi, {
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

  // Creating a new asset to transfer.
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
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
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
    },
  });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot transfer an asset with collection royalties to a program address not on allowlist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const programOwner = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner: programOwner.publicKey,
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: programOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramAllowList', [[SPL_SYSTEM_PROGRAM_ID]]),
    },
  });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: programOwned.publicKey,
    authority: programOwner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: programOwner.publicKey,
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
          basisPoints: 5,
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
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
    },
  });

  await transferV1(umi, {
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
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramDenyList', [[SPL_TOKEN_PROGRAM_ID]]),
    },
  });

  await transferV1(umi, {
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
          basisPoints: 5,
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
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: programOwned.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot transfer an asset with collection royalties to a program address on the denylist', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('ProgramDenyList', [[MPL_CORE_PROGRAM_ID]]),
    },
  });

  const result = transferV1(umi, {
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

test('it cannot create royalty percentages that dont add up to 100', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const creator1 = generateSigner(umi);
  const creator2 = generateSigner(umi);

  const result = createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [
            { address: creator1.publicKey, percentage: 20 },
            { address: creator2.publicKey, percentage: 20 },
          ],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});
test('it cannot create royalty basis points greater than 10000', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const result = createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 10001,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});

test('it cannot create royalty with duplicate creators', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const result = createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 10001,
          creators: [
            {
              address: umi.identity.publicKey,
              percentage: 10,
            },
            {
              address: umi.identity.publicKey,
              percentage: 90,
            },
          ],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});

test('it cannot add royalty percentages that dont add up to 100', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {});
  const creator1 = generateSigner(umi);
  const creator2 = generateSigner(umi);

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          { address: creator1.publicKey, percentage: 20 },
          { address: creator2.publicKey, percentage: 20 },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});

test('it cannot add royalty percentages that has duplicate creators', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {});
  const creator1 = generateSigner(umi);

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          { address: creator1.publicKey, percentage: 20 },
          { address: creator1.publicKey, percentage: 80 },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});
test('it cannot add royalty basis points greater than 10000', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const asset = await createAsset(umi, {});

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 10001,
        creators: [{ address: umi.identity.publicKey, percentage: 100 }],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});

test('it cannot update royalty percentages that do not add up to 100', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const creator1 = generateSigner(umi);
  const creator2 = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [
            { address: creator1.publicKey, percentage: 20 },
            { address: creator2.publicKey, percentage: 80 },
          ],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          { address: creator1.publicKey, percentage: 20 },
          { address: creator2.publicKey, percentage: 20 },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});
test('it cannot update royalty basis points greater than 10000', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 100,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 10001,
        creators: [{ address: umi.identity.publicKey, percentage: 100 }],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});

test('it cannot update royalty with duplicate creators', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 100,
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  const result = updatePluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 10001,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 10,
          },
          {
            address: umi.identity.publicKey,
            percentage: 90,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginSetting' });
});
