import { generateSigner, TransactionBuilder } from '@metaplex-foundation/umi';
import test from 'ava';

import { batch, create, transfer } from '../src';
import { assertAsset, createUmi, DEFAULT_ASSET } from './_setupRaw';
import { createAsset } from './_setupSdk';

test('it can batch create assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assets = [];

  let builder = new TransactionBuilder();

  for (let i = 0; i < 5; i += 1) {
    assets.push(generateSigner(umi));
    builder = builder.add(create(umi, { asset: assets[i], ...DEFAULT_ASSET }));
  }

  await batch(umi, builder).sendAndConfirm(umi);

  // eslint-disable-next-line no-restricted-syntax
  for (const asset of assets) {
    // eslint-disable-next-line no-await-in-loop
    await assertAsset(t, umi, {
      asset: asset.publicKey,
      owner: umi.identity.publicKey,
      updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    });
  }
});

test('it can batch transfer assets', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const assetSigner = generateSigner(umi);

  const asset = await createAsset(umi, { asset: assetSigner });

  let builder = new TransactionBuilder();

  for (let i = 0; i < 18; i += 1) {
    builder = builder.add(
      transfer(umi, { asset, newOwner: newOwner.publicKey })
    );
    builder = builder.add(
      transfer(umi, {
        asset,
        newOwner: umi.identity.publicKey,
        authority: newOwner,
      })
    );
  }

  await batch(umi, builder).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});
