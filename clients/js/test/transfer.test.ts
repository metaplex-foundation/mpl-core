import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';

import {
  transfer,
  updateAuthority,
} from '../src';
import { assertAsset, createAsset, createUmi } from './_setup';

test('it can transfer an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi)
  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });

  await transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });
});

test('it cannot transfer an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi)

  const result = transfer(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  });
});
