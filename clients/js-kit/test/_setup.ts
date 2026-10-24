/**
 * Test setup utilities for mpl-core js-kit client
 */

import type { Address } from '@solana/addresses';
import type { Account } from '@solana/accounts';
import { createSolanaRpc, type Rpc, type SolanaRpcApi } from '@solana/rpc';
import {
  createSolanaRpcSubscriptions,
  type RpcSubscriptions,
  type SolanaRpcSubscriptionsApi,
} from '@solana/rpc-subscriptions';
import { airdropFactory, lamports, fetchEncodedAccount } from '@solana/kit';
import {
  generateKeyPairSigner,
  type KeyPairSigner,
} from '@solana/signers';
import type { Assertions } from 'ava';

import {
  getCreateV1Instruction,
  getCreateCollectionV1Instruction,
  fetchAssetV1,
  fetchCollectionV1,
  type AssetV1,
  type CollectionV1,
  type PluginAuthorityPairArgs,
  type DataState,
  Key,
} from '../src';
import { sendAndConfirmInstructions } from './_transaction';

// Re-export transaction utilities
export { sendAndConfirm, sendAndConfirmInstructions } from './_transaction';

const LOCAL_VALIDATOR_URL = 'http://127.0.0.1:8899';
const LOCAL_VALIDATOR_WS_URL = 'ws://127.0.0.1:8900';

export function createRpc(): Rpc<SolanaRpcApi> {
  return createSolanaRpc(LOCAL_VALIDATOR_URL);
}

export function createRpcSubscriptions(): RpcSubscriptions<SolanaRpcSubscriptionsApi> {
  return createSolanaRpcSubscriptions(LOCAL_VALIDATOR_WS_URL);
}

export async function canRunTests(): Promise<boolean> {
  try {
    const rpc = createRpc();
    await rpc.getVersion().send();
    return true;
  } catch {
    return false;
  }
}

export function getSkipMessage(): string {
  return `
Local Solana validator is not running.

To run these tests:
1. Start the local validator from the repository root:
   pnpm validator

2. Run the tests:
   pnpm test

The validator should be running at ${LOCAL_VALIDATOR_URL}
`.trim();
}

export async function airdrop(
  rpc: Rpc<SolanaRpcApi>,
  recipient: Address,
  amount: bigint = 10_000_000_000n
): Promise<void> {
  const rpcSubscriptions = createRpcSubscriptions();
  const airdropFn = airdropFactory({ rpc, rpcSubscriptions });

  await airdropFn({
    recipientAddress: recipient,
    lamports: lamports(amount),
    commitment: 'confirmed',
  });
}

export async function generateSignerWithSol(
  rpc: Rpc<SolanaRpcApi>,
  amount: bigint = 1_000_000_000n
): Promise<KeyPairSigner> {
  const signer = await generateKeyPairSigner();
  await airdrop(rpc, signer.address, amount);
  return signer;
}

export const DEFAULT_ASSET = {
  name: 'Test Asset',
  uri: 'https://example.com/asset',
};

export const DEFAULT_COLLECTION = {
  name: 'Test Collection',
  uri: 'https://example.com/collection',
};

export type CreateAssetHelperArgs = {
  owner?: Address | KeyPairSigner;
  payer?: KeyPairSigner;
  asset?: KeyPairSigner;
  dataState?: DataState;
  name?: string;
  uri?: string;
  authority?: KeyPairSigner;
  updateAuthority?: Address | KeyPairSigner;
  collection?: Address;
  plugins?: PluginAuthorityPairArgs[];
};

export const createAsset = async (
  rpc: Rpc<SolanaRpcApi>,
  payer: KeyPairSigner,
  input: CreateAssetHelperArgs = {}
): Promise<Account<AssetV1>> => {
  const rpcSubscriptions = createRpcSubscriptions();
  const asset = input.asset || (await generateKeyPairSigner());
  const owner =
    typeof input.owner === 'string'
      ? input.owner
      : input.owner?.address || input.payer?.address || payer.address;
  const updateAuthority =
    typeof input.updateAuthority === 'string'
      ? input.updateAuthority
      : input.updateAuthority?.address;

  const instruction = getCreateV1Instruction({
    asset,
    payer: input.payer || payer,
    owner,
    updateAuthority,
    collection: input.collection,
    authority: input.authority,
    name: input.name || DEFAULT_ASSET.name,
    uri: input.uri || DEFAULT_ASSET.uri,
    plugins: input.plugins,
    dataState: input.dataState,
  });

  const signers = [asset, input.payer || payer];
  if (input.authority && input.authority.address !== payer.address) {
    signers.push(input.authority);
  }

  await sendAndConfirmInstructions(rpc, rpcSubscriptions, [instruction], signers);

  return fetchAssetV1(rpc, asset.address);
};

export type CreateCollectionHelperArgs = {
  payer?: KeyPairSigner;
  collection?: KeyPairSigner;
  name?: string;
  uri?: string;
  updateAuthority?: Address | KeyPairSigner;
  plugins?: PluginAuthorityPairArgs[];
};

export const createCollection = async (
  rpc: Rpc<SolanaRpcApi>,
  payer: KeyPairSigner,
  input: CreateCollectionHelperArgs = {}
): Promise<Account<CollectionV1>> => {
  const rpcSubscriptions = createRpcSubscriptions();
  const collection = input.collection || (await generateKeyPairSigner());
  const updateAuthority =
    typeof input.updateAuthority === 'string'
      ? input.updateAuthority
      : input.updateAuthority?.address || (input.payer || payer).address;

  const instruction = getCreateCollectionV1Instruction({
    collection,
    payer: input.payer || payer,
    updateAuthority,
    name: input.name || DEFAULT_COLLECTION.name,
    uri: input.uri || DEFAULT_COLLECTION.uri,
    plugins: input.plugins,
  });

  await sendAndConfirmInstructions(
    rpc,
    rpcSubscriptions,
    [instruction],
    [collection, input.payer || payer]
  );

  return fetchCollectionV1(rpc, collection.address);
};

export const createAssetWithCollection = async (
  rpc: Rpc<SolanaRpcApi>,
  payer: KeyPairSigner,
  assetInput: CreateAssetHelperArgs & { collection?: Address | KeyPairSigner },
  collectionInput: CreateCollectionHelperArgs = {}
): Promise<{
  asset: Account<AssetV1>;
  collection: Account<CollectionV1>;
}> => {
  let collectionAddress: Address | undefined;
  if (assetInput.collection) {
    collectionAddress =
      typeof assetInput.collection === 'string'
        ? assetInput.collection
        : (assetInput.collection as KeyPairSigner).address;
  }

  const collection = collectionAddress
    ? await fetchCollectionV1(rpc, collectionAddress)
    : await createCollection(rpc, payer, {
        payer: assetInput.payer,
        updateAuthority: assetInput.updateAuthority,
        ...collectionInput,
      });

  // Destructure to remove updateAuthority from asset creation
  // since collection becomes the update authority
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { updateAuthority, collection: _, ...assetInputWithoutUpdateAuth } = assetInput;

  // If updateAuthority is a signer and no explicit authority is provided,
  // use it as the authority for creating assets in the collection
  const authority =
    assetInputWithoutUpdateAuth.authority ||
    (updateAuthority && typeof updateAuthority !== 'string' ? updateAuthority : undefined);

  const asset = await createAsset(rpc, payer, {
    ...assetInputWithoutUpdateAuth,
    authority,
    collection: collection.address,
  });

  return {
    asset,
    collection,
  };
};

export type AssertAssetInput = {
  asset: Address | KeyPairSigner;
  owner: Address | KeyPairSigner;
  updateAuthority?: { type: string; address?: Address };
  name?: string | RegExp;
  uri?: string | RegExp;
  [key: string]: unknown;
};

export const assertAsset = async (
  t: Assertions,
  rpc: Rpc<SolanaRpcApi>,
  input: AssertAssetInput
): Promise<void> => {
  const { asset, owner, name, uri, updateAuthority, ...rest } = input;
  const assetAddress = typeof asset === 'string' ? asset : asset.address;
  const ownerAddress = typeof owner === 'string' ? owner : owner.address;

  const assetAccount = await fetchAssetV1(rpc, assetAddress);

  // Name
  if (typeof name === 'string') {
    t.is(assetAccount.data.name, name);
  } else if (name !== undefined) {
    t.regex(assetAccount.data.name, name);
  }

  // Uri
  if (typeof uri === 'string') {
    t.is(assetAccount.data.uri, uri);
  } else if (uri !== undefined) {
    t.regex(assetAccount.data.uri, uri);
  }

  // Key
  t.is(assetAccount.data.key, Key.AssetV1);

  // Owner
  t.is(assetAccount.data.owner, ownerAddress);

  // Update authority
  if (updateAuthority) {
    t.is(assetAccount.data.updateAuthority.__kind, updateAuthority.type as typeof assetAccount.data.updateAuthority.__kind);
    if (updateAuthority.address && 'value' in assetAccount.data.updateAuthority) {
      t.is(
        (assetAccount.data.updateAuthority as { __kind: string; value: Address }).value,
        updateAuthority.address
      );
    }
  }

  // Check remaining fields
  for (const [key, value] of Object.entries(rest)) {
    if (value !== undefined && key in assetAccount.data) {
      t.deepEqual((assetAccount.data as Record<string, unknown>)[key], value);
    }
  }
};

export type AssertCollectionInput = {
  collection: Address | KeyPairSigner;
  updateAuthority?: Address | KeyPairSigner;
  name?: string | RegExp;
  uri?: string | RegExp;
  numMinted?: number;
  currentSize?: number;
  [key: string]: unknown;
};

export const assertCollection = async (
  t: Assertions,
  rpc: Rpc<SolanaRpcApi>,
  input: AssertCollectionInput
): Promise<void> => {
  const { collection, name, uri, updateAuthority, ...rest } = input;
  const collectionAddress =
    typeof collection === 'string' ? collection : collection.address;

  const collectionAccount = await fetchCollectionV1(rpc, collectionAddress);

  // Name
  if (typeof name === 'string') {
    t.is(collectionAccount.data.name, name);
  } else if (name !== undefined) {
    t.regex(collectionAccount.data.name, name);
  }

  // Uri
  if (typeof uri === 'string') {
    t.is(collectionAccount.data.uri, uri);
  } else if (uri !== undefined) {
    t.regex(collectionAccount.data.uri, uri);
  }

  // Key
  t.is(collectionAccount.data.key, Key.CollectionV1);

  // Update authority
  if (updateAuthority) {
    const expectedAuthority =
      typeof updateAuthority === 'string'
        ? updateAuthority
        : updateAuthority.address;
    t.is(collectionAccount.data.updateAuthority, expectedAuthority);
  }

  // Check remaining fields
  for (const [key, value] of Object.entries(rest)) {
    if (value !== undefined && key in collectionAccount.data) {
      t.deepEqual(
        (collectionAccount.data as Record<string, unknown>)[key],
        value
      );
    }
  }
};

export const assertBurned = async (
  t: Assertions,
  rpc: Rpc<SolanaRpcApi>,
  asset: Address
): Promise<void> => {
  const account = await fetchEncodedAccount(rpc, asset);
  t.true(account.exists);
  if (account.exists) {
    t.is(account.data.length, 1);
    t.is(account.data[0], Key.Uninitialized);
  }
};
