import test from 'ava';
import { publicKey } from '@metaplex-foundation/umi';
import { das } from '../../src';
import { createUmiWithDas, DAS_API_ENDPOINT } from '../_setup';
import {
  dasTestAsset1Owner,
  dasTestAsset1,
  dasTestAsset2UpdateAuthority,
  dasTestAsset2,
  dasTestCollection1,
  dasTestCollection2UpdateAuthority,
  dasTestCollection2,
  dasTestAssetInCollection,
  dasTestCollection1PubKey,
  dasTestAsset1PubKey,
  dasTestAsset2PubKey,
  dasTestAssetInCollectionPubKey,
  dasTestCollection2PubKey,
} from './_setup';

test('das: it can search assets', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then assets are fetched via the MPL Core DAS helper
  const assets = await das.searchAssets(umi, {
    owner: dasTestAsset1Owner,
    interface: 'MplCoreAsset',
  });

  // The fetched assets structure is the same as with the MPL Core fetchAssetV1
  const asset = assets.find((a) => a.publicKey === dasTestAsset1PubKey);
  t.like(asset, dasTestAsset1);
});

test('das: it can fetch assets by owner', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then assets are fetched via the MPL Core DAS helper
  const assets = await das.fetchAssetsByOwner(umi, {
    owner: dasTestAsset1Owner,
  });

  // The fetched assets structure is the same as with the MPL Core fetchAssetV1
  const asset = assets.find((a) => a.publicKey === dasTestAsset1PubKey);
  t.like(asset, dasTestAsset1);
});

test('das: it can fetch assets by authority', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then assets are fetched via the MPL Core DAS helper
  const assets = await das.fetchAssetsByAuthority(umi, {
    authority: dasTestAsset1Owner,
  });

  // The fetched assets structure is the same as with the MPL Core fetchAssetV1
  const asset = assets.find((a) => a.publicKey === dasTestAsset1PubKey);
  t.like(asset, dasTestAsset1);
});

test('das: it can fetch assets by authority if authority is not owner', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then assets are fetched via the MPL Core DAS helper
  const assets = await das.fetchAssetsByAuthority(umi, {
    authority: dasTestAsset2UpdateAuthority,
  });

  // The fetched assets structure is the same as with the MPL Core fetchAssetV1
  const asset = assets.find((a) => a.publicKey === dasTestAsset2PubKey);
  t.like(asset, dasTestAsset2);
});

test('das: it can fetch assets by collection', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then assets are fetched via the MPL Core DAS helper
  const assets = await das.fetchAssetsByCollection(umi, {
    collection: dasTestCollection1PubKey,
  });

  // The fetched assets structure is the same as with the MPL Core fetchAssetV1
  const asset = assets.find(
    (a) => a.publicKey === dasTestAssetInCollectionPubKey
  );
  t.like(asset, dasTestAssetInCollection);
});

test('das: it can fetch collections by update authority', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then collections are fetched via the MPL Core DAS helper
  const collections = await das.fetchCollectionsByUpdateAuthority(umi, {
    updateAuthority: publicKey(dasTestAsset1Owner),
  });

  // One of the fetched collections structure is the same as with the MPL Core fetchCollectionV1
  const collection = collections.find(
    (a) => a.publicKey === dasTestCollection1PubKey
  );
  t.like(collection, dasTestCollection1);
});

test('das: it can fetch collections by update authority if update authority is not owner', async (t) => {
  // Given an Umi instance with DAS API
  const umi = createUmiWithDas(DAS_API_ENDPOINT);

  // Then collections are fetched via the MPL Core DAS helper
  const collections = await das.fetchCollectionsByUpdateAuthority(umi, {
    updateAuthority: publicKey(dasTestCollection2UpdateAuthority),
  });

  // One of the fetched collections structure is the same as with the MPL Core fetchCollectionV1
  const collection = collections.find(
    (a) => a.publicKey === dasTestCollection2PubKey
  );
  t.like(collection, dasTestCollection2);
});
