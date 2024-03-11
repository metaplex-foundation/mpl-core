import test from 'ava';

import {
  PluginType,
  fetchAssetWithPlugins,
  removePlugin,
  plugin,
} from '../src';
import { assertAsset, createAsset, createUmi } from './_setup';

test('it can remove a plugin from an asset', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      { plugin: plugin('Freeze', [{ frozen: false }]), authority: null },
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    freeze: {
      authority: {
        type: 'Owner',
      },
      frozen: false,
    },
  });

  await removePlugin(umi, {
    asset: asset.publicKey,
    pluginType: PluginType.Freeze,
  }).sendAndConfirm(umi);

  const asset2 = await fetchAssetWithPlugins(umi, asset.publicKey);

  t.is(asset2.freeze, undefined);
});
