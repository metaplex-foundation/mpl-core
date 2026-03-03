import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import {
  createNoopSigner,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import { assertAsset, createUmi, DEFAULT_ASSET } from '../_setupRaw';
import { createAsset, createCollection } from '../_setupSdk';
import {
  addPlugin,
  addCollectionPlugin,
  CheckResult,
  execute,
  findAssetSignerPda,
  removePlugin,
  updatePlugin,
} from '../../src';

test('it can create an asset with an agent identity', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'UpdateAuthority',
        },
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
        uri: 'https://example.com/agent.json',
      },
    ],
  });
});

test('it can add an agent identity to an existing asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {});

  await addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'AgentIdentity',
      uri: 'https://example.com/agent.json',
      lifecycleChecks: {
        execute: [CheckResult.CAN_LISTEN],
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'UpdateAuthority',
        },
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
        uri: 'https://example.com/agent.json',
      },
    ],
  });
});

test('it cannot create a collection with an agent identity', async (t) => {
  const umi = await createUmi();

  const result = createCollection(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await t.throwsAsync(result, { name: 'InvalidPluginAdapterTarget' });
});

test('it cannot add an agent identity to a collection', async (t) => {
  const umi = await createUmi();

  const collection = await createCollection(umi, {});

  const result = addCollectionPlugin(umi, {
    collection: collection.publicKey,
    plugin: {
      type: 'AgentIdentity',
      uri: 'https://example.com/agent.json',
      lifecycleChecks: {
        execute: [CheckResult.CAN_LISTEN],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidPluginAdapterTarget' });
});

test('it cannot add a duplicate agent identity', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'AgentIdentity',
      uri: 'https://example.com/agent2.json',
      lifecycleChecks: {
        execute: [CheckResult.CAN_LISTEN],
      },
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'ExternalPluginAdapterAlreadyExists' });
});

test('it can update an agent identity uri', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      key: {
        type: 'AgentIdentity',
      },
      type: 'AgentIdentity',
      uri: 'https://example.com/updated-agent.json',
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'UpdateAuthority',
        },
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
        uri: 'https://example.com/updated-agent.json',
      },
    ],
  });
});

test('it can remove an agent identity', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await removePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'AgentIdentity',
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: undefined,
  });
});

test('it cannot add an agent identity as non-authority', async (t) => {
  const umi = await createUmi();
  const nonAuthority = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {});

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'AgentIdentity',
      uri: 'https://example.com/agent.json',
      lifecycleChecks: {
        execute: [CheckResult.CAN_LISTEN],
      },
    },
    authority: nonAuthority,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });
});

test('it can create an agent identity with multiple lifecycle checks', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN, CheckResult.CAN_APPROVE],
        },
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'UpdateAuthority',
        },
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN, CheckResult.CAN_APPROVE],
        },
        uri: 'https://example.com/agent.json',
      },
    ],
  });
});

test('it can create an agent identity with a specific plugin authority', async (t) => {
  const umi = await createUmi();
  const pluginAuthority = await generateSignerWithSol(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
        initPluginAuthority: {
          type: 'Address',
          address: pluginAuthority.publicKey,
        },
      },
    ],
  });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'Address',
          address: pluginAuthority.publicKey,
        },
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
        uri: 'https://example.com/agent.json',
      },
    ],
  });
});

test('it can update agent identity lifecycle checks', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      key: {
        type: 'AgentIdentity',
      },
      type: 'AgentIdentity',
      lifecycleChecks: {
        execute: [CheckResult.CAN_LISTEN, CheckResult.CAN_APPROVE],
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'UpdateAuthority',
        },
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN, CheckResult.CAN_APPROVE],
        },
        uri: 'https://example.com/agent.json',
      },
    ],
  });
});

test('it can update agent identity uri and lifecycle checks simultaneously', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      key: {
        type: 'AgentIdentity',
      },
      type: 'AgentIdentity',
      uri: 'https://example.com/agent-v3.json',
      lifecycleChecks: {
        execute: [
          CheckResult.CAN_LISTEN,
          CheckResult.CAN_APPROVE,
          CheckResult.CAN_REJECT,
        ],
      },
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    agentIdentities: [
      {
        type: 'AgentIdentity',
        authority: {
          type: 'UpdateAuthority',
        },
        lifecycleChecks: {
          execute: [
            CheckResult.CAN_LISTEN,
            CheckResult.CAN_APPROVE,
            CheckResult.CAN_REJECT,
          ],
        },
        uri: 'https://example.com/agent-v3.json',
      },
    ],
  });
});

test('it can execute on an asset with agent identity', async (t) => {
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AgentIdentity',
        uri: 'https://example.com/agent.json',
        lifecycleChecks: {
          execute: [CheckResult.CAN_LISTEN],
        },
      },
    ],
  });

  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  await umi.rpc.airdrop(publicKey(assetSigner), sol(1));

  await execute(umi, {
    asset,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: recipient.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  t.deepEqual(afterRecipientBalance, sol(0.5));
});
