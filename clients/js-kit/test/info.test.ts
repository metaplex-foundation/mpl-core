import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import { fetchEncodedAccount } from '@solana/kit';
import { getCreateV1Instruction, DataState } from '../src';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from './_setup';

test('fetch account info for account state', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAddress, payer]
  );

  const account = await fetchEncodedAccount(rpc, assetAddress.address);
  if (account.exists) {
    console.log(`Account Size ${account.data.length} bytes`);
  }

  t.pass();
});

test.skip('fetch account info for ledger state', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    logWrapper: address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAddress, payer]
  );

  const account = await fetchEncodedAccount(rpc, assetAddress.address);
  if (account.exists) {
    // console.log(`Account Size ${account.data.length} bytes`);
  }

  t.pass();
});
