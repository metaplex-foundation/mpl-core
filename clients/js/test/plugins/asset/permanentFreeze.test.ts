import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { addPlugin, plugin, transfer, updateAuthority, updatePlugin } from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it can freeze and unfreeze an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [{ plugin: plugin('PermanentFreeze', [{ frozen: true }]), authority: null }]
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
    plugins: [{ plugin: plugin('PermanentFreeze', [{ frozen: true }]), authority: null }]
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
    newOwner: newOwner.publicKey
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: "InvalidAuthority",
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
    name: "InvalidAuthority",
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });
});
