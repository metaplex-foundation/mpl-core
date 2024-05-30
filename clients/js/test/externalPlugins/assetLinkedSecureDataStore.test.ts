import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  assertAsset,
  assertCollection,
  createUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
} from '../_setupRaw';
import { createAsset, createAssetWithCollection } from '../_setupSdk';
import { ExternalPluginAdapterSchema, writeData } from '../../src';

test('it can create an asset linked secure store on a collection', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        {
          type: 'AssetLinkedSecureDataStore',
          dataAuthority: {
            type: 'UpdateAuthority',
          },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    }
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    assetLinkedSecureDataStores: [
      {
        type: 'AssetLinkedSecureDataStore',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot add asset linked secure store to an asset', async (t) => {
  const umi = await createUmi();

  const res = createAsset(umi, {
    plugins: [
      {
        type: 'AssetLinkedSecureDataStore',
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await t.throwsAsync(res, { name: 'InvalidPluginAdapterTarget' });
});

test('it can write data to an asset linked secure store', async (t) => {
  const umi = await createUmi();
  const dataAuthority = await generateSignerWithSol(umi);
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        {
          type: 'AssetLinkedSecureDataStore',
          dataAuthority: {
            type: 'Address',
            address: dataAuthority.publicKey,
          },
          schema: ExternalPluginAdapterSchema.Json,
        },
      ],
    }
  );

  await writeData(umi, {
    key: {
      type: 'AssetLinkedSecureDataStore',
      dataAuthority: {
        type: 'Address',
        address: dataAuthority.publicKey,
      },
    },
    collection: collection.publicKey,
    data: new TextEncoder().encode(JSON.stringify({ hello: 'world' })),
    asset: asset.publicKey,
    authority: dataAuthority,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    dataSections: [
      {
        type: 'DataSection',
        parentKey: {
          type: 'AssetLinkedSecureDataStore',
          dataAuthority: {
            type: 'Address',
            address: dataAuthority.publicKey,
          },
        },
        authority: { type: 'None' },
        data: { hello: 'world' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  // check the derived asset sdk correctly injects the data
  await assertAsset(
    t,
    umi,
    {
      ...DEFAULT_ASSET,
      asset: asset.publicKey,
      owner: umi.identity.publicKey,
      assetLinkedSecureDataStores: [
        {
          type: 'AssetLinkedSecureDataStore',
          authority: { type: 'UpdateAuthority' },
          dataAuthority: {
            type: 'Address',
            address: dataAuthority.publicKey,
          },
          schema: ExternalPluginAdapterSchema.Json,
          data: { hello: 'world' },
        },
      ],

      dataSections: [
        {
          type: 'DataSection',
          parentKey: {
            type: 'AssetLinkedSecureDataStore',
            dataAuthority: {
              type: 'Address',
              address: dataAuthority.publicKey,
            },
          },
          authority: { type: 'None' },
          data: { hello: 'world' },
          schema: ExternalPluginAdapterSchema.Json,
        },
      ],
    },
    {
      derivePlugins: true,
    }
  );
});
