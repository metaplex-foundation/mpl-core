import {
  Context,
  Instruction,
  Pda,
  publicKey,
  PublicKey,
  Signer,
  TransactionBuilder,
} from '@metaplex-foundation/umi';

import {
  AssetV1,
  CollectionV1,
  executeV1,
  findAssetSignerPda,
} from '../generated';

export type ExecuteInput = TransactionBuilder | Instruction[];

export type ExecuteArgs = Omit<
  Parameters<typeof executeV1>[1],
  'programId' | 'instructionData' | 'asset' | 'collection'
> & {
  asset: Pick<AssetV1, 'publicKey'>;
  collection?: Pick<CollectionV1, 'publicKey'>;
  instructions: ExecuteInput;
  signers?: Signer[];
  /** Optional execution delegate record account to pass as the first remaining account. */
  executionDelegateRecord?: PublicKey | Pda;
};

export const execute = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  args: ExecuteArgs
) => executeCommon(context, args);

const executeCommon = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  args: ExecuteArgs
) => {
  // Create a new builder to store the translated Execute instructions.
  let executeBuilder = new TransactionBuilder();
  // Caller-level signers provided via args.signers (used for Instruction[] path).
  const callerSigners: Signer[] = [...(args.signers || [])];

  let builder: TransactionBuilder = new TransactionBuilder();
  // Duck-type check instead of instanceof to avoid cross-realm/bundler breakage
  // when multiple copies of the umi package are resolved.
  if ('getInstructions' in args.instructions) {
    builder = args.instructions;
  } else if (Array.isArray(args.instructions)) {
    args.instructions.forEach((instruction: Instruction) => {
      const ixSigners: Signer[] = [];
      instruction.keys
        .filter((key) => key.isSigner)
        .forEach((key) => {
          const signer = callerSigners.find(
            (signerKey) => signerKey.publicKey === key.pubkey
          );
          if (signer) {
            ixSigners.push(signer);
          }
        });
      builder = builder.add({
        instruction,
        signers: ixSigners,
        bytesCreatedOnChain: 0,
      });
    });
  } else {
    throw new Error('No builder or instructions provided');
  }

  // eslint-disable-next-line no-restricted-syntax
  for (const ix of builder.items) {
    const [assetSigner] = findAssetSignerPda(context, {
      asset: args.asset.publicKey,
    });
    let baseBuilder = executeV1(context, {
      ...args,
      asset: args.asset.publicKey,
      collection: args.collection?.publicKey,
      assetSigner,
      // Forward the programID of the instruction being executed.
      programId: ix.instruction.programId,
      // Forward the data of the instruction being executed.
      instructionData: ix.instruction.data,
    });

    // If an executionDelegateRecord is provided, prepend it as the first
    // remaining account so the on-chain program can read it during validation.
    if (args.executionDelegateRecord) {
      baseBuilder = baseBuilder.addRemainingAccounts([
        {
          pubkey: publicKey(args.executionDelegateRecord),
          isSigner: false,
          isWritable: false,
        },
      ]);
    }

    executeBuilder = executeBuilder.add(
      baseBuilder
        // Add the instruction keys as remaining accounts.
        .addRemainingAccounts(
          ix.instruction.keys.map((key) => {
            // If the key is the asset signer, then we don't want to add it to the Execute instruction
            // as a signer because it only gets signed in the CPI.
            if (key.pubkey === assetSigner) {
              return {
                pubkey: key.pubkey,
                isSigner: false,
                isWritable: key.isWritable,
              };
            }
            return key;
          })
        )
    );

    // Capture the builder items so they can be modified.
    const executeBuilderItems = executeBuilder.items;
    // Merge caller-level signers with this item's signers, filtering out the asset signer.
    const itemSigners = [...callerSigners, ...ix.signers].filter(
      (signer) => signer.publicKey !== assetSigner
    );
    executeBuilderItems[executeBuilderItems.length - 1].signers.push(
      ...itemSigners
    );
    // Set the modified builder items.
    executeBuilder = executeBuilder.setItems(executeBuilderItems);
  }

  return executeBuilder;
};
