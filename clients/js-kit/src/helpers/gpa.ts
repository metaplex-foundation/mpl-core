/**
 * GPA (getProgramAccounts) helpers for fetching assets and collections
 */

import type { Address } from '@solana/addresses';
import type { Rpc, SolanaRpcApi } from '@solana/rpc';
import type { Account } from '@solana/accounts';
import type { Base58EncodedBytes } from '@solana/rpc-types';
import {
  type AssetV1,
  type CollectionV1,
  getAssetV1Decoder,
  getCollectionV1Decoder,
  Key,
} from '../generated';
import { MPL_CORE_PROGRAM_ADDRESS } from '../generated/programs';

/**
 * Account layout offsets for AssetV1:
 * - key: offset 0, 1 byte (enum)
 * - owner: offset 1, 32 bytes (Address)
 * - updateAuthority: offset 33, variable (discriminated union)
 *   - discriminator: 1 byte (0=None, 1=Address, 2=Collection)
 *   - address (if Address or Collection): 32 bytes
 */
const ASSET_KEY_OFFSET = 0;
const ASSET_OWNER_OFFSET = 1;
const ASSET_UPDATE_AUTHORITY_OFFSET = 33;

/**
 * Account layout offsets for CollectionV1:
 * - key: offset 0, 1 byte (enum)
 * - updateAuthority: offset 1, 32 bytes (Address)
 */
const COLLECTION_KEY_OFFSET = 0;
const COLLECTION_UPDATE_AUTHORITY_OFFSET = 1;

// Base58 alphabet
const BASE58_ALPHABET =
  '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

// Simple base58 encoder
function encodeBase58(bytes: Uint8Array): Base58EncodedBytes {
  if (bytes.length === 0) return '' as Base58EncodedBytes;

  // Count leading zeros
  let zeros = 0;
  while (zeros < bytes.length && bytes[zeros] === 0) {
    zeros++;
  }

  // Convert to base58
  const input = Array.from(bytes);
  const encoded: number[] = [];

  for (const byte of input) {
    let carry = byte;
    for (let j = 0; j < encoded.length; j++) {
      carry += encoded[j] << 8;
      encoded[j] = carry % 58;
      carry = Math.floor(carry / 58);
    }
    while (carry > 0) {
      encoded.push(carry % 58);
      carry = Math.floor(carry / 58);
    }
  }

  // Add leading '1's for zero bytes
  let result = '1'.repeat(zeros);
  for (let i = encoded.length - 1; i >= 0; i--) {
    result += BASE58_ALPHABET[encoded[i]];
  }

  return result as Base58EncodedBytes;
}

// Simple base58 decoder
function decodeBase58(str: string): Uint8Array {
  if (str.length === 0) return new Uint8Array(0);

  // Count leading '1's
  let zeros = 0;
  while (zeros < str.length && str[zeros] === '1') {
    zeros++;
  }

  // Decode from base58
  const decoded: number[] = [];
  for (const char of str) {
    const value = BASE58_ALPHABET.indexOf(char);
    if (value === -1) throw new Error(`Invalid base58 character: ${char}`);

    let carry = value;
    for (let j = 0; j < decoded.length; j++) {
      carry += decoded[j] * 58;
      decoded[j] = carry & 0xff;
      carry >>= 8;
    }
    while (carry > 0) {
      decoded.push(carry & 0xff);
      carry >>= 8;
    }
  }

  // Add leading zeros
  const result = new Uint8Array(zeros + decoded.length);
  for (let i = zeros + decoded.length - 1, j = 0; j < decoded.length; i--, j++) {
    result[i] = decoded[j];
  }

  return result;
}

/**
 * Fetch all assets owned by a specific address
 */
export async function fetchAssetsByOwner(
  rpc: Rpc<SolanaRpcApi>,
  owner: Address
): Promise<Account<AssetV1>[]> {
  const response = await rpc
    .getProgramAccounts(MPL_CORE_PROGRAM_ADDRESS, {
      encoding: 'base64',
      filters: [
        // Filter by key = AssetV1 (enum value 1)
        {
          memcmp: {
            offset: BigInt(ASSET_KEY_OFFSET),
            bytes: encodeBase58(new Uint8Array([Key.AssetV1])),
            encoding: 'base58',
          },
        },
        // Filter by owner
        {
          memcmp: {
            offset: BigInt(ASSET_OWNER_OFFSET),
            bytes: owner as unknown as Base58EncodedBytes,
            encoding: 'base58',
          },
        },
      ],
    })
    .send();

  const decoder = getAssetV1Decoder();
  return response.map((item: { pubkey: Address; account: { data: [string, string]; executable: boolean; lamports: bigint; owner: Address } }) => {
    const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
    return {
      address: item.pubkey,
      data: decoder.decode(data),
      executable: item.account.executable,
      lamports: item.account.lamports,
      programAddress: item.account.owner,
    } as Account<AssetV1>;
  });
}

/**
 * Fetch all assets in a specific collection
 */
export async function fetchAssetsByCollection(
  rpc: Rpc<SolanaRpcApi>,
  collection: Address
): Promise<Account<AssetV1>[]> {
  // Build the updateAuthority filter bytes:
  // discriminator (2 = Collection) + collection address
  const collectionBytes = decodeBase58(collection as string);
  const updateAuthorityBytes = new Uint8Array(33);
  updateAuthorityBytes[0] = 2; // Collection discriminator
  updateAuthorityBytes.set(collectionBytes, 1);

  const response = await rpc
    .getProgramAccounts(MPL_CORE_PROGRAM_ADDRESS, {
      encoding: 'base64',
      filters: [
        // Filter by key = AssetV1 (enum value 1)
        {
          memcmp: {
            offset: BigInt(ASSET_KEY_OFFSET),
            bytes: encodeBase58(new Uint8Array([Key.AssetV1])),
            encoding: 'base58',
          },
        },
        // Filter by updateAuthority = Collection(address)
        {
          memcmp: {
            offset: BigInt(ASSET_UPDATE_AUTHORITY_OFFSET),
            bytes: encodeBase58(updateAuthorityBytes),
            encoding: 'base58',
          },
        },
      ],
    })
    .send();

  const decoder = getAssetV1Decoder();
  return response.map((item: { pubkey: Address; account: { data: [string, string]; executable: boolean; lamports: bigint; owner: Address } }) => {
    const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
    return {
      address: item.pubkey,
      data: decoder.decode(data),
      executable: item.account.executable,
      lamports: item.account.lamports,
      programAddress: item.account.owner,
    } as Account<AssetV1>;
  });
}

/**
 * Fetch all assets with a specific update authority address
 */
export async function fetchAssetsByUpdateAuthority(
  rpc: Rpc<SolanaRpcApi>,
  updateAuthority: Address
): Promise<Account<AssetV1>[]> {
  // Build the updateAuthority filter bytes:
  // discriminator (1 = Address) + authority address
  const authorityBytes = decodeBase58(updateAuthority as string);
  const updateAuthorityBytes = new Uint8Array(33);
  updateAuthorityBytes[0] = 1; // Address discriminator
  updateAuthorityBytes.set(authorityBytes, 1);

  const response = await rpc
    .getProgramAccounts(MPL_CORE_PROGRAM_ADDRESS, {
      encoding: 'base64',
      filters: [
        // Filter by key = AssetV1 (enum value 1)
        {
          memcmp: {
            offset: BigInt(ASSET_KEY_OFFSET),
            bytes: encodeBase58(new Uint8Array([Key.AssetV1])),
            encoding: 'base58',
          },
        },
        // Filter by updateAuthority = Address(authority)
        {
          memcmp: {
            offset: BigInt(ASSET_UPDATE_AUTHORITY_OFFSET),
            bytes: encodeBase58(updateAuthorityBytes),
            encoding: 'base58',
          },
        },
      ],
    })
    .send();

  const decoder = getAssetV1Decoder();
  return response.map((item: { pubkey: Address; account: { data: [string, string]; executable: boolean; lamports: bigint; owner: Address } }) => {
    const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
    return {
      address: item.pubkey,
      data: decoder.decode(data),
      executable: item.account.executable,
      lamports: item.account.lamports,
      programAddress: item.account.owner,
    } as Account<AssetV1>;
  });
}

/**
 * Fetch all collections with a specific update authority
 */
export async function fetchCollectionsByUpdateAuthority(
  rpc: Rpc<SolanaRpcApi>,
  updateAuthority: Address
): Promise<Account<CollectionV1>[]> {
  const response = await rpc
    .getProgramAccounts(MPL_CORE_PROGRAM_ADDRESS, {
      encoding: 'base64',
      filters: [
        // Filter by key = CollectionV1 (enum value 5)
        {
          memcmp: {
            offset: BigInt(COLLECTION_KEY_OFFSET),
            bytes: encodeBase58(new Uint8Array([Key.CollectionV1])),
            encoding: 'base58',
          },
        },
        // Filter by updateAuthority
        {
          memcmp: {
            offset: BigInt(COLLECTION_UPDATE_AUTHORITY_OFFSET),
            bytes: updateAuthority as unknown as Base58EncodedBytes,
            encoding: 'base58',
          },
        },
      ],
    })
    .send();

  const decoder = getCollectionV1Decoder();
  return response.map((item: { pubkey: Address; account: { data: [string, string]; executable: boolean; lamports: bigint; owner: Address } }) => {
    const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
    return {
      address: item.pubkey,
      data: decoder.decode(data),
      executable: item.account.executable,
      lamports: item.account.lamports,
      programAddress: item.account.owner,
    } as Account<CollectionV1>;
  });
}
