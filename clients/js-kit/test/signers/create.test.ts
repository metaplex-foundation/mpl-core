import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { AccountRole } from '@solana/instructions';
import { getCreateV1Instruction } from '../../src';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';

test('it can create a new asset', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    ...DEFAULT_ASSET,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [assetAddress, payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: assetAddress.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot create a new asset if the asset does not sign', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    ...DEFAULT_ASSET,
  });

  // Create a new instruction with asset role downgraded from WRITABLE_SIGNER to WRITABLE
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

test('it cannot create a new asset if the payer does not sign', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    ...DEFAULT_ASSET,
  });

  // Create a new instruction with payer role downgraded from WRITABLE_SIGNER to WRITABLE
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 3 ? { ...account, role: AccountRole.WRITABLE } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [modifiedInstruction],
    [assetAddress]
  );

  await t.throwsAsync(result);
});

test('it fails if an authority is provided and it does not sign', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    authority,
    payer,
    ...DEFAULT_ASSET,
  });

  // Create a new instruction with authority role downgraded from READONLY_SIGNER to READONLY
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 2 ? { ...account, role: AccountRole.READONLY } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [modifiedInstruction],
    [assetAddress, payer]
  );

  await t.throwsAsync(result);
});

test('it fails even if an authority signs but the payer does not sign', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    authority,
    payer,
    ...DEFAULT_ASSET,
  });

  // Create a new instruction with payer role downgraded from WRITABLE_SIGNER to WRITABLE
  const modifiedAccounts = instruction.accounts.map((account, index) =>
    index === 3 ? { ...account, role: AccountRole.WRITABLE } : account
  );
  const modifiedInstruction = { ...instruction, accounts: modifiedAccounts };

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [modifiedInstruction],
    [assetAddress, authority]
  );

  await t.throwsAsync(result);
});
