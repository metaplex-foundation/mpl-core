import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  addCollectionPlugin,
  addPlugin,
  authority,
  plugin,
  pluginAuthorityPair,
  ruleSet,
  updateAuthority,
} from '../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from './_setup';

test('it can add a plugin to an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);

  // Then an account was created with the correct data.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can add an authority managed plugin to an asset via update auth', async (t) => {
  const umi = await createUmi();
  const updateAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: updateAuth,
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('UpdateDelegate', [{ frozen: false }]),
    authority: updateAuth,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [updateAuth.publicKey]),
    updateDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can add a plugin to an asset with a different authority than the default', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
    initAuthority: authority('Pubkey', { address: delegateAddress.publicKey }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Pubkey',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  });
});

test('it can add plugin to asset with a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Freeze',
        data: { frozen: false },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Transfer', [{}]),
    initAuthority: authority('Pubkey', { address: delegate.publicKey }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    freeze: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transfer: {
      authority: {
        type: 'Pubkey',
        address: delegate.publicKey,
      },
    },
  });
});

test('it can add a plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: plugin('Royalties', [
      {
        percentage: 5,
        creators: [],
        ruleSet: ruleSet('None'),
      },
    ]),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it cannot add an owner-managed plugin to a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: plugin('Freeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add an authority-managed plugin to an asset via delegate authority', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: authority('Pubkey', { address: delegate.publicKey }),
        }),
      ],
    }
  );

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: plugin('Royalties', [
      {
        percentage: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    ]),
    authority: delegate,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it can add an authority-managed plugin to an asset with the collection update authority', async (t) => {
  const umi = await createUmi();
  const collectionAuth = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    { authority: collectionAuth },
    { updateAuthority: collectionAuth, }
  );

  await addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: plugin('Royalties', [
      {
        percentage: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    ]),
    authority: collectionAuth,
    payer: umi.identity,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
  });
});

test('it cannot add a authority-managed plugin to an asset via delegate authority with the wrong authority', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: authority('Pubkey', { address: delegate.publicKey }),
        }),
      ],
    }
  );

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    plugin: plugin('Royalties', [
      {
        percentage: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    ]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it cannot add authority-managed plugin to an asset by owner', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(umi, { owner });

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    collection: collection.publicKey,
    authority: owner,
    plugin: plugin('Royalties', [
      {
        percentage: 5,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    ]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can add a plugin to a collection with a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          percentage: 5,
          creators: [
            {
              address: umi.identity.publicKey,
              percentage: 100,
            },
          ],
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
  });

  await addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: plugin('UpdateDelegate', [{}]),
    initAuthority: authority('Pubkey', { address: delegate.publicKey }),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      percentage: 5,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: ruleSet('None'),
    },
    updateDelegate: {
      authority: {
        type: 'Pubkey',
        address: delegate.publicKey,
      },
    },
  });
});
