import test from 'ava';
import { assertAsset, createUmi, DEFAULT_ASSET } from '../_setupRaw';
import { createAsset } from '../_setupSdk';
import { ExternalPluginAdapterSchema, writeData } from '../../src';

test('it can create a secure store', async (t) => {
  const umi = await createUmi();

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
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

test('it can write data to a secure store', async (t) => {
  const umi = await createUmi();

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
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

  const result = await writeData(umi, {
    key: {
      type: 'AssetLinkedSecureDataStore',
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
