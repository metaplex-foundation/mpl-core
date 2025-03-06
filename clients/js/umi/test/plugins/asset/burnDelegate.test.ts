import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { addPlugin, revokePluginAuthority, burn } from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createUmi,
  assertBurned,
} from '../../_setupRaw';
import { createAsset, createCollection } from '../../_setupSdk';

test('it can create an asset with burnDelegate', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'BurnDelegate',
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can add burnDelegate to an asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {});

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: undefined,
  });

  const burnDelegate = generateSigner(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'BurnDelegate',
      authority: {
        type: 'Address',
        address: burnDelegate.publicKey,
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: burnDelegate.publicKey,
      },
    },
  });
});

test('a burnDelegate can burn an asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {});
  const burnDelegate = generateSigner(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'BurnDelegate',
      authority: {
        type: 'Address',
        address: burnDelegate.publicKey,
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: burnDelegate.publicKey,
      },
    },
  });

  await burn(umi, {
    asset,
    authority: burnDelegate,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('an burnDelegate cannot burn an asset after delegate authority revoked', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {});
  const burnDelegate = generateSigner(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'BurnDelegate',
      authority: {
        type: 'Address',
        address: burnDelegate.publicKey,
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Address',
        address: burnDelegate.publicKey,
      },
    },
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'BurnDelegate',
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });

  const result = burn(umi, {
    asset,
    authority: burnDelegate,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    burnDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('a burnDelegate can burn using delegated update authority', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);

  const asset = await createAsset(umi, {
    updateAuthority: updateAuthority.publicKey,
    owner: owner.publicKey,
    plugins: [
      {
        type: 'BurnDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuthority.publicKey },
    burnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await burn(umi, {
    asset,
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('a burnDelegate can burn using delegated update authority from collection', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);

  const collection = await createCollection(umi, {
    updateAuthority: updateAuthority.publicKey,
  });

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    collection: collection.publicKey,
    plugins: [
      {
        type: 'BurnDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
    authority: updateAuthority,
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    burnDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await burn(umi, {
    asset,
    collection,
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});
