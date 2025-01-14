import {
  Context,
  publicKey,
  Signer,
  TransactionBuilder,
} from '@metaplex-foundation/umi';
import {
  executeCollectionV1,
  executeV1,
  findAssetSignerPda,
  findCollectionSignerPda,
} from '../generated';

export type ExecuteArgs = Omit<
  Parameters<typeof executeV1>[1],
  'programId' | 'instructionData'
> & {
  builder: TransactionBuilder;
};

export const execute = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { asset, builder, ...args }: ExecuteArgs
) => {
  // Create a new builder to store the translated Execute instructions.
  let newBuilder = new TransactionBuilder();
  // We want to track the signers from the original IXes so they can be added to the Execute instructions.
  const signers: Signer[] = [];
  // Find the asset signer for the asset.
  const [assetSigner] = findAssetSignerPda(context, {
    asset: publicKey(asset),
  });

  // eslint-disable-next-line no-restricted-syntax
  for (const ix of builder.items) {
    let executeBuilder = newBuilder.add(
      // Forward on the original args.
      executeV1(context, {
        ...args,
        asset: publicKey(asset),
        assetSigner,
        // Forward the programID of the instruction being executed.
        programId: ix.instruction.programId,
        // Forward the data of the instruction being executed.
        instructionData: ix.instruction.data,
      })
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
    // Add the Execute instruction to the new builder.
    newBuilder = newBuilder.add(executeBuilder);
  }

  return newBuilder;
};

export type ExecuteCollectionArgs = Omit<
  Parameters<typeof executeCollectionV1>[1],
  'programId' | 'instructionData'
> & {
  builder: TransactionBuilder;
};

export const executeCollection = (
  context: Pick<Context, 'payer' | 'programs' | 'eddsa' | 'identity'>,
  { collection, builder, ...args }: ExecuteCollectionArgs
) => {
  // Create a new builder to store the translated Execute instructions.
  let newBuilder = new TransactionBuilder();
  // We want to track the signers from the original IXes so they can be added to the Execute instructions.
  const signers: Signer[] = [];
  // Find the asset signer for the asset.
  const [collectionSigner] = findCollectionSignerPda(context, {
    collection: publicKey(collection),
  });

  // eslint-disable-next-line no-restricted-syntax
  for (const ix of builder.items) {
    let executeBuilder = newBuilder.add(
      // Forward on the original args.
      executeCollectionV1(context, {
        ...args,
        collection: publicKey(collection),
        collectionSigner,
        // Forward the programID of the instruction being executed.
        programId: ix.instruction.programId,
        // Forward the data of the instruction being executed.
        instructionData: ix.instruction.data,
      })
        // Add the instruction keys as remaining accounts.
        .addRemainingAccounts(
          ix.instruction.keys.map((key) => {
            // If the key is the asset signer, then we don't want to add it to the Execute instruction
            // as a signer because it only gets signed in the CPI.
            if (key.pubkey === collectionSigner) {
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
      ...signers.filter((signer) => signer.publicKey !== collectionSigner)
    );
    // Set the modified builder items.
    executeBuilder = executeBuilder.setItems(executeBuilderItems);
    // Add the Execute instruction to the new builder.
    newBuilder = newBuilder.add(executeBuilder);
  }

  return newBuilder;
};
