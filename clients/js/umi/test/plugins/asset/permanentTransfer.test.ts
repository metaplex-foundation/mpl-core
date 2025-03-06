import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  PluginType,
  addPluginV1,
  createPlugin,
  pluginAuthorityPair,
  addressPluginAuthority,
  removePluginV1,
  transferV1,
  updatePluginAuthority,
} from '../../../src';
import {
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from '../../_setupRaw';

test('it cannot add permanentTransfer after creation', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, { owner });

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'PermanentTransferDelegate' }),
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: undefined,
  });
});

test('it can transfer an asset as the owner and not the delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: newOwner,
    asset: asset.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: brandNewOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer an asset as the delegate and the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer an asset as not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: brandNewOwner.publicKey,
    authority: umi.payer,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: brandNewOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer asset that is a part of a collection forever as a delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const collection = await createCollection(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
    collection: collection.publicKey,
  });

  await transferV1(umi, {
    authority: umi.payer,
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: umi.payer,
    asset: asset.publicKey,
    collection: collection.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: brandNewOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer multiple assets that is a part of a collection forever as a delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const firstAssetOwner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: updatePluginAuthority(),
      }),
    ],
  });

  const asset1 = await createAsset(umi, {
    owner: firstAssetOwner,
    collection: collection.publicKey,
  });

  const asset2 = await createAsset(umi, {
    owner: firstAssetOwner,
    collection: collection.publicKey,
  });

  // move asset #1 twice as a delegate for collection
  await transferV1(umi, {
    authority: umi.payer,
    asset: asset1.publicKey,
    collection: collection.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: umi.payer,
    asset: asset1.publicKey,
    collection: collection.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset1,
    asset: asset1.publicKey,
    owner: brandNewOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentTransferDelegate: undefined,
  });

  // move asset #2 twice as a delegate for collection
  await transferV1(umi, {
    authority: umi.payer,
    asset: asset2.publicKey,
    collection: collection.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await transferV1(umi, {
    authority: umi.payer,
    asset: asset2.publicKey,
    collection: collection.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset2,
    asset: asset2.publicKey,
    owner: brandNewOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentTransferDelegate: undefined,
  });

  await assertCollection(t, umi, {
    currentSize: 2,
    numMinted: 2,
    collection: collection.publicKey,
    updateAuthority: umi.payer.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can remove permanent transfer plugin if collection update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collectionAuth = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      authority: collectionAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: updatePluginAuthority(),
        }),
      ],
    },
    {
      updateAuthority: collectionAuth,
    }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.PermanentTransferDelegate,
    authority: collectionAuth,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    permanentTransferDelegate: undefined,
  });
});

test('it can permanent transfer using collection delegate authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: delegate,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
  });
});

test('it can permanent transfer asset that is frozen as a delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  await transferV1(umi, {
    authority: delegate,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it cannot transfer asset that is frozen with permanent transfer by owner', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const result = transferV1(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
    },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it can collection permanent transfer asset that is frozen as a collection delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  await transferV1(umi, {
    authority: delegate,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it can collection permanent transfer asset that is frozen as a collection update auth', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const colAuth = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      authority: colAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: true },
        }),
      ],
    },
    {
      updateAuthority: colAuth,
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentTransferDelegate',
          authority: updatePluginAuthority(),
        }),
      ],
    }
  );

  await transferV1(umi, {
    authority: colAuth,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it can add another plugin on asset with permanent transfer plugin', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
      }),
    ],
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'TransferDelegate',
    }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
