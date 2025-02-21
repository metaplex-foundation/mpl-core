import test from 'ava';

import { generateSigner } from '@metaplex-foundation/umi';
import {
  fetchAssetsByCollection,
  fetchAssetsByOwner,
  fetchCollectionsByUpdateAuthority,
  fetchAllAssets,
} from '../../src';
import { createUmi } from '../_setupRaw';
import { createAsset, createCollection } from '../_setupSdk';

test('it can use the helper to fetch assets by owner', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const assets = await Promise.all(
    Array(5)
      .fill(0)
      .map(() => createAsset(umi, { owner: owner.publicKey }))
  );

  const fetchedAssets = await fetchAssetsByOwner(umi, owner.publicKey);

  t.is(fetchedAssets.length, assets.length);
  t.deepEqual(
    fetchedAssets.map((asset) => asset.publicKey).sort(),
    assets.map((asset) => asset.publicKey).sort()
  );
});

test('it can use helper to fetch assets by collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  const assets = await Promise.all(
    Array(5)
      .fill(0)
      .map(() => createAsset(umi, { collection }))
  );

  const fetchedAssets = await fetchAssetsByCollection(
    umi,
    collection.publicKey
  );

  t.is(fetchedAssets.length, assets.length);
  t.deepEqual(
    fetchedAssets.map((asset) => asset.publicKey).sort(),
    assets.map((asset) => asset.publicKey).sort()
  );
});

test('it can use helper to fetch collections by update authority', async (t) => {
  const umi = await createUmi();
  const updateAuthority = generateSigner(umi);

  const collections = await Promise.all(
    Array(5)
      .fill(0)
      .map(() =>
        createCollection(umi, { updateAuthority: updateAuthority.publicKey })
      )
  );

  const fetchedCollections = await fetchCollectionsByUpdateAuthority(
    umi,
    updateAuthority.publicKey
  );

  t.is(fetchedCollections.length, collections.length);
  t.deepEqual(
    fetchedCollections.map((collection) => collection.publicKey).sort(),
    collections.map((collection) => collection.publicKey).sort()
  );
});

test('it can use helper to fetch assets by collection and derive plugins', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'Attributes',
        attributeList: [
          {
            key: 'collection',
            value: 'col',
          },
        ],
      },
    ],
  });

  const override = await createAsset(umi, {
    collection,
    plugins: [
      {
        type: 'Attributes',
        attributeList: [
          {
            key: 'asset',
            value: 'asset',
          },
        ],
      },
    ],
  });

  const assets = await Promise.all(
    Array(4)
      .fill(0)
      .map(() =>
        createAsset(umi, {
          collection,
          plugins: [
            {
              type: 'FreezeDelegate',
              frozen: true,
            },
          ],
        })
      )
  );

  const fetchedAssets = await fetchAssetsByCollection(
    umi,
    collection.publicKey
  );

  t.is(fetchedAssets.length, assets.length + 1);

  fetchedAssets.forEach((asset) => {
    if (asset.publicKey === override.publicKey) {
      t.like(asset, {
        numMinted: undefined,
        currentSize: undefined,
        attributes: {
          attributeList: [
            {
              key: 'asset',
              value: 'asset',
            },
          ],
        },
      });
    } else {
      t.like(asset, {
        numMinted: undefined,
        currentSize: undefined,
        freezeDelegate: {
          frozen: true,
        },
        attributes: {
          attributeList: [
            {
              key: 'collection',
              value: 'col',
            },
          ],
        },
      });
    }
  });
});

test('it can use helper to fetch all assets', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'ImmutableMetadata',
      },
    ],
  });

  const assetsOfOwner1 = await Promise.all(
    Array(2)
      .fill(0)
      .map((_, index) =>
        createAsset(umi, {
          collection,
          name: `Asset ${index + 1}`,
          plugins: [
            {
              type: 'Attributes',
              attributeList: [
                {
                  key: 'asset',
                  value: 'asset',
                },
              ],
            },
          ],
        })
      )
  );

  const assetsOfOwner2 = await Promise.all(
    Array(2)
      .fill(0)
      .map((_, index) =>
        createAsset(umi, {
          collection,
          name: `Asset ${index + 1}`,
          plugins: [
            {
              type: 'Attributes',
              attributeList: [
                {
                  key: 'asset',
                  value: 'asset',
                },
              ],
            },
          ],
        })
      )
  );

  const allCreatedAssets = [...assetsOfOwner1, ...assetsOfOwner2];

  const assetPublicKeys = [...assetsOfOwner1, ...assetsOfOwner2].map(
    (asset) => asset.publicKey
  );

  const fetchedAssets = await fetchAllAssets(umi, assetPublicKeys);

  const createdAssetPubkeys = allCreatedAssets
    .map((asset) => asset.publicKey)
    .sort();
  const fetchedAssetPubkeys = fetchedAssets
    .map((asset) => asset.publicKey)
    .filter((pubkey) => createdAssetPubkeys.includes(pubkey))
    .sort();

  t.deepEqual(
    fetchedAssetPubkeys,
    createdAssetPubkeys,
    'All created assets should be found in fetched assets'
  );

  t.deepEqual(
    fetchedAssets[0].attributes,
    allCreatedAssets[0].attributes,
    'Asset level attribute plugin should be found in fetched assets'
  );

  t.deepEqual(
    fetchedAssets[0].immutableMetadata,
    collection.immutableMetadata,
    'Collection level immutableMetadata plugin should be found in fetched assets'
  );
});
