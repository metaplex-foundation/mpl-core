import {
  Context,
  Instruction,
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
  // We want to track the signers from the original IXes so they can be added to the Execute instructions.
  const signers: Signer[] = [];

  let builder: TransactionBuilder = new TransactionBuilder();
  if (args.instructions instanceof TransactionBuilder) {
    builder = args.instructions;
  } else if (args.instructions) {
    args.instructions.forEach((instruction) => {
      const ixSigners: Signer[] = [];
      instruction.keys.forEach((key) => {
        const signer = signers.find(
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
    const baseBuilder = executeV1(context, {
      ...args,
      asset: args.asset.publicKey,
      collection: args.collection?.publicKey,
      assetSigner,
      // Forward the programID of the instruction being executed.
      programId: ix.instruction.programId,
      // Forward the data of the instruction being executed.
      instructionData: ix.instruction.data,
    });

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
    // Add the signers to the Execute instruction.
    executeBuilderItems[0].signers.push(
      // Add the signers to the Execute instruction, filtering out the asset signer.
      ...signers.filter((signer) => signer.publicKey !== assetSigner)
    );
    // Set the modified builder items.
    executeBuilder = executeBuilder.setItems(executeBuilderItems);
  }

  return executeBuilder;
};
