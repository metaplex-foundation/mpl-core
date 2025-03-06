import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import {
  PluginType,
  fetchAssetV1,
  removePluginV1,
  pluginAuthorityPair,
  removeCollectionPluginV1,
  fetchCollectionV1,
  ruleSet,
  addressPluginAuthority,
  nonePluginAuthority,
  transferV1,
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
} from './_setupRaw';

test('it can remove a plugin from an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetV1(umi, asset.publicKey);

  t.is(asset2.freezeDelegate, undefined);
});

test('it cannot remove an owner plugin from an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const attacker = generateSigner(umi);
  const asset = await createAsset(umi, {
    owner: umi.identity,
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
});

test('it can remove authority managed plugin from collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
      }),
    ],
  });

  await removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
  }).sendAndConfirm(umi);

  const collection2 = await fetchCollectionV1(umi, collection.publicKey);

  t.is(collection2.updateDelegate, undefined);
});

test('it can remove authority managed plugin from asset in collection using update auth', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const updateAuth = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            basisPoints: 5,
            ruleSet: ruleSet('None'),
          },
        }),
      ],
      authority: updateAuth,
    },
    {
      updateAuthority: updateAuth,
    }
  );

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: updateAuth.publicKey,
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Royalties,
    authority: updateAuth,
    payer: umi.identity,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetV1(umi, asset.publicKey);

  t.is(asset2.royalties, undefined);
});

test('it can remove authority managed plugin from asset not in collection using update auth', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Attributes,
    authority: assetAuth,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: undefined,
  });
});

test('it can remove authority managed plugin from collection using delegate auth', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            creators: [{ address: umi.identity.publicKey, percentage: 100 }],
            basisPoints: 5,
            ruleSet: ruleSet('None'),
          },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Royalties,
    authority: delegate,
    // We provide the payer because an account with 0 lamports cannot receive small SOL payments.
    payer: umi.identity,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetV1(umi, asset.publicKey);

  t.is(asset2.royalties, undefined);
});

test('it cannot remove owner managed plugin if the delegate authority is not owner', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const asset = await createAsset(umi, {
    owner: umi.identity,
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    authority: delegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: false,
    },
  });
});

test('it cannot remove authority managed plugin if the delegate authority is not update authority', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        authority: addressPluginAuthority(delegate.publicKey),
        data: {
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          basisPoints: 5,
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Royalties,
    authority: delegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    royalties: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      basisPoints: 5,
      ruleSet: ruleSet('None'),
    },
  });
});

test('it cannot remove authority managed collection plugin if the delegate authority is not update authority', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        authority: addressPluginAuthority(delegate.publicKey),
        data: {
          creators: [{ address: umi.identity.publicKey, percentage: 100 }],
          basisPoints: 5,
          ruleSet: ruleSet('None'),
        },
      }),
    ],
  });

  const result = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.Royalties,
    authority: delegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    royalties: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      basisPoints: 5,
      ruleSet: ruleSet('None'),
    },
  });
});

test('it can remove a plugin from asset with existing plugins', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  await removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.BurnDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot remove a plugin from a frozen asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
      pluginAuthorityPair({
        type: 'TransferDelegate',
      }),
      pluginAuthorityPair({
        type: 'BurnDelegate',
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const result1 = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.BurnDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result1, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it cannot remove a plugin from an asset with a frozen collection', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'TransferDelegate',
        }),
        pluginAuthorityPair({
          type: 'BurnDelegate',
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: {
            frozen: true,
          },
        }),
      ],
    }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.TransferDelegate,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot use an invalid system program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const fakeSystemProgram = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const fakeLogWrapper = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.FreezeDelegate,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot use an invalid system program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const fakeSystemProgram = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await assertCollection(t, umi, {
    collection: collection.publicKey,
  });

  const result = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program for collections', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const fakeLogWrapper = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [pluginAuthorityPair({ type: 'UpdateDelegate' })],
  });

  await assertCollection(t, umi, {
    collection: collection.publicKey,
  });

  const result = removeCollectionPluginV1(umi, {
    collection: collection.publicKey,
    pluginType: PluginType.UpdateDelegate,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});

test('it cannot remove an authority managed plugin from an asset if not the authority', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Attributes,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });
});

test('it cannot use an invalid collection to remove a plugin on an asset', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
      }),
    ],
  });

  const collectionAuth = generateSigner(umi);
  const wrongCollection = await createCollection(umi, {
    updateAuthority: collectionAuth.publicKey,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    collection: wrongCollection.publicKey,
    pluginType: PluginType.Attributes,
    authority: collectionAuth,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'UpdateAuthority',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });
});

test('it cannot remove an authority managed plugin when the authority is None', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [{ key: 'key', value: 'value' }] },
        authority: nonePluginAuthority(),
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    attributes: {
      authority: {
        type: 'None',
      },
      attributeList: [{ key: 'key', value: 'value' }],
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    authority: assetAuth,
    pluginType: PluginType.Attributes,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it cannot remove an owner managed plugin when the authority is None', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: nonePluginAuthority(),
      }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    transferDelegate: {
      authority: {
        type: 'None',
      },
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.TransferDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});

test('it can remove an owner managed plugin from an asset when the authority is None, after transferring', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'TransferDelegate',
        authority: nonePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  removePluginV1(umi, {
    asset: asset.publicKey,
    authority: newOwner,
    pluginType: PluginType.TransferDelegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
  });
});

test('it cannot remove an authority managed plugin from an asset when the authority is None, after transferring', async (t) => {
  const umi = await createUmi();

  const assetAuth = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const asset = await createAsset(umi, {
    updateAuthority: assetAuth,
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentTransferDelegate',
        authority: nonePluginAuthority(),
      }),
    ],
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: assetAuth.publicKey },
    permanentTransferDelegate: {
      authority: {
        type: 'None',
      },
    },
  });

  const result = removePluginV1(umi, {
    asset: asset.publicKey,
    authority: newOwner,
    pluginType: PluginType.PermanentTransferDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});
