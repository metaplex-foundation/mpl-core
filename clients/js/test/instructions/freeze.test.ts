import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addressPluginAuthority,
  freezeAsset,
  pluginAuthorityPair,
  thawAsset,
} from '../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createAssetWithCollection,
  createUmi,
} from '../_setupRaw';

test('it can use the freeze helper to freeze an asset', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi);

  await freezeAsset(umi, {
    asset,
    delegate: delegate.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: true,
    },
  });
});

test('it can use the freeze helper to freeze an asset with the plugin defined', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  await freezeAsset(umi, {
    asset,
    delegate: delegate.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: true,
    },
  });
});

test('it can use the freeze helper to freeze an asset with the plugin delegated if unfrozen', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const delegate2 = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
        data: { frozen: false },
      }),
    ],
  });

  await freezeAsset(umi, {
    asset,
    delegate: delegate2.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate2.publicKey,
      },
      frozen: true,
    },
  });
});

test('it can use freeze to freeze asset in collection', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  await freezeAsset(umi, {
    asset,
    collection,
    delegate: delegate.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      frozen: true,
    },
  });
});

test('it cannot freeze a frozen asset', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  t.throws(
    () =>
      freezeAsset(umi, {
        asset,
        delegate: delegate.publicKey,
      }).sendAndConfirm(umi),
    { message: 'Cannot freeze: asset is already frozen' }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: true,
    },
  });
});

test('it cannot freeze a perma frozen asset in collection', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'PermanentFreezeDelegate',
          data: { frozen: true },
        }),
      ],
    }
  );

  t.throws(
    () =>
      freezeAsset(umi, {
        asset,
        collection,
        delegate: delegate.publicKey,
      }).sendAndConfirm(umi),
    { message: 'Cannot freeze: asset is already frozen' }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
  });
});

test('it can use thaw helper to thaw a frozen asset', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
        data: { frozen: true },
      }),
    ],
  });

  await thawAsset(umi, {
    asset,
    delegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it can use thaw helper to thaw a frozen asset in collection', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
        data: { frozen: true },
      }),
    ],
  });

  await thawAsset(umi, {
    asset,
    collection,
    delegate,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });
});

test('it cannot thaw an unfrozen asset', async (t) => {
  const umi = await createUmi();
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
        data: { frozen: false },
      }),
    ],
  });

  t.throws(
    () => {
      thawAsset(umi, {
        asset,
        delegate,
      }).sendAndConfirm(umi);
    },
    { message: 'Cannot thaw: asset is not frozen' }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
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
