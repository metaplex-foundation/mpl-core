import test from 'ava';
import { address } from '@solana/addresses';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddPluginV1Instruction,
  getTransferV1Instruction,
  getUpdatePluginV1Instruction,
  MPL_CORE_PROGRAM_ADDRESS,
} from '../../../src';
import { createPlugin, pluginAuthorityPair } from '../../../src/plugins';
import {
  createAsset,
  createAssetWithCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  assertCollection,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../../_setup';

const SPL_SYSTEM_PROGRAM_ADDRESS = address('11111111111111111111111111111111');
const SPL_TOKEN_PROGRAM_ADDRESS = address('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

test('it can transfer an asset with royalties', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },

    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: { __kind: 'None' },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: programOwned.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwned.address,
  });
});

test('it can transfer an asset with collection royalties', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: payer.address, percentage: 100 }],
            ruleSet: { __kind: 'None' },
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: { __kind: 'None' },
    },
  });

  const instruction = getTransferV1Instruction({
    collection: collection.address,
    asset: asset.address,
    payer,
    newOwner: programOwned.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwned.address,
  });
});

test('it can transfer an asset with royalties to an allowlisted program address', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: {
            __kind: 'ProgramAllowList',
            fields: [[SPL_SYSTEM_PROGRAM_ADDRESS, MPL_CORE_PROGRAM_ADDRESS]],
          },
        },
      }),
    ],
  });

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramAllowList',
        fields: [[SPL_SYSTEM_PROGRAM_ADDRESS, MPL_CORE_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: programOwned.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwned.address,
  });
});

test('it can transfer an asset with collection royalties to an allowlisted program address', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: payer.address, percentage: 100 }],
            ruleSet: {
              __kind: 'ProgramAllowList',
              fields: [[SPL_SYSTEM_PROGRAM_ADDRESS, MPL_CORE_PROGRAM_ADDRESS]],
            },
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramAllowList',
        fields: [[SPL_SYSTEM_PROGRAM_ADDRESS, MPL_CORE_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newOwner: programOwned.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwned.address,
  });
});

test('it cannot transfer an asset with royalties to a program address not on the allowlist', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const programOwner = await generateKeyPairSigner();

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer, {
    owner: programOwner.address,
  });

  // Creating a new asset to transfer.
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: {
            __kind: 'ProgramAllowList',
            fields: [[SPL_SYSTEM_PROGRAM_ADDRESS]],
          },
        },
      }),
    ],
  });

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramAllowList',
        fields: [[SPL_SYSTEM_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: programOwned.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot transfer an asset with collection royalties to a program address not on allowlist', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const programOwner = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      owner: programOwner.address,
    },
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: payer.address, percentage: 100 }],
            ruleSet: {
              __kind: 'ProgramAllowList',
              fields: [[SPL_SYSTEM_PROGRAM_ADDRESS]],
            },
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: programOwner.address,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramAllowList',
        fields: [[SPL_SYSTEM_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newOwner: programOwned.address,
    authority: programOwner,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer, programOwner]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwner.address,
  });
});

test('it can transfer an asset with royalties to a program address not on the denylist', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Creating a new asset to transfer.
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: {
            __kind: 'ProgramDenyList',
            fields: [[SPL_TOKEN_PROGRAM_ADDRESS]],
          },
        },
      }),
    ],
  });

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramDenyList',
        fields: [[SPL_TOKEN_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: programOwned.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwned.address,
  });
});

test('it can transfer an asset with collection royalties to a program address not on the denylist', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: payer.address, percentage: 100 }],
            ruleSet: {
              __kind: 'ProgramDenyList',
              fields: [[SPL_TOKEN_PROGRAM_ADDRESS]],
            },
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramDenyList',
        fields: [[SPL_TOKEN_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newOwner: programOwned.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: programOwned.address,
  });
});

test('it cannot transfer an asset with royalties to a denylisted program', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Creating a new asset to transfer.
  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: {
            __kind: 'ProgramDenyList',
            fields: [[MPL_CORE_PROGRAM_ADDRESS]],
          },
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: programOwned.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot transfer an asset with collection royalties to a program address on the denylist', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        pluginAuthorityPair({
          type: 'Royalties',
          data: {
            basisPoints: 5,
            creators: [{ address: payer.address, percentage: 100 }],
            ruleSet: {
              __kind: 'ProgramDenyList',
              fields: [[MPL_CORE_PROGRAM_ADDRESS]],
            },
          },
        }),
      ],
    }
  );

  // Here we're creating a new owner that's program owned, so we're just going to use another asset.
  const programOwned = await createAsset(rpc, payer);

  // Then an account was created with the correct data.
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    royalties: {
      authority: {
        type: 'UpdateAuthority',
      },
      basisPoints: 5,
      creators: [{ address: payer.address, percentage: 100 }],
      ruleSet: {
        __kind: 'ProgramDenyList',
        fields: [[MPL_CORE_PROGRAM_ADDRESS]],
      },
    },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    newOwner: programOwned.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
  });
});

test('it cannot create royalty percentages that dont add up to 100', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const creator1 = await generateKeyPairSigner();
  const creator2 = await generateKeyPairSigner();

  const result = createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [
            { address: creator1.address, percentage: 20 },
            { address: creator2.address, percentage: 20 },
          ],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot create royalty basis points greater than 10000', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 10001,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot create royalty with duplicate creators', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const result = createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 10001,
          creators: [
            {
              address: payer.address,
              percentage: 10,
            },
            {
              address: payer.address,
              percentage: 90,
            },
          ],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  await t.throwsAsync(result);
});

test('it cannot add royalty percentages that dont add up to 100', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {});
  const creator1 = await generateKeyPairSigner();
  const creator2 = await generateKeyPairSigner();

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          { address: creator1.address, percentage: 20 },
          { address: creator2.address, percentage: 20 },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add royalty percentages that has duplicate creators', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {});
  const creator1 = await generateKeyPairSigner();

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          { address: creator1.address, percentage: 20 },
          { address: creator1.address, percentage: 80 },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add royalty basis points greater than 10000', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const asset = await createAsset(rpc, payer, {});

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 10001,
        creators: [{ address: payer.address, percentage: 100 }],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot update royalty percentages that do not add up to 100', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const creator1 = await generateKeyPairSigner();
  const creator2 = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 5,
          creators: [
            { address: creator1.address, percentage: 20 },
            { address: creator2.address, percentage: 80 },
          ],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          { address: creator1.address, percentage: 20 },
          { address: creator2.address, percentage: 20 },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot update royalty basis points greater than 10000', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 100,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 10001,
        creators: [{ address: payer.address, percentage: 100 }],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot update royalty with duplicate creators', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Royalties',
        data: {
          basisPoints: 100,
          creators: [{ address: payer.address, percentage: 100 }],
          ruleSet: { __kind: 'None' },
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: createPlugin({
      type: 'Royalties',
      data: {
        basisPoints: 10001,
        creators: [
          {
            address: payer.address,
            percentage: 10,
          },
          {
            address: payer.address,
            percentage: 90,
          },
        ],
        ruleSet: { __kind: 'None' },
      },
    }),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});
