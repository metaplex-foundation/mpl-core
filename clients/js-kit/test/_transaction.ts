/**
 * Transaction sending utilities for tests using @solana/kit pipes
 */

import { pipe } from '@solana/functional';
import type { Rpc } from '@solana/rpc';
import type { SolanaRpcApi } from '@solana/rpc';
import type { RpcSubscriptions } from '@solana/rpc-subscriptions';
import type { SolanaRpcSubscriptionsApi } from '@solana/rpc-subscriptions';
import type { TransactionSigner } from '@solana/signers';
import { isKeyPairSigner } from '@solana/signers';
import type { IInstruction } from '@solana/instructions';
import {
  appendTransactionMessageInstructions,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
} from '@solana/transaction-messages';
import {
  compileTransaction,
  signTransaction,
  assertIsTransactionWithinSizeLimit,
} from '@solana/transactions';
import { sendAndConfirmTransactionFactory } from '@solana/kit';

/**
 * Send and confirm a single instruction using @solana/kit pipes
 */
export async function sendAndConfirm(
  rpc: Rpc<SolanaRpcApi>,
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>,
  instruction: IInstruction,
  signers: TransactionSigner[]
): Promise<void> {
  return sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], signers);
}

/**
 * Send and confirm multiple instructions using @solana/kit pipes
 */
export async function sendAndConfirmInstructions(
  rpc: Rpc<SolanaRpcApi>,
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>,
  instructions: readonly IInstruction[],
  signers: TransactionSigner[]
): Promise<void> {
  if (signers.length === 0) {
    throw new Error('At least one signer is required');
  }

  if (instructions.length === 0) {
    throw new Error('At least one instruction is required');
  }

  // Get latest blockhash
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  // Build transaction message using pipes - this is the @solana/kit way
  // Use the second signer as fee payer (typically the payer with SOL)
  // First signer is often the new account being created
  const feePayer = signers.length > 1 ? signers[1] : signers[0];
  const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayer(feePayer.address, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) => appendTransactionMessageInstructions(instructions, tx)
  );

  // Compile transaction
  const transaction = compileTransaction(transactionMessage);

  // Sign transaction - all signers must be KeyPairSigners in test environment
  const keyPairs = signers.map((signer) => {
    if (!isKeyPairSigner(signer)) {
      throw new Error(
        `Expected KeyPairSigner but got ${typeof signer}. ` +
        'All signers in tests must be KeyPairSigners from generateKeyPairSigner().'
      );
    }
    return signer.keyPair;
  });
  const signedTransaction = await signTransaction(keyPairs, transaction);

  // Assert size limit (narrows type to SendableTransaction)
  assertIsTransactionWithinSizeLimit(signedTransaction);

  // Send and confirm using the factory
  const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({
    rpc,
    rpcSubscriptions,
  });

  await sendAndConfirmTransaction(signedTransaction, {
    commitment: 'confirmed',
  });
}
