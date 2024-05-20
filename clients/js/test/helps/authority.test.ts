import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  addressPluginAuthority,
  hasAssetUpdateAuthority,
  pluginAuthorityPair,
} from '../../src';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
} from '../_setupRaw';

test('it throws when not matching asset and collection', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {
    owner,
  });

  const collection = await createCollection(umi, {
    updateAuthority: updateAuth.publicKey,
  });

  t.throws(() =>
    hasAssetUpdateAuthority(updateAuth.publicKey, asset, collection)
  );
});

test('it can detect correct basic asset update auth', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    updateAuthority: updateAuth.publicKey,
  });

  t.assert(hasAssetUpdateAuthority(updateAuth.publicKey, asset));
  t.assert(!hasAssetUpdateAuthority(owner.publicKey, asset));
});

test('it can detect correct update auth from collection', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      authority: updateAuth,
      owner,
    },
    {
      updateAuthority: updateAuth.publicKey,
    }
  );

  t.assert(hasAssetUpdateAuthority(updateAuth.publicKey, asset, collection));
  t.assert(!hasAssetUpdateAuthority(owner.publicKey, asset, collection));
});

test('it can detect correct update auth from asset update delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    updateAuthority: updateAuth.publicKey,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        authority: addressPluginAuthority(delegate.publicKey),
      }),
    ],
  });

  t.assert(hasAssetUpdateAuthority(delegate.publicKey, asset));
  t.assert(!hasAssetUpdateAuthority(owner.publicKey, asset));
  t.assert(hasAssetUpdateAuthority(updateAuth.publicKey, asset));
});

test('it can detect correct update auth from collection update delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const updateAuth = generateSigner(umi);

  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      authority: updateAuth,
      owner,
    },
    {
      updateAuthority: updateAuth.publicKey,
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: addressPluginAuthority(delegate.publicKey),
        }),
      ],
    }
  );

  t.assert(hasAssetUpdateAuthority(delegate.publicKey, asset, collection));
  t.assert(!hasAssetUpdateAuthority(owner.publicKey, asset, collection));
  t.assert(hasAssetUpdateAuthority(updateAuth.publicKey, asset, collection));
});
