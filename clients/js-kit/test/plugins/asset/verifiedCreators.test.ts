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

test('it can create asset with verified creators plugin', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: { signatures: [] },
      }),
    ],
  });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [],
    },
  });
});

test('it cannot create asset with verified creators plugin and unauthorized signature', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();

  const res = createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: true,
            },
          ],
        },
      }),
    ],
  });

  await t.throwsAsync(res);
});

test('it can create asset with verified creators plugin with authorized signature', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: payer.address,
              verified: true,
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
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: payer.address,
          verified: true,
        },
      ],
    },
  });
});

test('it cannot add verified creator plugin to asset by owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    owner,
  });

  const instruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer: owner,
    authority: owner,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: owner.address,
              verified: true,
            },
          ],
        },
      ],
    },
  });

  const res = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [owner]
  );

  await t.throwsAsync(res);
});

test('it can create asset with verified creators plugin with unverified signatures and then verify', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: payer.address,
              verified: false,
            },
            {
              address: creator.address,
              verified: false,
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
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: payer.address,
          verified: false,
        },
        {
          address: creator.address,
          verified: false,
        },
      ],
    },
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: creator,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              verified: false,
            },
            {
              address: creator.address,
              verified: true,
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
    [creator, payer]
  );

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              verified: true,
            },
            {
              address: creator.address,
              verified: true,
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction2],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: payer.address,
          verified: true,
        },
        {
          address: creator.address,
          verified: true,
        },
      ],
    },
  });
});

test('it can unverify signature verified creator plugin', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: false,
            },
          ],
        },
      }),
    ],
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: creator,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: creator.address,
              verified: true,
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
    [creator, payer]
  );

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: creator,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: creator.address,
              verified: false,
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction2],
    [creator, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.address,
          verified: false,
        },
      ],
    },
  });
});

test('it cannot verify a verified creator plugin with unauthorized signature', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();
  const unauthed = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: false,
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
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: creator.address,
              verified: true,
            },
          ],
        },
      ],
    },
  });

  const res = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [unauthed, payer]
  );

  await t.throwsAsync(res);
});

test('it cannot remove verified creator plugin signture with unauthorized signature', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();
  const unauthed = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: false,
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
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [],
        },
      ],
    },
  });

  const res = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [unauthed, payer]
  );

  await t.throwsAsync(res);
});

test('it can remove and add unverified creator plugin signature with update auth', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();
  const creator2 = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: false,
            },
          ],
        },
      }),
    ],
  });

  const instruction = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: payer.address,
              verified: true,
            },
            {
              address: creator2.address,
              verified: false,
            },
          ],
        },
      ],
    },
  });

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], [payer]);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: payer.address,
          verified: true,
        },
      ],
    },
  });
});

test('it cannot remove verified creator plugin signature with update auth', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: false,
            },
          ],
        },
      }),
    ],
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: creator,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: creator.address,
              verified: true,
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
    [creator, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.address,
          verified: true,
        },
      ],
    },
  });

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [],
        },
      ],
    },
  });

  const res = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction2],
    [payer]
  );

  await t.throwsAsync(res);
});

test('it cannot unverify verified creator plugin signature with update auth', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();
  const creator = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: creator.address,
              verified: false,
            },
          ],
        },
      }),
    ],
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    authority: creator,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: creator.address,
              verified: true,
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
    [creator, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    updateAuthority: { type: 'Address', address: payer.address },
    verifiedCreators: {
      authority: {
        type: 'UpdateAuthority',
      },
      signatures: [
        {
          address: creator.address,
          verified: true,
        },
      ],
    },
  });

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'VerifiedCreators',
      fields: [
        {
          signatures: [
            {
              address: creator.address,
              verified: false,
            },
          ],
        },
      ],
    },
  });

  const res = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [updateInstruction2],
    [payer]
  );

  await t.throwsAsync(res);
});

test('it cannot add duplicate verified creator signatures', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  const res = createAsset(rpc, payer, {
    owner,
    plugins: [
      pluginAuthorityPair({
        type: 'VerifiedCreators',
        data: {
          signatures: [
            {
              address: payer.address,
              verified: false,
            },
            {
              address: payer.address,
              verified: false,
            },
          ],
        },
      }),
    ],
  });

  await t.throwsAsync(res);
});
