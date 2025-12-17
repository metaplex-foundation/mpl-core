import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import type { Address } from '@solana/addresses';
import type { Rpc } from '@solana/rpc';
import type { SolanaRpcApi } from '@solana/rpc';
import {
  getCollectInstruction,
  getAddPluginV1Instruction,
  getRemovePluginV1Instruction,
  getBurnV1Instruction,
  PluginType,
  ExternalPluginAdapterSchema,
  ExternalValidationResult,
  CheckResult,
} from '../src';
import { pluginAuthorityPair } from '../src/plugins';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  airdrop,
  sendAndConfirmInstructions,
} from './_setup';
import { fixedAccountInit } from '@metaplex-foundation/mpl-core-oracle-example';

const recipient1 = address('8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v');
const recipient2 = address('MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt');

test.before(async () => {
  const rpc = createRpc();
  await airdrop(rpc, recipient1, 100_000_000n);
  await airdrop(rpc, recipient2, 100_000_000n);
});

const hasCollectAmount = async (rpc: Rpc<SolanaRpcApi>, address_: Address) => {
  const account = await rpc.getAccountInfo(address_, { encoding: 'base64' }).send();
  if (account.value) {
    const accountSize = account.value.data[0] ? Buffer.from(account.value.data[0], 'base64').length : 0;
    const rentResponse = await rpc.getMinimumBalanceForRentExemption(BigInt(accountSize)).send();
    const rent = rentResponse;
    const diff = account.value.lamports - rent;
    return diff === 1_500_000n;
  }
  return false;
};

const assertNoExcessRent = async (rpc: Rpc<SolanaRpcApi>, address_: Address) => {
  const account = await rpc.getAccountInfo(address_, { encoding: 'base64' }).send();
  if (account.value) {
    const accountSize = account.value.data[0] ? Buffer.from(account.value.data[0], 'base64').length : 0;
    const rentResponse = await rpc.getMinimumBalanceForRentExemption(BigInt(accountSize)).send();
    const rent = rentResponse;
    return account.value.lamports === rent;
  }
  return false;
};

test('it can create a new asset with collect amount', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );
});

test('it can add asset plugin with collect amount', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: pluginAuthorityPair({
      type: 'FreezeDelegate',
      data: { frozen: true },
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );
});

test('it can add asset external plugin with collect amount', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: pluginAuthorityPair({
      type: 'AppData',
      data: {
        dataAuthority: { __kind: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Json,
      },
    }),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );
});

test('it can remove asset plugin with collect amount', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
    ],
  });

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.FreezeDelegate,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );
});

test('it can remove asset external plugin with collect amount', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'UpdateAuthority' },
          schema: ExternalPluginAdapterSchema.Json,
        },
      }),
    ],
  });

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );

  const instruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    pluginType: PluginType.AppData,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  t.assert(
    await hasCollectAmount(rpc, asset.address),
    'Collect amount not found'
  );
});

test.serial('it can collect', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const balStart1 = await rpc.getBalance(recipient1).send();
  const balStart2 = await rpc.getBalance(recipient2).send();

  const collectInstruction = getCollectInstruction({
    recipient1,
    recipient2,
  });

  // Add the asset as a remaining account
  const instructionWithRemainingAccounts = {
    ...collectInstruction,
    accounts: [
      ...collectInstruction.accounts,
      { address: asset.address, role: 3 }, // 3 = writable
    ],
  };

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instructionWithRemainingAccounts],
    [payer]
  );

  const balEnd1 = await rpc.getBalance(recipient1).send();
  const balEnd2 = await rpc.getBalance(recipient2).send();

  t.is(await hasCollectAmount(rpc, asset.address), false);
  t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
  t.deepEqual(balEnd2.value - balStart2.value, 750_000n);
});

test.serial('it can collect from an asset with plugins', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            {
              key: 'Test',
              value: 'Test',
            },
          ],
        },
      }),
    ],
  });

  const balStart1 = await rpc.getBalance(recipient1).send();
  const balStart2 = await rpc.getBalance(recipient2).send();

  const collectInstruction = getCollectInstruction({
    recipient1,
    recipient2,
  });

  const instructionWithRemainingAccounts = {
    ...collectInstruction,
    accounts: [
      ...collectInstruction.accounts,
      { address: asset.address, role: 3 },
    ],
  };

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instructionWithRemainingAccounts],
    [payer]
  );

  const balEnd1 = await rpc.getBalance(recipient1).send();
  const balEnd2 = await rpc.getBalance(recipient2).send();

  t.is(await hasCollectAmount(rpc, asset.address), false);
  t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
  t.deepEqual(balEnd2.value - balStart2.value, 750_000n);

  t.assert(await assertNoExcessRent(rpc, asset.address), 'Excess rent found');
});

test.serial('it can collect from an asset with external plugins', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const account = await generateKeyPairSigner();

  // write to example program oracle account
  await fixedAccountInit(
    {
      rpc,
      rpcSubscriptions,
      createKeyPairSigner: generateKeyPairSigner,
    },
    {
      account,
      signer: payer,
      payer,
      args: {
        oracleData: {
          __kind: 'V1',
          create: ExternalValidationResult.Pass,
          update: ExternalValidationResult.Rejected,
          transfer: ExternalValidationResult.Pass,
          burn: ExternalValidationResult.Pass,
        },
      },
    }
  );

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: false },
      }),
      pluginAuthorityPair({
        type: 'Attributes',
        data: {
          attributeList: [
            {
              key: 'Test',
              value: 'Test',
            },
          ],
        },
      }),
      pluginAuthorityPair({
        type: 'Oracle',
        data: {
          resultsOffset: {
            __kind: 'Anchor',
          },
          lifecycleChecks: {
            create: [CheckResult.CAN_REJECT],
            transfer: [CheckResult.CAN_REJECT],
          },
          baseAddress: account.address,
        },
      }),
    ],
  });

  const balStart1 = await rpc.getBalance(recipient1).send();
  const balStart2 = await rpc.getBalance(recipient2).send();

  const collectInstruction = getCollectInstruction({
    recipient1,
    recipient2,
  });

  const instructionWithRemainingAccounts = {
    ...collectInstruction,
    accounts: [
      ...collectInstruction.accounts,
      { address: asset.address, role: 3 },
    ],
  };

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instructionWithRemainingAccounts],
    [payer]
  );

  const balEnd1 = await rpc.getBalance(recipient1).send();
  const balEnd2 = await rpc.getBalance(recipient2).send();

  t.is(await hasCollectAmount(rpc, asset.address), false);
  t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
  t.deepEqual(balEnd2.value - balStart2.value, 750_000n);

  t.assert(await assertNoExcessRent(rpc, asset.address), 'Excess rent found');
});

test.serial(
  'it can collect from an asset with plugins that was burned',
  async (t) => {
    const rpc = createRpc();
    const rpcSubscriptions = createRpcSubscriptions();
    const payer = await generateSignerWithSol(rpc);

    const asset = await createAsset(rpc, payer, {
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: false },
        }),
        pluginAuthorityPair({
          type: 'Attributes',
          data: {
            attributeList: [
              {
                key: 'Test',
                value: 'Test',
              },
            ],
          },
        }),
      ],
    });

    const balStart1 = await rpc.getBalance(recipient1).send();
    const balStart2 = await rpc.getBalance(recipient2).send();

    const burnInstruction = getBurnV1Instruction({
      asset: asset.address,
      payer,
    });

    await sendAndConfirmInstructions(
      rpc,
      rpcSubscriptions,
      [burnInstruction],
      [payer]
    );

    const collectInstruction = getCollectInstruction({
      recipient1,
      recipient2,
    });

    const instructionWithRemainingAccounts = {
      ...collectInstruction,
      accounts: [
        ...collectInstruction.accounts,
        { address: asset.address, role: 3 },
      ],
    };

    await sendAndConfirmInstructions(
      rpc,
      rpcSubscriptions,
      [instructionWithRemainingAccounts],
      [payer]
    );

    const balEnd1 = await rpc.getBalance(recipient1).send();
    const balEnd2 = await rpc.getBalance(recipient2).send();

    t.is(await hasCollectAmount(rpc, asset.address), false);
    t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
    t.deepEqual(balEnd2.value - balStart2.value, 750_000n);

    t.assert(
      await assertNoExcessRent(rpc, asset.address),
      'Excess rent found'
    );
  }
);

test.serial(
  'it can collect from an asset with external plugins that was burned',
  async (t) => {
    const rpc = createRpc();
    const rpcSubscriptions = createRpcSubscriptions();
    const payer = await generateSignerWithSol(rpc);
    const account = await generateKeyPairSigner();

    // write to example program oracle account
    await fixedAccountInit(
      {
        rpc,
        rpcSubscriptions,
        createKeyPairSigner: generateKeyPairSigner,
      },
      {
        account,
        signer: payer,
        payer,
        args: {
          oracleData: {
            __kind: 'V1',
            create: ExternalValidationResult.Pass,
            update: ExternalValidationResult.Rejected,
            transfer: ExternalValidationResult.Pass,
            burn: ExternalValidationResult.Pass,
          },
        },
      }
    );

    const asset = await createAsset(rpc, payer, {
      plugins: [
        pluginAuthorityPair({
          type: 'FreezeDelegate',
          data: { frozen: false },
        }),
        pluginAuthorityPair({
          type: 'Attributes',
          data: {
            attributeList: [
              {
                key: 'Test',
                value: 'Test',
              },
            ],
          },
        }),
        pluginAuthorityPair({
          type: 'Oracle',
          data: {
            resultsOffset: {
              __kind: 'Anchor',
            },
            lifecycleChecks: {
              create: [CheckResult.CAN_REJECT],
              transfer: [CheckResult.CAN_REJECT],
            },
            baseAddress: account.address,
          },
        }),
      ],
    });

    const balStart1 = await rpc.getBalance(recipient1).send();
    const balStart2 = await rpc.getBalance(recipient2).send();

    const burnInstruction = getBurnV1Instruction({
      asset: asset.address,
      payer,
    });

    await sendAndConfirmInstructions(
      rpc,
      rpcSubscriptions,
      [burnInstruction],
      [payer]
    );

    const collectInstruction = getCollectInstruction({
      recipient1,
      recipient2,
    });

    const instructionWithRemainingAccounts = {
      ...collectInstruction,
      accounts: [
        ...collectInstruction.accounts,
        { address: asset.address, role: 3 },
      ],
    };

    await sendAndConfirmInstructions(
      rpc,
      rpcSubscriptions,
      [instructionWithRemainingAccounts],
      [payer]
    );

    const balEnd1 = await rpc.getBalance(recipient1).send();
    const balEnd2 = await rpc.getBalance(recipient2).send();

    t.is(await hasCollectAmount(rpc, asset.address), false);
    t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
    t.deepEqual(balEnd2.value - balStart2.value, 750_000n);

    t.assert(
      await assertNoExcessRent(rpc, asset.address),
      'Excess rent found'
    );
  }
);

test.serial('it can collect burned asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const balStart1 = await rpc.getBalance(recipient1).send();
  const balStart2 = await rpc.getBalance(recipient2).send();

  const burnInstruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [burnInstruction],
    [payer]
  );

  const collectInstruction = getCollectInstruction({
    recipient1,
    recipient2,
  });

  const instructionWithRemainingAccounts = {
    ...collectInstruction,
    accounts: [
      ...collectInstruction.accounts,
      { address: asset.address, role: 3 },
    ],
  };

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instructionWithRemainingAccounts],
    [payer]
  );

  const balEnd1 = await rpc.getBalance(recipient1).send();
  const balEnd2 = await rpc.getBalance(recipient2).send();

  t.is(await hasCollectAmount(rpc, asset.address), false);
  t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
  t.deepEqual(balEnd2.value - balStart2.value, 750_000n);

  t.assert(await assertNoExcessRent(rpc, asset.address), 'Excess rent found');
});

test.serial(
  'it can collect multiple times on same asset idempotently',
  async (t) => {
    const rpc = createRpc();
    const rpcSubscriptions = createRpcSubscriptions();
    const payer = await generateSignerWithSol(rpc);

    const asset = await createAsset(rpc, payer);

    const balStart1 = await rpc.getBalance(recipient1).send();
    const balStart2 = await rpc.getBalance(recipient2).send();

    const collectInstruction = getCollectInstruction({
      recipient1,
      recipient2,
    });

    const instructionWithRemainingAccounts = {
      ...collectInstruction,
      accounts: [
        ...collectInstruction.accounts,
        { address: asset.address, role: 3 },
      ],
    };

    await sendAndConfirmInstructions(
      rpc,
      rpcSubscriptions,
      [instructionWithRemainingAccounts],
      [payer]
    );

    const balMid1 = await rpc.getBalance(recipient1).send();
    const balMid2 = await rpc.getBalance(recipient2).send();

    t.is(await hasCollectAmount(rpc, asset.address), false);
    t.deepEqual(balMid1.value - balStart1.value, 750_000n);
    t.deepEqual(balMid2.value - balStart2.value, 750_000n);

    const collectInstruction2 = getCollectInstruction({
      recipient1,
      recipient2,
    });

    const instructionWithRemainingAccounts2 = {
      ...collectInstruction2,
      accounts: [
        ...collectInstruction2.accounts,
        { address: asset.address, role: 3 },
      ],
    };

    await sendAndConfirmInstructions(
      rpc,
      rpcSubscriptions,
      [instructionWithRemainingAccounts2],
      [payer]
    );

    const balEnd1 = await rpc.getBalance(recipient1).send();
    const balEnd2 = await rpc.getBalance(recipient2).send();

    t.is(await hasCollectAmount(rpc, asset.address), false);
    t.deepEqual(balEnd1.value - balStart1.value, 750_000n);
    t.deepEqual(balEnd2.value - balStart2.value, 750_000n);

    t.assert(
      await assertNoExcessRent(rpc, asset.address),
      'Excess rent found'
    );
  }
);

test.serial('it can collect multiple assets at once', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const asset2 = await createAsset(rpc, payer);
  const asset3 = await createAsset(rpc, payer);

  const balStart1 = await rpc.getBalance(recipient1).send();
  const balStart2 = await rpc.getBalance(recipient2).send();

  const collectInstruction = getCollectInstruction({
    recipient1,
    recipient2,
  });

  const instructionWithRemainingAccounts = {
    ...collectInstruction,
    accounts: [
      ...collectInstruction.accounts,
      { address: asset.address, role: 3 },
      { address: asset2.address, role: 3 },
      { address: asset3.address, role: 3 },
    ],
  };

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instructionWithRemainingAccounts],
    [payer]
  );

  const balEnd1 = await rpc.getBalance(recipient1).send();
  const balEnd2 = await rpc.getBalance(recipient2).send();

  t.is(await hasCollectAmount(rpc, asset.address), false);
  t.is(await hasCollectAmount(rpc, asset2.address), false);
  t.is(await hasCollectAmount(rpc, asset3.address), false);
  t.deepEqual(balEnd1.value - balStart1.value, 2_250_000n);
  t.deepEqual(balEnd2.value - balStart2.value, 2_250_000n);

  t.assert(await assertNoExcessRent(rpc, asset.address), 'Excess rent found');
});

test('it can transfer after collecting', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);

  const collectInstruction = getCollectInstruction({
    recipient1,
    recipient2,
  });

  const instructionWithRemainingAccounts = {
    ...collectInstruction,
    accounts: [
      ...collectInstruction.accounts,
      { address: asset.address, role: 3 },
    ],
  };

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instructionWithRemainingAccounts],
    [payer]
  );

  t.is(await hasCollectAmount(rpc, asset.address), false);

  const newOwner = await generateKeyPairSigner();

  const { getTransferV1Instruction } = await import('../src');

  const transferInstruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [transferInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  t.assert(await assertNoExcessRent(rpc, asset.address), 'Excess rent found');
});
