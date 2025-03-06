import test from 'ava';

import { mplCoreOracleExample } from '@metaplex-foundation/mpl-core-oracle-example';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  assertAsset,
  createUmi as baseCreateUmi,
  DEFAULT_ASSET,
} from '../_setupRaw';
import { createAsset } from '../_setupSdk';
import { addPlugin, CheckResult, updatePlugin } from '../../src';

const createUmi = async () =>
  (await baseCreateUmi()).use(mplCoreOracleExample());

test.skip('it cannot create asset with lifecycle hook that has no lifecycle checks', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

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

test.skip('it cannot add lifecycle hook with no lifecycle checks to asset', async (t) => {
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

test.skip('it cannot update lifecycle hook to have no lifecycle checks', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'LifecycleHook',
        hookedProgram: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  const result = updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      key: {
        type: 'LifecycleHook',
        hookedProgram: account.publicKey,
      },
      type: 'LifecycleHook',
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
