import {
  createNoopSigner,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';

import { transferSol } from '@metaplex-foundation/mpl-toolbox';
import { executeCollection, findCollectionSignerPda } from '../src';
import { assertCollection, createUmi } from './_setupRaw';
import { createCollection } from './_setupSdk';

test('it can execute for a collection as the update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const collection = await createCollection(umi);
  const collectionSigner = findCollectionSignerPda(umi, {
    collection: collection.publicKey,
  });
  await umi.rpc.airdrop(publicKey(collectionSigner), sol(1));

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const beforeCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(beforeCollectionSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00154512));

  await executeCollection(umi, {
    collection: collection.publicKey,
    builder: transferSol(umi, {
      source: createNoopSigner(publicKey(collectionSigner)),
      destination: recipient.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi, { send: { skipPreflight: true } });

  const afterCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterCollectionBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(afterCollectionSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterCollectionBalance, sol(0.00154512));
});

test('it can execute multiple instructions for a collection as the update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const collection = await createCollection(umi);
  const collectionSigner = findCollectionSignerPda(umi, {
    collection: collection.publicKey,
  });
  await umi.rpc.airdrop(publicKey(collectionSigner), sol(1));

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const beforeCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(beforeCollectionSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00154512));

  await executeCollection(umi, {
    collection: collection.publicKey,
    builder: transferSol(umi, {
      source: createNoopSigner(publicKey(collectionSigner)),
      destination: recipient.publicKey,
      amount: sol(0.25),
    }).add(
      transferSol(umi, {
        source: createNoopSigner(publicKey(collectionSigner)),
        destination: recipient.publicKey,
        amount: sol(0.25),
      })
    ),
  }).sendAndConfirm(umi);

  const afterCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterCollectionBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(afterCollectionSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterCollectionBalance, sol(0.00154512));
});

test('it can execute for a collection as the update authority with an Instruction', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const collection = await createCollection(umi);
  const collectionSigner = findCollectionSignerPda(umi, {
    collection: collection.publicKey,
  });
  await umi.rpc.airdrop(publicKey(collectionSigner), sol(1));

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const beforeCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(beforeCollectionSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00154512));

  const instruction = transferSol(umi, {
    source: createNoopSigner(publicKey(collectionSigner)),
    destination: recipient.publicKey,
    amount: sol(0.5),
  }).getInstructions()[0];

  await executeCollection(umi, {
    collection: collection.publicKey,
    instruction,
    signers: [createNoopSigner(publicKey(collectionSigner))],
  }).sendAndConfirm(umi);

  const afterCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterCollectionBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(afterCollectionSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterCollectionBalance, sol(0.00154512));
});

test('it can execute for a collection as the update authority with an Instruction[]', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const recipient = generateSigner(umi);

  const collection = await createCollection(umi);
  const collectionSigner = findCollectionSignerPda(umi, {
    collection: collection.publicKey,
  });
  await umi.rpc.airdrop(publicKey(collectionSigner), sol(1));

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });

  const beforeCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const beforeRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const beforeAssetBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(beforeCollectionSignerBalance, sol(1));
  t.deepEqual(beforeRecipientBalance, sol(0));
  t.deepEqual(beforeAssetBalance, sol(0.00154512));

  const instructions = transferSol(umi, {
    source: createNoopSigner(publicKey(collectionSigner)),
    destination: recipient.publicKey,
    amount: sol(0.25),
  })
    .add(
      transferSol(umi, {
        source: createNoopSigner(publicKey(collectionSigner)),
        destination: recipient.publicKey,
        amount: sol(0.25),
      })
    )
    .getInstructions();

  await executeCollection(umi, {
    collection: collection.publicKey,
    instructions,
    signers: [createNoopSigner(publicKey(collectionSigner))],
  }).sendAndConfirm(umi);

  const afterCollectionSignerBalance = await umi.rpc.getBalance(
    publicKey(collectionSigner)
  );
  const afterRecipientBalance = await umi.rpc.getBalance(publicKey(recipient));
  const afterCollectionBalance = await umi.rpc.getBalance(
    publicKey(collection.publicKey)
  );

  t.deepEqual(afterCollectionSignerBalance, sol(0.5));
  t.deepEqual(afterRecipientBalance, sol(0.5));
  t.deepEqual(afterCollectionBalance, sol(0.00154512));
});

test('it cannot execute for a collection if not the update authority', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const collection = await createCollection(umi);
  const collectionSigner = findCollectionSignerPda(umi, {
    collection: collection.publicKey,
  });

  const result = executeCollection(umi, {
    collection: collection.publicKey,
    authority: attacker,
    builder: transferSol(umi, {
      source: createNoopSigner(publicKey(collectionSigner)),
      destination: attacker.publicKey,
      amount: sol(0.5),
    }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});
