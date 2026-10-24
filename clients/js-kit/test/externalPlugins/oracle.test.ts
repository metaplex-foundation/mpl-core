/**
 * Oracle External Plugin Tests
 *
 * NOTE: Most of these tests require the @metaplex-foundation/mpl-core-oracle-example
 * package to be ported to web3.js 2.0 before they can be fully functional.
 *
 * The oracle example program provides test oracle accounts that can validate
 * lifecycle events (create, update, transfer, burn) on assets and collections.
 *
 * Tests that can work without the oracle example program are included,
 * but most tests are commented out pending the oracle example port.
 */

import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import {
  getCreateV1Instruction,
  getUpdateV1Instruction,
  getTransferV1Instruction,
  getBurnV1Instruction,
  getAddExternalPluginAdapterV1Instruction,
  getUpdateExternalPluginAdapterV1Instruction,
  fetchAssetV1,
  DataState,
  Key,
  CheckResult,
  ExternalValidationResult,
} from '../../src';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';
import {
  externalPluginAdapterInitInfoArgs,
  externalPluginAdapterUpdateInfoArgs,
} from '../../src/plugins';

// TODO: Once @metaplex-foundation/mpl-core-oracle-example is ported to web3.js 2.0,
// uncomment these imports and update them to the new package structure:
//
// import {
//   mplCoreOracleExample,
//   fixedAccountInit,
//   fixedAccountSet,
//   preconfiguredAssetPdaInit,
//   MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
//   preconfiguredAssetPdaSet,
//   preconfiguredProgramPdaInit,
//   preconfiguredProgramPdaSet,
//   preconfiguredOwnerPdaInit,
//   preconfiguredOwnerPdaSet,
//   preconfiguredRecipientPdaInit,
//   preconfiguredRecipientPdaSet,
//   customPdaAllSeedsInit,
//   customPdaAllSeedsSet,
//   customPdaTypicalInit,
//   customPdaTypicalSet,
//   preconfiguredAssetPdaCustomOffsetInit,
//   preconfiguredAssetPdaCustomOffsetSet,
//   close,
// } from '@metaplex-foundation/mpl-core-oracle-example';

/**
 * Tests below are COMMENTED OUT because they require the oracle example program.
 * Uncomment these tests once the oracle example package is ported.
 */

/*
test('it can add oracle to asset for multiple lifecycle events', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const account = await generateKeyPairSigner();

  // write to example program oracle account
  // TODO: Convert fixedAccountInit to web3.js 2.0
  await fixedAccountInit(rpc, {
    account,
    signer: payer,
    payer: payer,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  });

  // create asset referencing the oracle account
  const asset = await createAsset(rpc, payer, {
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.address,
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    oracles: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        authority: {
          type: 'UpdateAuthority',
        },
        baseAddress: account.address,
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
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const account = await generateKeyPairSigner();
  const account2 = await generateKeyPairSigner();
  const delegateAddress = await generateKeyPairSigner();

  // write to example program oracle account
  await fixedAccountInit(rpc, {
    account,
    signer: payer,
    payer: payer,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  });

  // write to example program oracle account
  await fixedAccountInit(rpc, {
    account: account2,
    signer: payer,
    payer: payer,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  });

  // create asset referencing the oracle account
  const asset = await createAsset(rpc, payer, {
    plugins: [
      {
        type: 'TransferDelegate',
      },
    ],
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT],
          transfer: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.address,
      }),
    ],
  });

  const addRoyaltiesInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'Royalties',
      fields: [{
        basisPoints: 5,
        creators: [{ address: payer.address, percentage: 100 }],
        ruleSet: { __kind: 'None' },
      }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addRoyaltiesInstruction],
    [payer]
  );

  const addOracleInstruction = getAddExternalPluginAdapterV1Instruction({
    asset: asset.address,
    payer,
    initInfo: {
      __kind: 'Oracle',
      fields: [{
        baseAddress: account2.address,
        baseAddressConfig: null,
        resultsOffset: {
          __kind: 'Anchor',
        },
        lifecycleChecks: {
          create: [],
          update: [CheckResult.CAN_REJECT],
          transfer: [],
          burn: [CheckResult.CAN_REJECT],
        },
        initPluginAuthority: null,
      }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addOracleInstruction],
    [payer]
  );

  const addFreezeDelegateInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'FreezeDelegate',
      fields: [{
        frozen: false,
      }],
    },
    initInfo: {
      __kind: 'Address',
      fields: [delegateAddress.address],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addFreezeDelegateInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
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
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: { type: 'None' },
    },
    freezeDelegate: {
      authority: {
        type: 'Address',
        address: delegateAddress.address,
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
        baseAddress: account.address,
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
        baseAddress: account2.address,
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddressConfig: undefined,
      },
    ],
  });
});
*/

/**
 * Tests that can work without oracle example program
 */

test('add oracle to asset with no offset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const account = await generateKeyPairSigner();

  // Create asset with oracle plugin (using NoOffset type)
  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'NoOffset',
        },
        lifecycleChecks: {
          burn: [CheckResult.CAN_REJECT],
        },
        baseAddress: account.address,
      }),
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  const asset = await fetchAssetV1(rpc, assetAddress.address);

  t.is(asset.data.key, Key.AssetV1);
  t.is(asset.data.owner, payer.address);
  t.is(asset.data.name, 'Test Asset');
  t.is(asset.data.uri, 'https://example.com/asset');
  // TODO: Add assertions for oracle plugin once external plugin adapter parsing is implemented
});

test('it cannot create asset with oracle that has no lifecycle checks', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const account = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          // Empty lifecycle checks should fail
        },
        baseAddress: account.address,
      }),
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create asset with oracle that can approve', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const account = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_APPROVE],
        },
        baseAddress: account.address,
      }),
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create asset with oracle that can approve in addition to reject', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const account = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_REJECT, CheckResult.CAN_APPROVE],
        },
        baseAddress: account.address,
      }),
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create asset with oracle that can listen', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const account = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_LISTEN],
        },
        baseAddress: account.address,
      }),
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it create fails but does not panic when oracle account does not exist', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const oracleSigner = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Asset',
    uri: 'https://example.com/asset',
    externalPluginAdapters: [
      externalPluginAdapterInitInfoArgs({
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
        baseAddress: oracleSigner.address,
      }),
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

/*
 * The following tests require the oracle example program and are commented out.
 * Uncomment these tests once @metaplex-foundation/mpl-core-oracle-example
 * is ported to web3.js 2.0.
 *
 * Tests include:
 * - it can use fixed address oracle to deny update
 * - it can use fixed address oracle to deny update via collection
 * - it can use fixed address oracle to deny transfer
 * - it can add oracle that can reject to asset
 * - it cannot add oracle with no lifecycle checks to asset
 * - it cannot update oracle to have no lifecycle checks
 * - it cannot add oracle to asset that can approve
 * - it cannot add oracle to asset that can approve in addition to reject
 * - it cannot update oracle to approve
 * - it cannot update oracle to approve in addition to reject
 * - it cannot add oracle to asset that can listen
 * - it cannot update oracle to listen
 * - it cannot use fixed address oracle to deny transfer if not registered for lifecycle event
 * - it can use fixed address oracle to deny create
 * - it can use fixed address oracle to deny burn
 * - it can use preconfigured program pda oracle to deny update
 * - it can use preconfigured collection pda oracle to deny update
 * - it can use preconfigured owner pda oracle to deny burn
 * - it can use preconfigured recipient pda oracle to deny transfer
 * - it can use preconfigured asset pda oracle to deny update
 * - it can use custom pda (all seeds) oracle to deny transfer
 * - it can use custom pda (typical) oracle to deny transfer
 * - it can use custom pda (with custom program ID) oracle to deny transfer
 * - it can use preconfigured asset pda custom offset oracle to deny update
 * - it can use one fixed address oracle to deny transfer when a second oracle allows it
 * - it can update asset to different size name with oracle
 * - it can update oracle to smaller registry record
 * - it can update oracle to larger registry record
 * - it transfer fails but does not panic when oracle account does not exist
 * - it transfer fails but does not panic when oracle account is too small
 * - it empty account does not default to valid oracle
 * - it can update oracle with external plugin authority different than asset update authority
 * - it cannot update oracle using update authority when different from external plugin authority
 * - it can update oracle on collection with external plugin authority different than asset update authority
 * - it cannot update oracle on collection using update authority when different from external plugin authority
 * - it can create an oracle on a collection with create set to reject
 * - it can use fixed address oracle on a collection to deny create
 */
