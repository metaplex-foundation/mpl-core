import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { AccountRole } from '@solana/instructions';
import { getUpdateV1Instruction } from '../../src';
import {
  createAsset,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  DEFAULT_ASSET,
  sendAndConfirmInstructions,
} from '../_setup';

test('it can update an asset as the update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Update',
    newUri: '',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    name: 'Test Update',
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot update an asset if the payer does not sign', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    newName: 'Test Update',
    newUri: '',
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
    name: DEFAULT_ASSET.name,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot update an asset if the authority is provided but does not sign', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority,
    newName: 'Test Update',
    newUri: '',
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
    name: DEFAULT_ASSET.name,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot update an asset if the authority is provided and the payer does not sign', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const authority = await generateKeyPairSigner();
  const owner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, {
    owner: owner.address,
  });
  await assertAsset(t, rpc, {
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const instruction = getUpdateV1Instruction({
    asset: asset.address,
    payer,
    authority,
    newName: 'Test Update',
    newUri: '',
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
    name: DEFAULT_ASSET.name,
    asset: asset.address,
    owner: owner,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});
