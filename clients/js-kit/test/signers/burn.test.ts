import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { AccountRole } from '@solana/instructions';
import { getBurnV1Instruction } from '../../src';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  sendAndConfirmInstructions,
} from '../_setup';

test('it can burn an asset as the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {});
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
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

test('it cannot burn an asset if the owner does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {});
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
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

test('it cannot burn an asset if an authority is provided but does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {});
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: authority,
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

test('it cannot burn an asset if an authority is provided and the payer does not sign', async (t) => {
  // Given a Umi instance and a new signer.
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {});
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getBurnV1Instruction({
    asset: asset.address,
    payer,
    authority: authority,
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
