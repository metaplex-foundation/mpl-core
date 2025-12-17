import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { AccountRole } from '@solana/instructions';
import { getCreateCollectionV1Instruction } from '../../src';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertCollection,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../_setup';

test('it can create a new collection', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();

  const instruction = getCreateCollectionV1Instruction({
    collection: collectionAddress,
    payer,
    ...DEFAULT_COLLECTION,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [collectionAddress, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collectionAddress.address,
    updateAuthority: payer.address,
  });
});

test('it cannot create a new collection if the collection does not sign', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();

  const instruction = getCreateCollectionV1Instruction({
    collection: collectionAddress,
    payer,
    ...DEFAULT_COLLECTION,
  });

  // Create a new instruction with collection role downgraded from WRITABLE_SIGNER to WRITABLE
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 0 ? { ...account, role: AccountRole.WRITABLE } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [modifiedInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot create a new collection if the payer does not sign', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();

  const instruction = getCreateCollectionV1Instruction({
    collection: collectionAddress,
    payer,
    ...DEFAULT_COLLECTION,
  });

  // Create a new instruction with payer role downgraded from WRITABLE_SIGNER to WRITABLE
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 2 ? { ...account, role: AccountRole.WRITABLE } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [modifiedInstruction],
    [collectionAddress]
  );

  await t.throwsAsync(result);
});
