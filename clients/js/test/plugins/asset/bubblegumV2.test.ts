import test from 'ava';
import { createPlugin, addPluginV1 } from '../../../src';
import { createUmi } from '../../_setupRaw';
import { createAsset } from '../../_setupSdk';

test('it cannot create asset with BubblegumV2 plugin', async (t) => {
  const umi = await createUmi();
  const result = createAsset(umi, {
    plugins: [
      {
        type: 'BubblegumV2',
      },
    ],
  });

  await t.throwsAsync(result, {
    name: 'InvalidPlugin',
  });
});

test('it cannot add BubblegumV2 to asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  const result = addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({
      type: 'BubblegumV2',
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });
});
