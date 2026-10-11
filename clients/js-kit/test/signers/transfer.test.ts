import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { AccountRole } from '@solana/instructions';
import { getTransferV1Instruction } from '../../src';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  sendAndConfirmInstructions,
} from '../_setup';

test('it can transfer an asset as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    payer,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: newOwner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot transfer an asset if the owner does not sign', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    payer,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    payer,
    newOwner: newOwner.address,
  });

  // Create a new instruction with payer role downgraded from WRITABLE_SIGNER to WRITABLE
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 2 ? { ...account, role: AccountRole.WRITABLE } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [modifiedInstruction],
    []
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot transfer an asset if the authority is provided but does not sign', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    authority,
    payer,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    authority,
    payer,
    newOwner: newOwner.address,
  });

  // Create a new instruction with authority role downgraded from READONLY_SIGNER to READONLY
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 3 ? { ...account, role: AccountRole.READONLY } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [modifiedInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot transfer an asset if the authority is provided and the payer does not sign', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    authority,
    payer,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getTransferV1Instruction({
    asset: asset.address,
    authority,
    payer,
    newOwner: newOwner.address,
  });

  // Create a new instruction with payer role downgraded from WRITABLE_SIGNER to WRITABLE
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 2 ? { ...account, role: AccountRole.WRITABLE } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [modifiedInstruction],
    [authority]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});
