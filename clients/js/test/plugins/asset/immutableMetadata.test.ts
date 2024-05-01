import test from 'ava';
import {
  addPluginV1,
  createPlugin,
  pluginAuthorityPair,
  updateV1,
} from '../../../src';
import { assertAsset, createAsset, createUmi } from '../../_setup';

test('it can prevent the asset from metadata updating', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'ImmutableMetadata',
      }),
    ],
  });

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'bread',
    newUri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});

test('it can mutate its metadata unless ImmutableMetadata plugin is added', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 2',
    newUri: 'https://example.com/bread2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Test Bread 2',
    uri: 'https://example.com/bread2',
  });

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'ImmutableMetadata',
    }),
  }).sendAndConfirm(umi);

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Test Bread 3',
    newUri: 'https://example.com/bread3',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
