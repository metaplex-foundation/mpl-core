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
const BASE10_256 = 256n;
const BASE58 = 58n;

function countLeadingCharacter(str: string, character: string): number {
  const firstDifferentCharacter = str
    .split('')
    .findIndex((char) => char !== character);
  return firstDifferentCharacter === -1 ? str.length : firstDifferentCharacter;
}

function countLeadingZeroBytes(bytes: Uint8Array): number {
  const firstNonZeroByte = bytes.findIndex((byte) => byte !== 0);
  return firstNonZeroByte === -1 ? bytes.length : firstNonZeroByte;
}

function encodeBase58Value(value: bigint): string {
  if (value === 0n) {
    return '';
  }

  const quotient = value / BASE58;
  const remainder = Number(value % BASE58);
  return `${encodeBase58Value(quotient)}${BASE58_ALPHABET[remainder]}`;
}

function decodeBase58Value(str: string): bigint {
  return str.split('').reduce((value, char) => {
    const nextDigit = BASE58_ALPHABET.indexOf(char);
    if (nextDigit === -1) {
      throw new Error(`Invalid base58 character: ${char}`);
    }
    return value * BASE58 + BigInt(nextDigit);
  }, 0n);
}

function bigintToBytes(value: bigint): number[] {
  if (value === 0n) {
    return [];
  }

  const quotient = value / BASE10_256;
  const remainder = Number(value % BASE10_256);
  return [...bigintToBytes(quotient), remainder];
}

// Simple base58 encoder
function encodeBase58(bytes: Uint8Array): Base58EncodedBytes {
  if (bytes.length === 0) return '' as Base58EncodedBytes;

  const leadingZeroCount = countLeadingZeroBytes(bytes);
  const encodedValue = bytes.reduce(
    (value, byte) => value * BASE10_256 + BigInt(byte),
    0n
  );
  return `${'1'.repeat(leadingZeroCount)}${encodeBase58Value(encodedValue)}` as Base58EncodedBytes;
}

// Simple base58 decoder
function decodeBase58(str: string): Uint8Array {
  if (str.length === 0) return new Uint8Array(0);

  const leadingZeroCount = countLeadingCharacter(str, '1');
  const decodedValue = decodeBase58Value(str.slice(leadingZeroCount));
  return Uint8Array.from([
    ...Array.from({ length: leadingZeroCount }, () => 0),
    ...bigintToBytes(decodedValue),
  ]);
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
  return response.map(
    (item: {
      pubkey: Address;
      account: {
        data: [string, string];
        executable: boolean;
        lamports: bigint;
        owner: Address;
      };
    }) => {
      const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
      return {
        address: item.pubkey,
        data: decoder.decode(data),
        executable: item.account.executable,
        lamports: item.account.lamports,
        programAddress: item.account.owner,
      } as Account<AssetV1>;
    }
  );
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
  return response.map(
    (item: {
      pubkey: Address;
      account: {
        data: [string, string];
        executable: boolean;
        lamports: bigint;
        owner: Address;
      };
    }) => {
      const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
      return {
        address: item.pubkey,
        data: decoder.decode(data),
        executable: item.account.executable,
        lamports: item.account.lamports,
        programAddress: item.account.owner,
      } as Account<AssetV1>;
    }
  );
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
  return response.map(
    (item: {
      pubkey: Address;
      account: {
        data: [string, string];
        executable: boolean;
        lamports: bigint;
        owner: Address;
      };
    }) => {
      const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
      return {
        address: item.pubkey,
        data: decoder.decode(data),
        executable: item.account.executable,
        lamports: item.account.lamports,
        programAddress: item.account.owner,
      } as Account<AssetV1>;
    }
  );
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
  return response.map(
    (item: {
      pubkey: Address;
      account: {
        data: [string, string];
        executable: boolean;
        lamports: bigint;
        owner: Address;
      };
    }) => {
      const data = new Uint8Array(Buffer.from(item.account.data[0], 'base64'));
      return {
        address: item.pubkey,
        data: decoder.decode(data),
        executable: item.account.executable,
        lamports: item.account.lamports,
        programAddress: item.account.owner,
      } as Account<CollectionV1>;
    }
  );
}
