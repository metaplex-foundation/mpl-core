import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import { assertAsset, createUmi, DEFAULT_ASSET } from '../_setupRaw';
import { createAsset } from '../_setupSdk';
import { ExternalPluginAdapterSchema, writeData } from '../../src';

test('it can create a secure store', async (t) => {
  const umi = await createUmi();

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'SecureDataStore',
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    secureDataStores: [
      {
        type: 'SecureDataStore',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it can write data to a secure store', async (t) => {
  const umi = await createUmi();

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'SecureDataStore',
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    secureDataStores: [
      {
        type: 'SecureDataStore',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  const result = await writeData(umi, {
    key: {
      type: 'SecureDataStore',
      dataAuthority: {
        type: 'UpdateAuthority',
      },
    },
    data: Buffer.from('Hello'),
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  console.log(await umi.rpc.getTransaction(result.signature));

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    secureDataStores: [
      {
        type: 'SecureDataStore',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: {
          type: 'UpdateAuthority',
        },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it can add and write to secure data store on asset with data authority', async (t) => {
  const umi = await createUmi();
  const dataAuthority = generateSigner(umi);
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'SecureDataStore',
        dataAuthority: {
          type: 'Address',
          address: dataAuthority.publicKey,
        },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'SecureDataStore',
      dataAuthority: {
        type: 'Address',
        address: dataAuthority.publicKey,
      },
    },
    data: new TextEncoder().encode(JSON.stringify({ hello: 'world' })),
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    owner: umi.identity,
    asset: asset.publicKey,
    secureDataStores: [
      {
        type: 'SecureDataStore',
        authority: {
          type: 'UpdateAuthority',
        },
        data: { hello: 'world' },
        dataAuthority: {
          type: 'Address',
          address: dataAuthority.publicKey,
        },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});
