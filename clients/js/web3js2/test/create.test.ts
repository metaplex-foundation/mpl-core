import {
  appendTransactionMessageInstruction,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  lamports,
  pipe,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from '@solana/kit';
import { fetchBaseAssetV1, getCreateV1Instruction } from '../src/generated';
import test from 'ava';

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

test('create an Asset', async (t) => {
  const httpProvider = 'http://127.0.0.1:8899';
  const wssProvider = 'ws://127.0.0.1:8900';
  const rpc = createSolanaRpc(httpProvider);
  const rpcSubscriptions = createSolanaRpcSubscriptions(wssProvider);

  const user = await generateKeyPairSigner();

  console.log('🔄 - Airdropping to user:', user.address);
  const airdropSignature = await rpc
    .requestAirdrop(user.address, lamports(1000000000n), {
      commitment: 'finalized',
    })
    .send();

  await sleep(2000);

  console.log('🔄 - Airdrop signature:', airdropSignature);

  const balance = await rpc.getBalance(user.address).send();
  t.is(balance.value, lamports(1000000000n), 'User should have 1 SOL');

  const asset = await generateKeyPairSigner();

  const createAsssetix = getCreateV1Instruction({
    asset,
    name: 'test',
    uri: 'test.com',
    payer: user,
  });

  const latestBlockhash = await rpc.getLatestBlockhash().send();

  const transaction = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayer(user.address, tx),
    (tx) =>
      setTransactionMessageLifetimeUsingBlockhash(latestBlockhash.value, tx),
    (tx) => appendTransactionMessageInstruction(createAsssetix, tx)
  );

  const signedTransaction =
    await signTransactionMessageWithSigners(transaction);
  const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({
    rpc,
    rpcSubscriptions,
  });

  try {
    await sendAndConfirmTransaction(signedTransaction, {
      commitment: 'confirmed',
    });
    const signature = getSignatureFromTransaction(signedTransaction);
    console.log('✅ - Create Asset Transaction:', signature);
  } catch (e) {
    console.error('Transfer failed:', e);
    t.fail('Asset creation failed');
  }

  console.log('🔄 - Fetching asset:', asset.address);

  const fetchedAsset = await fetchBaseAssetV1(rpc, asset.address);

  t.is(fetchedAsset.data.name, 'test');
  t.is(fetchedAsset.data.uri, 'test.com');
  t.deepEqual(fetchedAsset.data.owner, user.address);
  t.deepEqual(fetchedAsset.data.updateAuthority, {
    __kind: 'Address',
    fields: [user.address],
  });
});
