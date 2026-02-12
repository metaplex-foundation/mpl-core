import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { hasAssetUpdateAuthority } from '../../src';
import { pluginAuthorityPair } from '../../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  generateSignerWithSol,
} from '../_setup';

test('it throws when not matching asset and collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();

  const { asset } = await createAssetWithCollection(rpc, payer, {});

  const collection = await createCollection(rpc, payer, {
    updateAuthority: updateAuth.address,
  });

  t.throws(() =>
    hasAssetUpdateAuthority(updateAuth.address, asset.data, collection.data)
  );
});

test('it can detect correct basic asset update auth', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuth.address,
  });

  t.assert(hasAssetUpdateAuthority(updateAuth.address, asset.data));
  t.assert(!hasAssetUpdateAuthority(payer.address, asset.data));
});

test('it can detect correct update auth from collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      updateAuthority: updateAuth.address,
    }
  );

  t.assert(
    hasAssetUpdateAuthority(updateAuth.address, asset.data, collection.data)
  );
  t.assert(!hasAssetUpdateAuthority(payer.address, asset.data, collection.data));
});

test('it can detect correct update auth from asset update delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();
  const delegate = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    updateAuthority: updateAuth.address,
    plugins: [
      pluginAuthorityPair({
        type: 'UpdateDelegate',
        authority: { __kind: 'Address', address: delegate.address },
      }),
    ],
  });

  t.assert(hasAssetUpdateAuthority(delegate.address, asset.data));
  t.assert(!hasAssetUpdateAuthority(payer.address, asset.data));
  t.assert(hasAssetUpdateAuthority(updateAuth.address, asset.data));
});

test('it can detect correct update auth from collection update delegate', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const updateAuth = await generateKeyPairSigner();
  const delegate = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      updateAuthority: updateAuth.address,
      plugins: [
        pluginAuthorityPair({
          type: 'UpdateDelegate',
          authority: { __kind: 'Address', address: delegate.address },
        }),
      ],
    }
  );

  t.assert(
    hasAssetUpdateAuthority(delegate.address, asset.data, collection.data)
  );
  t.assert(
    !hasAssetUpdateAuthority(payer.address, asset.data, collection.data)
  );
  t.assert(
    hasAssetUpdateAuthority(updateAuth.address, asset.data, collection.data)
  );
});
