import test from 'ava';
import { addPlugin, plugin, updateAuthority, updatePlugin } from '../../../src';
import {
  DEFAULT_ASSET,
  assertAsset,
  createAsset,
  createUmi,
} from '../../_setup';

test('it can freeze and unfreeze an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('Freeze', [{ frozen: true }]),
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
      frozen: true,
    },
  });

  await updatePlugin(umi, {
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
