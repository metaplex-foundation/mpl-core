import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { type Address, address } from '@solana/addresses';
import { lamports } from '@solana/kit';
import {
  getStructEncoder,
  getU32Encoder,
  getU64Encoder,
} from '@solana/codecs';
import type { IInstruction } from '@solana/instructions';
import { getExecuteV1InstructionAsync, findAssetSignerPda } from '../src';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  airdrop,
  sendAndConfirmInstructions,
  DEFAULT_ASSET,
} from './_setup';

// Helper function to create a transfer SOL instruction (System Program transfer)
function createTransferSolInstruction(
  source: Address,
  destination: Address,
  amount: bigint
): IInstruction {
  const SYSTEM_PROGRAM_ADDRESS = '11111111111111111111111111111111';
  const TRANSFER_DISCRIMINATOR = 2;

  const encoder = getStructEncoder([
    ['discriminator', getU32Encoder()],
    ['lamports', getU64Encoder()],
  ]);

  const data = encoder.encode({
    discriminator: TRANSFER_DISCRIMINATOR,
    lamports: amount,
  });

  return {
    programAddress: address(SYSTEM_PROGRAM_ADDRESS),
    accounts: [
      { address: source, role: 1 /* WritableSigner */ },
      { address: destination, role: 2 /* Writable */ },
    ],
    data: new Uint8Array(data),
  };
}

test('it can execute for an asset as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const recipient = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });
  await airdrop(rpc, assetSigner, 1_000_000_000n);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const beforeAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const beforeRecipientBalance = await rpc.getBalance(recipient.address).send();
  const beforeAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(beforeAssetSignerBalance.value, lamports(1_000_000_000n));
  t.deepEqual(beforeRecipientBalance.value, lamports(0n));
  t.deepEqual(beforeAssetBalance.value, lamports(3156480n));

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  const afterAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const afterRecipientBalance = await rpc.getBalance(recipient.address).send();
  const afterAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(afterAssetSignerBalance.value, lamports(500_000_000n));
  t.deepEqual(afterRecipientBalance.value, lamports(500_000_000n));
  t.deepEqual(afterAssetBalance.value, lamports(3156480n + 48720n));
});

test('it can execute multiple instructions for an asset as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const recipient = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });
  await airdrop(rpc, assetSigner, 1_000_000_000n);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const beforeAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const beforeRecipientBalance = await rpc.getBalance(recipient.address).send();
  const beforeAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(beforeAssetSignerBalance.value, lamports(1_000_000_000n));
  t.deepEqual(beforeRecipientBalance.value, lamports(0n));
  t.deepEqual(beforeAssetBalance.value, lamports(3156480n));

  // Create two transfer instructions
  const transfer1 = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    250_000_000n
  );

  const transfer2 = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    250_000_000n
  );

  // Combine instruction data
  const combinedData = new Uint8Array(transfer1.data!.length + transfer2.data!.length);
  combinedData.set(new Uint8Array(transfer1.data!), 0);
  combinedData.set(new Uint8Array(transfer2.data!), transfer1.data!.length);

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    instructionData: combinedData,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  const afterAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const afterRecipientBalance = await rpc.getBalance(recipient.address).send();
  const afterAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(afterAssetSignerBalance.value, lamports(500_000_000n));
  t.deepEqual(afterRecipientBalance.value, lamports(500_000_000n));
  t.deepEqual(afterAssetBalance.value, lamports(3156480n + 48720n * 2n));
});

test('it can execute for an asset as the owner with an Instruction', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const recipient = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });
  await airdrop(rpc, assetSigner, 1_000_000_000n);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const beforeAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const beforeRecipientBalance = await rpc.getBalance(recipient.address).send();
  const beforeAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(beforeAssetSignerBalance.value, lamports(1_000_000_000n));
  t.deepEqual(beforeRecipientBalance.value, lamports(0n));
  t.deepEqual(beforeAssetBalance.value, lamports(3156480n));

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  const afterAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const afterRecipientBalance = await rpc.getBalance(recipient.address).send();
  const afterAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(afterAssetSignerBalance.value, lamports(500_000_000n));
  t.deepEqual(afterRecipientBalance.value, lamports(500_000_000n));
  t.deepEqual(afterAssetBalance.value, lamports(3156480n + 48720n));
});

test('it can execute for an asset as the owner with an Instruction[]', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const recipient = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer);
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });
  await airdrop(rpc, assetSigner, 1_000_000_000n);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });

  const beforeAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const beforeRecipientBalance = await rpc.getBalance(recipient.address).send();
  const beforeAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(beforeAssetSignerBalance.value, lamports(1_000_000_000n));
  t.deepEqual(beforeRecipientBalance.value, lamports(0n));
  t.deepEqual(beforeAssetBalance.value, lamports(3156480n));

  // Create two transfer instructions
  const transfer1 = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    250_000_000n
  );

  const transfer2 = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    250_000_000n
  );

  // Combine instruction data
  const combinedData = new Uint8Array(transfer1.data!.length + transfer2.data!.length);
  combinedData.set(new Uint8Array(transfer1.data!), 0);
  combinedData.set(new Uint8Array(transfer2.data!), transfer1.data!.length);

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    instructionData: combinedData,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  const afterAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const afterRecipientBalance = await rpc.getBalance(recipient.address).send();
  const afterAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(afterAssetSignerBalance.value, lamports(500_000_000n));
  t.deepEqual(afterRecipientBalance.value, lamports(500_000_000n));
  t.deepEqual(afterAssetBalance.value, lamports(3156480n + 48720n * 2n));
});

test('it cannot execute for an asset if not the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const attacker = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer);
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    attacker.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer: attacker,
    authority: attacker,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [attacker]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot execute for an asset as the update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const asset = await createAsset(rpc, payer, { owner: newOwner.address });
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    newOwner.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    authority: payer,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: newOwner.address,
    updateAuthority: { type: 'Address', address: payer.address },
  });
});

test('it cannot execute for an asset in collection if no collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();

  const { asset } = await createAssetWithCollection(rpc, payer, {});
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    newOwner.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it can execute for an asset in collection as the owner', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const recipient = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });
  await airdrop(rpc, assetSigner, 1_000_000_000n);

  const beforeAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const beforeRecipientBalance = await rpc.getBalance(recipient.address).send();
  const beforeAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(beforeAssetSignerBalance.value, lamports(1_000_000_000n));
  t.deepEqual(beforeRecipientBalance.value, lamports(0n));
  t.deepEqual(beforeAssetBalance.value, lamports(3156480n));

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    collection: collection.address,
    payer,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  const afterAssetSignerBalance = await rpc.getBalance(assetSigner).send();
  const afterRecipientBalance = await rpc.getBalance(recipient.address).send();
  const afterAssetBalance = await rpc.getBalance(asset.address).send();

  t.deepEqual(afterAssetSignerBalance.value, lamports(500_000_000n));
  t.deepEqual(afterRecipientBalance.value, lamports(500_000_000n));
  t.deepEqual(afterAssetBalance.value, lamports(3156480n + 48720n));
});

test('it cannot transfer asset in collection with the wrong collection', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const recipient = await generateKeyPairSigner();

  const { asset } = await createAssetWithCollection(rpc, payer, {});
  const wrongCollection = await createCollection(rpc, payer);
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    recipient.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    collection: wrongCollection.address,
    payer,
    instructionData: new Uint8Array(transferInstruction.data!),
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid system program', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const newOwner = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  const { asset, collection } = await createAssetWithCollection(rpc, payer, {});
  const [assetSigner] = await findAssetSignerPda({ asset: asset.address });

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer,
    updateAuthority: { type: 'Collection', address: collection.address },
  });

  const transferInstruction = createTransferSolInstruction(
    assetSigner,
    newOwner.address,
    500_000_000n
  );

  const executeInstruction = await getExecuteV1InstructionAsync({
    asset: asset.address,
    payer,
    instructionData: new Uint8Array(transferInstruction.data!),
    systemProgram: fakeSystemProgram.address,
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [executeInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});
