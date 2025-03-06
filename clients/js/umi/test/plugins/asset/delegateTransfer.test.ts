import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  approvePluginAuthority,
  transfer,
  revokePluginAuthority,
} from '../../../src';
import { DEFAULT_ASSET, assertAsset, createUmi } from '../../_setupRaw';
import { createAsset, createCollection } from '../../_setupSdk';

test('a delegate can transfer the asset', async (t) => {
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [{ type: 'TransferDelegate' }],
  });

  await approvePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'TransferDelegate',
    },
    newAuthority: {
      type: 'Address',
      address: delegateAddress.publicKey,
    },
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwnerAddress.publicKey,
    authority: delegateAddress,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwnerAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('owner can transfer asset with delegate transfer', async (t) => {
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'TransferDelegate',
        authority: {
          type: 'Address',
          address: delegateAddress.publicKey,
        },
      },
    ],
  });

  await transfer(umi, {
    asset,
    newOwner: newOwnerAddress.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwnerAddress.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can revoke a delegate transfer plugin', async (t) => {
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'TransferDelegate',
        authority: {
          type: 'Address',
          address: delegateAddress.publicKey,
        },
      },
    ],
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'TransferDelegate',
    },
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

test('it cannot transfer after delegate authority has been revoked', async (t) => {
  const umi = await createUmi();
  const delegateAddress = generateSigner(umi);
  const newOwnerAddress = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'TransferDelegate',
        authority: {
          type: 'Address',
          address: delegateAddress.publicKey,
        },
      },
    ],
  });

  await revokePluginAuthority(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'TransferDelegate',
    },
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset,
    newOwner: newOwnerAddress.publicKey,
    authority: delegateAddress,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

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

test('it can transfer using delegated update authority', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    updateAuthority: updateAuthority.publicKey,
    plugins: [
      {
        type: 'TransferDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
    authority: updateAuthority,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuthority.publicKey },
    transferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  // Resets to `Owner` after the transfer.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: updateAuthority.publicKey },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});

test('it can transfer using delegated update authority from collection', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const updateAuthority = generateSigner(umi);

  const collection = await createCollection(umi, {
    updateAuthority: updateAuthority.publicKey,
  });

  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    collection: collection.publicKey,
    plugins: [
      {
        type: 'TransferDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
    authority: updateAuthority,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: { type: 'Collection' },
    transferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await transfer(umi, {
    asset,
    collection,
    newOwner: newOwner.publicKey,
    authority: updateAuthority,
  }).sendAndConfirm(umi);

  // Resets to `Owner` after the transfer.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection' },
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
  });
});
