import test from 'ava';

import {
  mplCoreOracleExample,
  fixedAccountInit,
  fixedAccountSet,
  preconfiguredAssetPdaInit,
  MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
  preconfiguredAssetPdaSet,
  preconfiguredProgramPdaInit,
  preconfiguredProgramPdaSet,
  preconfiguredOwnerPdaInit,
  preconfiguredOwnerPdaSet,
  preconfiguredRecipientPdaInit,
  preconfiguredRecipientPdaSet,
  customPdaAllSeedsInit,
  customPdaAllSeedsSet,
  customPdaTypicalInit,
  customPdaTypicalSet,
  preconfiguredAssetPdaCustomOffsetInit,
  preconfiguredAssetPdaCustomOffsetSet,
  close,
} from '@metaplex-foundation/mpl-core-oracle-example';
import { generateSigner } from '@metaplex-foundation/umi';
import { ExternalValidationResult } from '@metaplex-foundation/mpl-core-oracle-example/dist/src/hooked';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  assertAsset,
  assertBurned,
  createUmi as baseCreateUmi,
  DEFAULT_ASSET,
} from '../_setupRaw';
import { createAsset, createAssetWithCollection } from '../_setupSdk';
import {
  burn,
  CheckResult,
  create,
  findOracleAccount,
  OracleInitInfoArgs,
  transfer,
  update,
  addPlugin,
  updatePlugin,
} from '../../src';

const createUmi = async () =>
  (await baseCreateUmi()).use(mplCoreOracleExample());

test('it can use fixed address oracle to deny update', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const result = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use fixed address oracle to deny update via collection', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const { asset, collection } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        {
          type: 'Oracle',
          resultsOffset: {
            type: 'Anchor',
          },
          lifecycleChecks: {
            update: [CheckResult.CAN_REJECT],
          },
          baseAddress: account.publicKey,
        },
      ],
    }
  );

  const result = update(umi, {
    asset,
    collection,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    collection,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use fixed address oracle to deny transfer', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it cannot create asset with oracle that has no lifecycle checks', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  // Oracle with no lifecycle checks
  const result = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {},
        baseAddress: account.publicKey,
      },
    ],
  });

  await t.throwsAsync(result, { name: 'RequiresLifecycleCheck' });
});

test('it cannot add oracle with no lifecycle checks to asset', async (t) => {
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
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {},
      baseAddress: account.publicKey,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'RequiresLifecycleCheck' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot update oracle to have no lifecycle checks', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  // Oracle with no lifecycle checks
  const result = updatePlugin(umi, {
    asset: asset.publicKey,

    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: account.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {},
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'RequiresLifecycleCheck' });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot create asset with oracle that can approve', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  // Oracle with `CheckResult.CAN_APPROVE`
  const result = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_APPROVE],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });
});

test('it cannot create asset with oracle that can approve in addition to reject', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  // Oracle with `CheckResult.CAN_APPROVE`
  const result = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_APPROVE, CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });
});

test('it cannot add oracle to asset that can approve', async (t) => {
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

  // Oracle with `CheckResult.CAN_APPROVE`
  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_APPROVE],
      },
      baseAddress: account.publicKey,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot add oracle to asset that can approve in addition to reject', async (t) => {
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

  // Oracle with `CheckResult.CAN_APPROVE`
  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_APPROVE, CheckResult.CAN_REJECT],
      },
      baseAddress: account.publicKey,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot update oracle to approve', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  const result = updatePlugin(umi, {
    asset: asset.publicKey,

    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: account.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_APPROVE],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot update oracle to approve in addition to reject', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  const result = updatePlugin(umi, {
    asset: asset.publicKey,

    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: account.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_APPROVE, CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot create asset with oracle that can listen', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  // Oracle with `CheckResult.CAN_LISTEN`
  const result = createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_LISTEN],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });
});

test('it cannot add oracle to asset that can listen', async (t) => {
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

  // Oracle with `CheckResult.CAN_LISTEN`
  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_LISTEN],
      },
      baseAddress: account.publicKey,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot update oracle to listen', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });

  const result = updatePlugin(umi, {
    asset: asset.publicKey,

    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: account.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_LISTEN],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'OracleCanRejectOnly' });

  // TODO: Add external plugin validation.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
  });
});

test('it cannot use fixed address oracle to deny transfer if not registered for lifecycle event', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const owner = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it can use fixed address oracle to deny create', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Rejected,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const assetSigner = generateSigner(umi);
  const result = create(umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await create(umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner.publicKey,
    owner: umi.identity.publicKey,
  });
});

test('it can use fixed address oracle to deny burn', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const result = burn(umi, {
    asset,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await burn(umi, {
    asset,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('it can use preconfigured program pda oracle to deny update', async (t) => {
  const umi = await createUmi();

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      update: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredProgram',
    },
  };

  const asset = await createAsset(umi, {
    plugins: [oraclePlugin],
  });

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {});

  // Need to close program account from previous test runs on same amman/validator session.
  try {
    await close(umi, {
      account,
      signer: umi.identity,
      payer: umi.identity,
    }).sendAndConfirm(umi);
  } catch (error) {
    if (error.name === 'ProgramErrorNotRecognizedError') {
      // Do nothing.
    } else {
      throw error;
    }
  }

  // write to the PDA which corresponds to the asset
  await preconfiguredProgramPdaInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await preconfiguredProgramPdaSet(umi, {
    account,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use preconfigured collection pda oracle to deny update', async (t) => {
  const umi = await createUmi();

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      update: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredCollection',
    },
  };

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      plugins: [oraclePlugin],
    },
    {}
  );

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    collection: collection.publicKey,
  });

  // write to the PDA which corresponds to the asset
  await preconfiguredAssetPdaInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      additionalPubkey: collection.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    collection,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await preconfiguredAssetPdaSet(umi, {
    account,
    signer: umi.identity,
    args: {
      additionalPubkey: collection.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    collection,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use preconfigured owner pda oracle to deny burn', async (t) => {
  const umi = await createUmi();
  const owner = await generateSignerWithSol(umi);
  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      burn: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredOwner',
    },
  };

  const asset = await createAsset(umi, {
    owner,
    plugins: [oraclePlugin],
  });

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    owner: owner.publicKey,
  });

  // write to the PDA which corresponds to the asset
  await preconfiguredOwnerPdaInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      additionalPubkey: owner.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  const result = burn(umi, {
    asset,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner,
  });

  await preconfiguredOwnerPdaSet(umi, {
    account,
    signer: umi.identity,
    args: {
      additionalPubkey: owner.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await burn(umi, {
    asset,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('it can use preconfigured recipient pda oracle to deny transfer', async (t) => {
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      transfer: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredRecipient',
    },
  };

  const asset = await createAsset(umi, {
    plugins: [oraclePlugin],
  });

  // Find the oracle PDA for the new owner.
  const account = findOracleAccount(umi, oraclePlugin, {
    recipient: newOwner.publicKey,
  });

  // write to the PDA which corresponds to the new owner.
  await preconfiguredRecipientPdaInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      additionalPubkey: newOwner.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await preconfiguredRecipientPdaSet(umi, {
    account,
    signer: umi.identity,
    args: {
      additionalPubkey: newOwner.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it can use preconfigured asset pda oracle to deny update', async (t) => {
  const umi = await createUmi();

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      update: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredAsset',
    },
  };

  const asset = await createAsset(umi, {
    plugins: [oraclePlugin],
  });

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    asset: asset.publicKey,
  });

  // write to the PDA which corresponds to the asset
  await preconfiguredAssetPdaInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      additionalPubkey: asset.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await preconfiguredAssetPdaSet(umi, {
    account,
    signer: umi.identity,
    args: {
      additionalPubkey: asset.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use custom pda (all seeds) oracle to deny transfer', async (t) => {
  const umi = await createUmi();
  const seedPubkey = generateSigner(umi).publicKey;
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      transfer: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'CustomPda',
      seeds: [
        { type: 'Collection' },
        { type: 'Owner' },
        { type: 'Recipient' },
        { type: 'Asset' },
        { type: 'Address', pubkey: seedPubkey },
        {
          type: 'Bytes',
          bytes: Buffer.from('example-seed-bytes', 'utf8'),
        },
      ],
    },
  };

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [oraclePlugin],
    },
    {}
  );

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    collection: collection.publicKey,
    owner: owner.publicKey,
    recipient: newOwner.publicKey,
    asset: asset.publicKey,
  });

  // write to the PDA which corresponds to the asset
  await customPdaAllSeedsInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      collection: collection.publicKey,
      owner: owner.publicKey,
      recipient: newOwner.publicKey,
      asset: asset.publicKey,
      address: seedPubkey,
      bytes: Buffer.from('example-seed-bytes', 'utf8'),
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset,
    collection,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await customPdaAllSeedsSet(umi, {
    account,
    signer: umi.identity,
    args: {
      collection: collection.publicKey,
      owner: owner.publicKey,
      recipient: newOwner.publicKey,
      asset: asset.publicKey,
      address: seedPubkey,
      bytes: Buffer.from('example-seed-bytes', 'utf8'),
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset,
    collection,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it can use custom pda (typical) oracle to deny transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      transfer: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'CustomPda',
      seeds: [
        {
          type: 'Bytes',
          bytes: Buffer.from('prefix-seed-bytes', 'utf8'),
        },
        { type: 'Collection' },
        {
          type: 'Bytes',
          bytes: Buffer.from('additional-bytes-seed-bytes', 'utf8'),
        },
      ],
    },
  };

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [oraclePlugin],
    },
    {}
  );

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    collection: collection.publicKey,
  });

  // write to the PDA
  await customPdaTypicalInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      prefixBytes: Buffer.from('prefix-seed-bytes', 'utf8'),
      collection: collection.publicKey,
      additionalBytes: Buffer.from('additional-bytes-seed-bytes', 'utf8'),
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = transfer(umi, {
    asset,
    collection,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await customPdaTypicalSet(umi, {
    account,
    signer: umi.identity,
    args: {
      prefixBytes: Buffer.from('prefix-seed-bytes', 'utf8'),
      collection: collection.publicKey,
      additionalBytes: Buffer.from('additional-bytes-seed-bytes', 'utf8'),
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset,
    collection,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it can use preconfigured asset pda custom offset oracle to deny update', async (t) => {
  const umi = await createUmi();

  // This test uses an oracle with the data struct:
  // pub struct CustomDataValidation {
  //     pub authority: Pubkey,
  //     pub sequence_num: u64,
  //     pub validation: OracleValidation,
  // }
  //
  // Thus the `resultsOffset` below is set to 48.  This is because the anchor discriminator, the
  // `authority` `Pubkey`, and the `sequence_num` all precede the `OracleValidation` struct
  // within the account:
  //
  // 8 (anchor discriminator) + 32 (authority) + 8 (sequence number) = 48.
  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Custom',
      offset: 48n,
    },
    lifecycleChecks: {
      update: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredAsset',
    },
  };

  const asset = await createAsset(umi, {
    plugins: [oraclePlugin],
  });

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    asset: asset.publicKey,
  });

  const dataAuthority = generateSigner(umi);
  // write to the PDA which corresponds to the asset
  await preconfiguredAssetPdaCustomOffsetInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    authority: dataAuthority.publicKey,
    asset: asset.publicKey,
    oracleData: {
      __kind: 'V1',
      create: ExternalValidationResult.Pass,
      update: ExternalValidationResult.Rejected,
      transfer: ExternalValidationResult.Pass,
      burn: ExternalValidationResult.Pass,
    },
  }).sendAndConfirm(umi);

  const updateResult = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(updateResult, { name: 'InvalidAuthority' });

  // Making sure the incorrect authority cannot update the oracle.  This is more just a test of the
  // example program functionality.
  const setResult = preconfiguredAssetPdaCustomOffsetSet(umi, {
    account,
    authority: umi.identity,
    sequenceNum: 2,
    asset: asset.publicKey,
    oracleData: {
      __kind: 'V1',
      create: ExternalValidationResult.Pass,
      update: ExternalValidationResult.Pass,
      transfer: ExternalValidationResult.Pass,
      burn: ExternalValidationResult.Pass,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(setResult, { name: 'ProgramErrorNotRecognizedError' });

  // Making sure a lower sequence number passes but does not update the oracle.  This is also just
  // a test of the example program functionality.
  await preconfiguredAssetPdaCustomOffsetSet(umi, {
    account,
    authority: dataAuthority,
    sequenceNum: 0,
    asset: asset.publicKey,
    oracleData: {
      __kind: 'V1',
      create: ExternalValidationResult.Pass,
      update: ExternalValidationResult.Pass,
      transfer: ExternalValidationResult.Pass,
      burn: ExternalValidationResult.Pass,
    },
  }).sendAndConfirm(umi);

  // Validate still cannot update the mpl-core asset because the oracle did not change.
  const updateResult2 = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(updateResult2, { name: 'InvalidAuthority' });

  // Oracle update that works.
  await preconfiguredAssetPdaCustomOffsetSet(umi, {
    account,
    authority: dataAuthority,
    sequenceNum: 2,
    asset: asset.publicKey,
    oracleData: {
      __kind: 'V1',
      create: ExternalValidationResult.Pass,
      update: ExternalValidationResult.Pass,
      transfer: ExternalValidationResult.Pass,
      burn: ExternalValidationResult.Pass,
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use one fixed address oracle to deny transfer when a second oracle allows it', async (t) => {
  const umi = await createUmi();
  const account1 = generateSigner(umi);
  const account2 = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: account1,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: account2,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing both oracle accounts
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account1.publicKey,
      },
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account2.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account1.publicKey,
    signer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});
