import { SPL_SYSTEM_PROGRAM_ID } from '@metaplex-foundation/mpl-toolbox';
import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { createCollection, createAsset } from '../_setupSdk';
import { createUmi, DEFAULT_ASSET } from '../_setupRaw';
import {
  addCollectionExternalPluginAdapterV1,
  addExternalPluginAdapterV1,
  createCollectionV2,
  createV2,
  ExternalPluginAdapterSchema,
} from '../../src';

test('it cannot create an asset with a DataSection', async (t) => {
  const umi = await createUmi();
  const asset = await generateSigner(umi);

  const result = createV2(umi, {
    asset,
    ...DEFAULT_ASSET,
    externalPluginAdapters: [
      {
        __kind: 'DataSection',
        fields: [
          {
            parentKey: {
              __kind: 'LinkedLifecycleHook',
              fields: [SPL_SYSTEM_PROGRAM_ID],
            },
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      },
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'CannotAddDataSection' });
});

test('it cannot create a collection with a DataSection', async (t) => {
  const umi = await createUmi();
  const collection = await generateSigner(umi);

  const result = createCollectionV2(umi, {
    collection,
    ...DEFAULT_ASSET,
    externalPluginAdapters: [
      {
        __kind: 'DataSection',
        fields: [
          {
            parentKey: {
              __kind: 'LinkedLifecycleHook',
              fields: [SPL_SYSTEM_PROGRAM_ID],
            },
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      },
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'CannotAddDataSection' });
});

test('it cannot add a DataSection to an asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi);

  const result = addExternalPluginAdapterV1(umi, {
    asset: asset.publicKey,
    initInfo: {
      __kind: 'DataSection',
      fields: [
        {
          parentKey: {
            __kind: 'LinkedLifecycleHook',
            fields: [SPL_SYSTEM_PROGRAM_ID],
          },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'CannotAddDataSection' });
});

test('it cannot add a DataSection to a collection', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi);

  const result = addCollectionExternalPluginAdapterV1(umi, {
    collection: collection.publicKey,
    initInfo: {
      __kind: 'DataSection',
      fields: [
        {
          parentKey: {
            __kind: 'LinkedLifecycleHook',
            fields: [SPL_SYSTEM_PROGRAM_ID],
          },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'CannotAddDataSection' });
});
