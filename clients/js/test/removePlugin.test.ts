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
} from '../src';
import {
  DEFAULT_COLLECTION,
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from './_setup';

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

test('it can remove authority managed plugin from asset using update auth', async (t) => {
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
