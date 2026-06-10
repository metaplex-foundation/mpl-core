import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  fetchAssetsByOwner,
  fetchAssetsByCollection,
  fetchAssetsByUpdateAuthority,
} from '../src';
import {
  createAsset,
  createCollection,
  createRpc,
  generateSignerWithSol,
} from './_setup';

test('it can gpa fetch assets by owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  await createAsset(rpc, payer, {
    name: 'asset1',
    owner: owner.address,
  });
  await createAsset(rpc, payer, {
    name: 'asset2',
    owner: owner.address,
  });
  await createAsset(rpc, payer, {});

  const assets = await fetchAssetsByOwner(rpc, owner.address);
  const names = ['asset1', 'asset2'];

  t.is(assets.length, 2);
  t.assert(assets.every((asset) => names.includes(asset.data.name)));
  t.assert(assets.every((asset) => asset.data.owner === owner.address));
});

test('it can gpa fetch assets by collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await createCollection(rpc, payer);

  await createAsset(rpc, payer, {
    name: 'asset1',
    collection: collection.address,
  });
  await createAsset(rpc, payer, {
    name: 'asset2',
    collection: collection.address,
  });
  await createAsset(rpc, payer, {});

  const assets = await fetchAssetsByCollection(rpc, collection.address);
  const names = ['asset1', 'asset2'];

  t.is(assets.length, 2);
  t.assert(assets.every((asset) => names.includes(asset.data.name)));
  assets.forEach((asset) => {
    t.deepEqual(asset.data.updateAuthority, {
      __kind: 'Collection',
      fields: [collection.address],
    });
  });
});

test('it can gpa fetch assets by updateAuthority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();

  await createAsset(rpc, payer, {
    name: 'asset1',
    updateAuthority: authority.address,
  });
  await createAsset(rpc, payer, {
    name: 'asset2',
    updateAuthority: authority.address,
  });
  await createAsset(rpc, payer, {});

  const assets = await fetchAssetsByUpdateAuthority(rpc, authority.address);
  const names = ['asset1', 'asset2'];

  t.is(assets.length, 2);
  t.assert(assets.every((asset) => names.includes(asset.data.name)));
  assets.forEach((asset) => {
    t.deepEqual(asset.data.updateAuthority, {
      __kind: 'Address',
      fields: [authority.address],
    });
  });
});
