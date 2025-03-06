import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import { pluginAuthority, pluginAuthorityPair, transferV1 } from '../src';
import {
  assertAsset,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from './_setupRaw';

test('it can transfer an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot transfer an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot transfer asset in collection if no collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it can transfer asset in collection as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('it cannot transfer asset in collection with the wrong collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});
  const wrongCollection = await createCollection(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: wrongCollection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('authorities on owner-managed plugins are reset on transfer', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const freezeDelegate = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
        authority: pluginAuthority('Address', {
          address: freezeDelegate.publicKey,
        }),
      }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: freezeDelegate.publicKey,
      },
      offset: 119n,
      frozen: false,
    },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: {
        type: 'Owner',
      },
      offset: 119n,
      frozen: false,
    },
  });
});

test('authorities on permanent plugins should not be reset on transfer', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const freezeDelegate = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'PermanentFreezeDelegate',
        data: { frozen: false },
        authority: pluginAuthority('Address', {
          address: freezeDelegate.publicKey,
        }),
      }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeDelegate: {
      authority: {
        type: 'Address',
        address: freezeDelegate.publicKey,
      },
      offset: 119n,
      frozen: false,
    },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    permanentFreezeDelegate: {
      authority: {
        type: 'Address',
        address: freezeDelegate.publicKey,
      },
      offset: 119n,
      frozen: false,
    },
  });
});

test('authorities on authority-managed plugin should not be reset on transfer', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const delegate = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'Attributes',
        data: { attributeList: [] },
        authority: pluginAuthority('Address', { address: delegate.publicKey }),
      }),
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      offset: 119n,
      attributeList: [],
    },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    attributes: {
      authority: {
        type: 'Address',
        address: delegate.publicKey,
      },
      offset: 119n,
      attributeList: [],
    },
  });
});

test('it cannot transfer an asset using asset as authority', async (t) => {
  const umi = await createUmi();
  const myAsset = generateSigner(umi);

  const asset = await createAsset(umi, { asset: myAsset });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: generateSigner(umi).publicKey,
    authority: myAsset,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot use an invalid system program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const fakeSystemProgram = generateSigner(umi);

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});

test('it cannot use an invalid noop program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const fakeLogWrapper = generateSigner(umi);

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    logWrapper: fakeLogWrapper.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidLogWrapperProgram' });
});
