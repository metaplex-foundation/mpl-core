import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addPlugin,
  plugin,
  pluginAuthorityPair,
  transfer,
  updateAuthority,
  updateCollectionPlugin,
  updatePlugin,
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
} from '../../_setup';

test('it can freeze and unfreeze an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({ type: 'PermanentFreeze', data: { frozen: true } }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('PermanentFreeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot be transferred while frozen', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({ type: 'PermanentFreeze', data: { frozen: true } }),
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  const result = transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it cannot add permanentFreeze after creation', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, { owner });

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('PermanentFreeze', [{ frozen: true }]),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });
});

test('it can add permanent freeze to collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'PermanentFreeze', data: { frozen: true } }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });
});

test('it can freeze and unfreeze a collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'PermanentFreeze', data: { frozen: true } }),
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: plugin('PermanentFreeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: false,
    },
  });
});

test('it cannot move asset in a permanently frozen collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreeze',
          data: { frozen: true },
        }),
      ],
    }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  const result = transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });
});

test('it can move asset with permanent freeze override in a frozen collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreeze',
          data: { frozen: false },
        }),
      ],
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreeze',
          data: { frozen: true },
        }),
      ],
    }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: umi.identity,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: updateAuthority('Collection', [collection.publicKey]),
  });
});

test('it can remove a permanent freeze plugin from an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({ type: 'PermanentFreeze', data: { frozen: true } }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    permanentFreeze: {
      authority: {
        type: 'UpdateAuthority',
      },
      frozen: true,
    },
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('PermanentFreeze', [{ frozen: false }]),
  }).sendAndConfirm(umi);

  const asset2 = await createAsset(umi, { owner: umi.identity });

  t.is(asset2.permanentFreeze, undefined);
});
