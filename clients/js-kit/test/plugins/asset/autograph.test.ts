import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import {
  getAddPluginV1Instruction,
  getUpdatePluginV1Instruction,
} from '../../../src';
import { pluginAuthorityPair } from '../../../src/plugins';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../../_setup';

test('it can create asset with autograph plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: { signatures: [] },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [],
    },
  });
});

test('it cannot create asset with autograph plugin and unauthorized signature', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const autograph = await generateKeyPairSigner();

  const res = createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: autograph.address,
              message: 'hi',
            },
          ],
        },
      }),
    ],
  });

  await t.throwsAsync(res);
});

test('it can create asset with autograph plugin with authorized signature', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'hi',
            },
          ],
        },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: payer.address,
          message: 'hi',
        },
      ],
    },
  });
});

test('it cannot add autograph plugin to asset by creator', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can add autograph plugin to asset by owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: owner.address,
              message: 'owner',
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: owner.address,
          message: 'owner',
        },
      ],
    },
  });
});

test('it cannot add autograph plugin to asset by 3rd party', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const autograph = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    authority: autograph,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: autograph.address,
              message: 'autograph',
            },
          ],
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [autograph, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add autograph to asset by unauthorized 3rd party', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const unauthed = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: unauthed,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
            {
              address: owner.address,
              message: 'owner',
            },
          ],
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [unauthed, payer]
  );

  await t.throwsAsync(result);
});

test('it can add additional autograph to asset via update by 3rd party', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const autograph = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: autograph,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
            {
              address: autograph.address,
              message: 'autograph',
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [autograph, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: payer.address,
          message: 'creator',
        },
        {
          address: autograph.address,
          message: 'autograph',
        },
      ],
    },
  });
});

test('it can remove autograph if owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [],
    },
  });
});

test('it cannot remove autograph if not owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const autograph = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: autograph,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [],
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [autograph, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot modify autograph message as signer', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              message: 'creator2',
            },
          ],
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot modify autograph message as owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
            {
              address: owner.address,
              message: 'owner',
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction1],
    [owner, payer]
  );

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              message: 'creator2',
            },
            {
              address: owner.address,
              message: 'owner2',
            },
          ],
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction2],
    [owner, payer]
  );

  await t.throwsAsync(result);
});

test('it can remove and add autographs as owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: owner,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: owner.address,
              message: 'owner',
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Owner',
      },
      signatures: [
        {
          address: owner.address,
          message: 'owner',
        },
      ],
    },
  });
});

test('it can remove and add autographs as delegate', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);
  const delegate = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
          ],
        },
        authority: {
          __kind: 'Address',
          address: delegate.address,
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: delegate,
    plugin: {
      __kind: 'Autograph',
      fields: [
        {
          signatures: [
            {
              address: delegate.address,
              message: 'delegate',
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [delegate, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    autograph: {
      authority: {
        type: 'Address',
        address: delegate.address,
      },
      signatures: [
        {
          address: delegate.address,
          message: 'delegate',
        },
      ],
    },
  });
});

test('it cannot add duplicate autographs', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const res = createAsset(rpc, payer, {
    owner: owner.address,
    plugins: [
      pluginAuthorityPair({
        type: 'Autograph',
        data: {
          signatures: [
            {
              address: payer.address,
              message: 'creator',
            },
            {
              address: payer.address,
              message: 'creator2',
            },
          ],
        },
      }),
    ],
  });

  await t.throwsAsync(res);
});
