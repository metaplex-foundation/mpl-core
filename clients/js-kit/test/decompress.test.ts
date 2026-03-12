import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import { address } from '@solana/addresses';
import {
  getCreateV1Instruction,
  getCompressV1Instruction,
  getDecompressV1Instruction,
  DataState,
  fetchAssetV1,
  fetchHashedAssetV1,
  getAssetV1Encoder,
  getHashedAssetSchemaEncoder,
  type HashedAssetSchema,
  Key,
} from '../src';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  sendAndConfirmInstructions,
} from './_setup';
import { hash } from '../src/hash';
import { updateAuthorityToBase } from '../src/plugins';

test.skip('it can decompress a previously compressed asset as the owner', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();

  // Create asset in account state
  const createInstruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [createInstruction],
    [assetAddress, payer]
  );

  // Verify asset was created correctly
  const beforeAsset = await fetchAssetV1(rpc, assetAddress.address);
  t.is(beforeAsset.data.key, Key.AssetV1);
  t.is(beforeAsset.data.owner, payer.address);
  t.is(beforeAsset.data.name, 'Test Bread');
  t.is(beforeAsset.data.uri, 'https://example.com/bread');
  t.is(beforeAsset.data.updateAuthority.__kind, 'Address');
  if (beforeAsset.data.updateAuthority.__kind === 'Address') {
    t.is(beforeAsset.data.updateAuthority.fields[0], payer.address);
  }

  // Compress the asset
  const compressInstruction = getCompressV1Instruction({
    asset: assetAddress.address,
    payer,
    logWrapper: address('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV'),
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [compressInstruction],
    [payer]
  );

  // Verify asset is now compressed as a hashed asset
  const afterCompressedAsset = await fetchHashedAssetV1(
    rpc,
    assetAddress.address
  );

  // Verify the hash matches the expected value
  const assetHash = hash(new Uint8Array(getAssetV1Encoder().encode(beforeAsset.data)));
  const hashedAssetSchema: HashedAssetSchema = {
    assetHash,
    pluginHashes: [],
  };

  const expectedHash = hash(
    new Uint8Array(getHashedAssetSchemaEncoder().encode(hashedAssetSchema))
  );
  t.deepEqual(afterCompressedAsset.data.hash, expectedHash);

  // Decompress the asset
  const decompressInstruction = getDecompressV1Instruction({
    asset: assetAddress.address,
    payer,
    compressionProof: {
      updateAuthority: updateAuthorityToBase({
        type: 'Address',
        address: payer.address,
      }),
      owner: payer.address,
      name: 'Test Bread',
      uri: 'https://example.com/bread',
      seq: 1n,
      plugins: [],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [decompressInstruction],
    [payer]
  );

  // Verify asset is now decompressed
  const afterDecompressedAsset = await fetchAssetV1(
    rpc,
    assetAddress.address
  );

  t.is(afterDecompressedAsset.data.key, Key.AssetV1);
  t.is(afterDecompressedAsset.data.owner, payer.address);
  t.is(afterDecompressedAsset.data.name, 'Test Bread');
  t.is(afterDecompressedAsset.data.uri, 'https://example.com/bread');
  t.is(afterDecompressedAsset.data.updateAuthority.__kind, 'Address');
  if (afterDecompressedAsset.data.updateAuthority.__kind === 'Address') {
    t.is(afterDecompressedAsset.data.updateAuthority.fields[0], payer.address);
  }
});

test('it cannot use an invalid system program', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const fakeSystemProgram = await generateKeyPairSigner();

  // Create asset
  const createInstruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [createInstruction],
    [assetAddress, payer]
  );

  // Attempt to decompress with invalid system program
  const decompressInstruction = getDecompressV1Instruction({
    asset: assetAddress.address,
    payer,
    systemProgram: fakeSystemProgram.address,
    compressionProof: {
      owner: payer.address,
      updateAuthority: {
        __kind: 'None',
      },
      name: '',
      uri: '',
      seq: 0n,
      plugins: [],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [decompressInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});

test('it cannot use an invalid noop program', async (t) => {
  const rpc = createRpc();
  const rpcSubscriptions = createRpcSubscriptions();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const fakeLogWrapper = await generateKeyPairSigner();

  // Create asset
  const createInstruction = getCreateV1Instruction({
    dataState: DataState.AccountState,
    asset: assetAddress,
    payer,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [createInstruction],
    [assetAddress, payer]
  );

  // Attempt to decompress with invalid log wrapper
  const decompressInstruction = getDecompressV1Instruction({
    asset: assetAddress.address,
    payer,
    logWrapper: fakeLogWrapper.address,
    compressionProof: {
      owner: payer.address,
      updateAuthority: {
        __kind: 'None',
      },
      name: '',
      uri: '',
      seq: 0n,
      plugins: [],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [decompressInstruction],
    [payer]
  );

  await t.throwsAsync(result);
});
