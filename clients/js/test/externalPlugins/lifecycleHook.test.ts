import test from 'ava';

import { mplCoreOracleExample } from '@metaplex-foundation/mpl-core-oracle-example';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  assertAsset,
  createUmi as baseCreateUmi,
  DEFAULT_ASSET,
} from '../_setupRaw';
import { createAsset } from '../_setupSdk';
import { addPlugin, updatePlugin } from '../../src';

const createUmi = async () =>
  (await baseCreateUmi()).use(mplCoreOracleExample());

test('it cannot create asset with lifecycle hook that has no lifecycle checks', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  // Lifecycle hook with no lifecycle checks
  const result = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'LifecycleHook',
        hookedProgram: account.publicKey,
        lifecycleChecks: {},
      },
    ],
  });

  await t.throwsAsync(result, { name: 'RequiresLifecycleCheck' });
});

test('it cannot add lifecycle hook with no lifecycle checks to asset', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  // Oracle with no lifecycle checks
  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'LifecycleHook',
      hookedProgram: account.publicKey,
      lifecycleChecks: {},
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'RequiresLifecycleCheck' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot update lifecycle hook to have no lifecycle checks', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  // Oracle with no lifecycle checks
  const result = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'LifecycleHook',
      hookedProgram: account.publicKey,
      lifecycleChecks: {},
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'RequiresLifecycleCheck' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});
