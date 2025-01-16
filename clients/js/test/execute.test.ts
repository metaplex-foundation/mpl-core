import {
  addAmounts,
  createNoopSigner,
  generateSigner,
  lamports,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';

import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import { execute, findAssetSignerPda } from '../src';
import { assertAsset, createAsset, createUmi } from './_setupRaw';
import { createAssetWithCollection, createCollection } from './_setupSdk';

test('it can execute for an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const asset = await createAsset(umi);
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  await umi.rpc.airdrop(publicKey(assetSigner), sol(1));

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const beforeAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(beforeAssetSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00315648));

  await execute(umi, {
    asset,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: recipient.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  const afterAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(afterAssetSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterAssetBalance, addAmounts(sol(0.00315648), lamports(48720)));
});

test('it can execute multiple instructions for an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const asset = await createAsset(umi);
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  await umi.rpc.airdrop(publicKey(assetSigner), sol(1));

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const beforeAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(beforeAssetSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00315648));

  await execute(umi, {
    asset,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: recipient.publicKey,
      amount: sol(0.25),
    }).add(
      transferSol(umi, {
        source: createNoopSigner(publicKey(assetSigner)),
        destination: recipient.publicKey,
        amount: sol(0.25),
      })
    ),
  }).sendAndConfirm(umi);

  const afterAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(afterAssetSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(
    afterAssetBalance,
    addAmounts(sol(0.00315648), lamports(48720 * 2))
  );
});

test('it can execute for an asset as the owner with an Instruction', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const asset = await createAsset(umi);
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  await umi.rpc.airdrop(publicKey(assetSigner), sol(1));

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const beforeAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(beforeAssetSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00315648));

  const instruction = transferSol(umi, {
    source: createNoopSigner(publicKey(assetSigner)),
    destination: recipient.publicKey,
    amount: sol(0.5),
  }).getInstructions()[0];

  await execute(umi, {
    asset,
    instructions: [instruction],
    signers: [createNoopSigner(publicKey(assetSigner))],
  }).sendAndConfirm(umi);

  const afterAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(afterAssetSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterAssetBalance, addAmounts(sol(0.00315648), lamports(48720)));
});

test('it can execute for an asset as the owner with an Instruction[]', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const asset = await createAsset(umi);
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  await umi.rpc.airdrop(publicKey(assetSigner), sol(1));

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  const beforeAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(beforeAssetSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00315648));

  const instructions = transferSol(umi, {
    source: createNoopSigner(publicKey(assetSigner)),
    destination: recipient.publicKey,
    amount: sol(0.25),
  })
    .add(
      transferSol(umi, {
        source: createNoopSigner(publicKey(assetSigner)),
        destination: recipient.publicKey,
        amount: sol(0.25),
      })
    )
    .getInstructions();

  await execute(umi, {
    asset,
    instructions,
    signers: [createNoopSigner(publicKey(assetSigner))],
  }).sendAndConfirm(umi);

  const afterAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(afterAssetSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(
    afterAssetBalance,
    addAmounts(sol(0.00315648), lamports(48720 * 2))
  );
});

test('it cannot execute for an asset if not the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi);
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });

  const result = execute(umi, {
    asset,
    authority: attacker,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: attacker.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot execute for an asset as the update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, { owner: newOwner.publicKey });
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });

  const result = execute(umi, {
    asset,
    authority: umi.identity,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: newOwner.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('it cannot execute for an asset in collection if no collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {});
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });

  const result = execute(umi, {
    asset,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: newOwner.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'MissingCollection' });
});

test('it can execute for an asset in collection as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  await umi.rpc.airdrop(publicKey(assetSigner), sol(1));

  const beforeAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(beforeAssetSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00315648));

  await execute(umi, {
    asset,
    collection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: recipient.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  const afterAssetSignerBalance = await umi.rpc.getBalance(
    publicKey(assetSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterAssetBalance = await umi.rpc.getBalance(
    publicKey(asset.publicKey)
  );

  t.deepEqual(afterAssetSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterAssetBalance, addAmounts(sol(0.00315648), lamports(48720)));
});

test('it cannot transfer asset in collection with the wrong collection', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {});
  const wrongCollection = await createCollection(umi);
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });
  const result = execute(umi, {
    asset,
    collection: wrongCollection,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: recipient.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('it cannot use an invalid system program', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const fakeSystemProgram = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});
  const assetSigner = findAssetSignerPda(umi, { asset: asset.publicKey });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });

  const result = execute(umi, {
    asset,
    instructions: transferSol(umi, {
      source: createNoopSigner(publicKey(assetSigner)),
      destination: newOwner.publicKey,
      amount: sol(0.5),
    }),
    systemProgram: fakeSystemProgram.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidSystemProgram' });
});
