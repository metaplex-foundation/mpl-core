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
import {
  generateSigner,
  sol,
  assertAccountExists,
} from '@metaplex-foundation/umi';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { createAccount } from '@metaplex-foundation/mpl-toolbox';
import {
  assertAsset,
  assertBurned,
  assertCollection,
  createUmi as baseCreateUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
} from '../_setupRaw';
import { createAsset, createAssetWithCollection } from '../_setupSdk';
import {
  burn,
  CheckResult,
  create,
  createCollection,
  updateCollectionPlugin,
  findOracleAccount,
  OracleInitInfoArgs,
  transfer,
  update,
  addPlugin,
  updatePlugin,
  fetchAssetV1,
  ExternalValidationResult,
  ruleSet,
  fetchCollection,
} from '../../src';

const createUmi = async () =>
  (await baseCreateUmi()).use(mplCoreOracleExample());

test('it can add oracle to asset for multiple lifecycle events', async (t) => {
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
          create: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it can add multiple oracles and internal plugins to asset', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);
  const account2 = generateSigner(umi);
  const delegateAddress = generateSigner(umi);

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

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: account2,
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
        type: 'TransferDelegate',
      },
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Royalties',
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        update: [CheckResult.CAN_REJECT],
        burn: [CheckResult.CAN_REJECT],
      },
      baseAddress: account2.publicKey,
    },
  }).sendAndConfirm(umi);

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'FreezeDelegate',
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    transferDelegate: {
      authority: {
        type: 'Owner',
      },
    },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: umi.identity.publicKey, percentage: 100 }],
      ruleSet: ruleSet('None'),
    },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.publicKey,
      },
      frozen: false,
    },
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account2.publicKey,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('add oracle to asset with no offset', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        lifecycleChecks: {
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });
});

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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

test('it can add oracle that can reject to asset', async (t) => {
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

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_REJECT],
      },
      baseAddress: account.publicKey,
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });
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
    oracles: undefined,
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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
    oracles: undefined,
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
    oracles: undefined,
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
    oracles: undefined,
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
    oracles: undefined,
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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
    oracles: undefined,
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
    oracles: undefined,
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.publicKey,
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'PreconfiguredProgram',
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'PreconfiguredCollection',
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'PreconfiguredOwner',
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'PreconfiguredRecipient',
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'PreconfiguredAsset',
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'CustomPda',
          seeds: [
            { type: 'Collection' },
            { type: 'Owner' },
            { type: 'Recipient' },
            { type: 'Asset' },
            { type: 'Address', pubkey: seedPubkey },
            {
              type: 'Bytes',
              bytes: new Uint8Array(Buffer.from('example-seed-bytes', 'utf8')),
            },
          ],
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'CustomPda',
          seeds: [
            {
              type: 'Bytes',
              bytes: new Uint8Array(Buffer.from('prefix-seed-bytes', 'utf8')),
            },
            { type: 'Collection' },
            {
              type: 'Bytes',
              bytes: new Uint8Array(
                Buffer.from('additional-bytes-seed-bytes', 'utf8')
              ),
            },
          ],
        },
      },
    ],
  });
});

test('it can use custom pda (with custom program ID) oracle to deny transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  // Configure an Oracle plugin to have a custom program ID.  In order to reuse the oracle
  // example program we will set the base address to a random Pubkey, and set the custom program
  // ID to the oracle example program ID.
  const randomProgramId = generateSigner(umi).publicKey;
  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      transfer: [CheckResult.CAN_REJECT],
    },
    baseAddress: randomProgramId,
    baseAddressConfig: {
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
      customProgramId: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: randomProgramId,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'CustomPda',
          seeds: [
            {
              type: 'Bytes',
              bytes: new Uint8Array(Buffer.from('prefix-seed-bytes', 'utf8')),
            },
            { type: 'Collection' },
            {
              type: 'Bytes',
              bytes: new Uint8Array(
                Buffer.from('additional-bytes-seed-bytes', 'utf8')
              ),
            },
          ],
          customProgramId: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        },
      },
    ],
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
    baseAddressConfig: {
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Custom',
          offset: 48n,
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: {
          type: 'PreconfiguredAsset',
        },
      },
    ],
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
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account1.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
      },
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account2.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });
});

test('it can update asset to different size name with oracle', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  const asset = generateSigner(umi);
  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await fixedAccountSet(umi, {
    signer: umi.identity,
    account: oracleSigner.publicKey,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const a = await fetchAssetV1(umi, asset.publicKey);

  await update(umi, {
    asset: a,
    name: 'name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'name 2',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it can update oracle to smaller registry record', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  const asset = generateSigner(umi);
  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  const accountBefore = await umi.rpc.getAccount(asset.publicKey);
  t.true(accountBefore.exists);
  assertAccountExists(accountBefore);
  const beforeLength = accountBefore.data.length;

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: oracleSigner.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: undefined,
          update: undefined,
          transfer: [CheckResult.CAN_REJECT],
          burn: undefined,
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  // Validate that account got smaller by specific amount.
  const accountAfter = await umi.rpc.getAccount(asset.publicKey);
  t.true(accountAfter.exists);
  assertAccountExists(accountAfter);
  const afterLength = accountAfter.data.length;
  t.is(afterLength - beforeLength, -5);
});

test('it can update oracle to larger registry record', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  const asset = generateSigner(umi);
  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  const accountBefore = await umi.rpc.getAccount(asset.publicKey);
  t.true(accountBefore.exists);
  assertAccountExists(accountBefore);
  const beforeLength = accountBefore.data.length;

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: oracleSigner.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor',
      },
      lifecycleChecks: {
        create: [CheckResult.CAN_REJECT],
        update: [CheckResult.CAN_REJECT],
        transfer: [CheckResult.CAN_REJECT],
        burn: [CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          update: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  // Validate that account got larger by specific amount.
  const accountAfter = await umi.rpc.getAccount(asset.publicKey);
  t.true(accountAfter.exists);
  assertAccountExists(accountAfter);
  const afterLength = accountAfter.data.length;
  t.is(afterLength - beforeLength, 15);
});

test('it create fails but does not panic when oracle account does not exist', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);

  const asset = generateSigner(umi);
  const result = create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          update: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidOracleAccountData' });
});

test('it transfer fails but does not panic when oracle account does not exist', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);

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
        baseAddress: oracleSigner.publicKey,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: oracleSigner.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });

  const newOwner = generateSigner(umi);
  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidOracleAccountData' });
});

test('it transfer fails but does not panic when oracle account is too small', async (t) => {
  const umi = await createUmi();
  const newAccount = generateSigner(umi);

  // Create an invalid oracle account that is an account with 3 bytes.
  await createAccount(umi, {
    newAccount,
    lamports: sol(0.1),
    space: 3,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: newAccount.publicKey,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: newAccount.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });

  const newOwner = generateSigner(umi);
  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidOracleAccountData' });
});

test('it empty account does not default to valid oracle', async (t) => {
  const umi = await createUmi();
  const newAccount = generateSigner(umi);

  // Create an invalid oracle account that is an account with 42 bytes.
  await createAccount(umi, {
    newAccount,
    lamports: sol(0.1),
    space: 42,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: newAccount.publicKey,
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: newAccount.publicKey,
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });

  const newOwner = generateSigner(umi);
  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'UninitializedOracleAccount' });
});

test('it can update oracle with external plugin authority different than asset update authority', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const asset = generateSigner(umi);
  const oracleUpdateAuthority = generateSigner(umi);
  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        initPluginAuthority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    authority: oracleUpdateAuthority,
    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: oracleSigner.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'NoOffset',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_REJECT],
        burn: [CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it cannot update oracle using update authority when different from external plugin authority', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const asset = generateSigner(umi);
  const oracleUpdateAuthority = generateSigner(umi);
  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        initPluginAuthority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  const result = updatePlugin(umi, {
    asset: asset.publicKey,
    authority: umi.identity,
    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: oracleSigner.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'NoOffset',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_REJECT],
        burn: [CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it can update oracle on collection with external plugin authority different than asset update authority', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const collection = generateSigner(umi);
  const oracleUpdateAuthority = generateSigner(umi);
  await createCollection(umi, {
    collection,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        initPluginAuthority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    authority: oracleUpdateAuthority,
    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: oracleSigner.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'NoOffset',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_REJECT],
        burn: [CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it cannot update oracle on collection using update authority when different from external plugin authority', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const collection = generateSigner(umi);
  const oracleUpdateAuthority = generateSigner(umi);
  await createCollection(umi, {
    collection,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        initPluginAuthority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  const result = updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    authority: umi.identity,
    plugin: {
      key: {
        type: 'Oracle',
        baseAddress: oracleSigner.publicKey,
      },
      type: 'Oracle',
      resultsOffset: {
        type: 'NoOffset',
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_REJECT],
        burn: [CheckResult.CAN_REJECT],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    oracles: [
      {
        authority: {
          type: 'Address',
          address: oracleUpdateAuthority.publicKey,
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it can create an oracle on a collection with create set to reject', async (t) => {
  const umi = await createUmi();
  const oracleSigner = generateSigner(umi);
  await fixedAccountInit(umi, {
    signer: umi.identity,
    account: oracleSigner,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const collection = generateSigner(umi);
  await createCollection(umi, {
    collection,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddress: oracleSigner.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });
});

test('it can use fixed address oracle on a collection to deny create', async (t) => {
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

  const collectionSigner = generateSigner(umi);
  await createCollection(umi, {
    collection: collectionSigner,
    name: 'Test name',
    uri: 'https://example.com',
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

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collectionSigner.publicKey,
    updateAuthority: umi.identity.publicKey,
    oracles: [
      {
        authority: {
          type: 'UpdateAuthority',
        },
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.publicKey,
        baseAddressConfig: undefined,
      },
    ],
  });

  const collection = await fetchCollection(umi, collectionSigner.publicKey);

  // create asset referencing the oracle account
  const assetSigner = generateSigner(umi);
  const result = create(umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner,
    collection,
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
    collection,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner.publicKey,
    owner: umi.identity.publicKey,
  });
});
