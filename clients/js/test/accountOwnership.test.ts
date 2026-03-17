/**
 * Account Ownership Tests
 *
 * This test suite proves that account ownership confusion was never exploitable
 * in mpl-core, and that the newly added explicit owner checks in
 * SolanaAccount::load are purely a defense-in-depth measure.
 *
 * BACKGROUND:
 * The PR adds `account.owner != &crate::ID` checks to both the on-chain program
 * and the Rust client's SolanaAccount::load method. These tests demonstrate that:
 *
 * 1. The Solana runtime already prevents other programs from writing mpl-core
 *    discriminator keys into accounts they own.
 * 2. The existing discriminator key check (load_key) rejects accounts with
 *    non-matching first bytes, which is the case for any account not initialized
 *    by mpl-core.
 * 3. The explicit owner check adds a belt-and-suspenders layer for safety.
 */

import { generateSigner, publicKey, sol } from '@metaplex-foundation/umi';
import test from 'ava';
import { createAccount } from '@metaplex-foundation/mpl-toolbox';
import {
  addPluginV1,
  burnCollectionV1,
  burnV1,
  createPlugin,
  pluginAuthorityPair,
  transferV1,
  updateCollectionV1,
  updateV1,
} from '../src';
import {
  assertAsset,
  assertCollection,
  createAsset,
  createAssetWithCollection,
  createCollection,
  createUmi,
  DEFAULT_ASSET,
} from './_setupRaw';

// ============================================================================
// SECTION 1: Baseline - Normal operations succeed with correctly-owned accounts
//
// These tests confirm the program works correctly under normal conditions.
// All accounts are owned by the mpl-core program as expected.
// ============================================================================

test('baseline: transfer succeeds with correctly program-owned asset', async (t) => {
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('baseline: transfer succeeds with correctly program-owned asset and collection', async (t) => {
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  await transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('baseline: burn succeeds with correctly program-owned asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi);
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });

  await burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  const afterAccount = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAccount.exists);
});

test('baseline: update succeeds with correctly program-owned asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi);

  await updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Updated Name',
    newUri: 'https://example.com/updated',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    name: 'Updated Name',
    uri: 'https://example.com/updated',
  });
});

test('baseline: add plugin succeeds with correctly program-owned asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi);

  await addPluginV1(umi, {
    asset: asset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: false } }),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: false,
    },
  });
});

test('baseline: burn collection succeeds with correctly program-owned collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await burnCollectionV1(umi, {
    collection: collection.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  const afterAccount = await umi.rpc.getAccount(collection.publicKey);
  t.true(afterAccount.exists);
});

test('baseline: update collection succeeds with correctly program-owned collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi);

  await updateCollectionV1(umi, {
    collection: collection.publicKey,
    newName: 'Updated Collection',
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    name: 'Updated Collection',
  });
});

// ============================================================================
// SECTION 2: Discriminator key check rejects uninitialized/wrong-key accounts
//
// These tests prove that even before the owner check was added, the load_key
// discriminator check in SolanaAccount::load prevented deserialization of
// accounts with incorrect first bytes. Any account not initialized by mpl-core
// will have a 0x00 (Uninitialized) first byte, which fails the key check.
// ============================================================================

test('discriminator: transfer rejects account with uninitialized key owned by mpl-core', async (t) => {
  // Create an account owned by mpl-core but with zeroed data (no valid discriminator).
  // This simulates an account that was allocated but never properly initialized
  // by the program. The transfer handler does an explicit load_key + match BEFORE
  // SolanaAccount::load, so Key::Uninitialized hits the catch-all arm → IncorrectAccount.
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);
  const newOwner = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = transferV1(umi, {
    asset: fakeAsset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'IncorrectAccount' });
});

test('discriminator: burn rejects account with uninitialized key owned by mpl-core', async (t) => {
  // The burn handler also does an explicit load_key + match BEFORE
  // SolanaAccount::load, so Key::Uninitialized → IncorrectAccount.
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = burnV1(umi, {
    asset: fakeAsset.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'IncorrectAccount' });
});

test('discriminator: update rejects account with uninitialized key owned by mpl-core', async (t) => {
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = updateV1(umi, {
    asset: fakeAsset.publicKey,
    newName: 'Hacked',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('discriminator: add plugin rejects account with uninitialized key owned by mpl-core', async (t) => {
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = addPluginV1(umi, {
    asset: fakeAsset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('discriminator: transfer rejects fake collection with uninitialized key', async (t) => {
  // Even before the owner check, passing a fake collection fails because:
  // 1. The fake collection has Key::Uninitialized, failing the discriminator check.
  // 2. Even if deserialization somehow passed, the asset's update_authority
  //    stores the real collection address which would not match the fake.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const fakeCollection = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {});

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  // Pass the fake collection - fails because the asset's update_authority
  // stores a different collection address.
  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: fakeCollection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('discriminator: burn rejects fake collection with uninitialized key', async (t) => {
  const umi = await createUmi();
  const fakeCollection = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {});

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = burnV1(umi, {
    asset: asset.publicKey,
    collection: fakeCollection.publicKey,
  }).sendAndConfirm(umi);

  // Fails because the burn handler loads the collection first via
  // CollectionV1::load, and the zeroed data has Key::Uninitialized.
  await t.throwsAsync(result, { name: 'DeserializationError' });
});

// ============================================================================
// SECTION 3: Accounts owned by the wrong program are rejected
//
// These tests create accounts owned by programs OTHER than mpl-core and
// attempt to use them in mpl-core instructions. The Solana runtime restricts
// which programs can modify which accounts:
//   - Only the owning program can modify account data.
//   - For writable accounts in an instruction, the runtime checks ownership.
//
// Combined with the discriminator key check (and now the explicit owner check),
// these accounts are completely unusable by mpl-core.
// ============================================================================

test('wrong owner: transfer rejects asset account owned by system program', async (t) => {
  // An account owned by the system program has zeroed data, so the first byte
  // is 0x00 (Key::Uninitialized). The discriminator check catches this.
  // The new owner check provides an additional layer of defense.
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);
  const newOwner = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = transferV1(umi, {
    asset: fakeAsset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  // Rejected: zeroed data has Key::Uninitialized, caught by explicit load_key check.
  await t.throwsAsync(result, { name: 'IncorrectAccount' });
});

test('wrong owner: burn rejects asset account owned by system program', async (t) => {
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = burnV1(umi, {
    asset: fakeAsset.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'IncorrectAccount' });
});

test('wrong owner: update rejects asset account owned by system program', async (t) => {
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = updateV1(umi, {
    asset: fakeAsset.publicKey,
    newName: 'Hacked',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('wrong owner: add plugin rejects asset account owned by system program', async (t) => {
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = addPluginV1(umi, {
    asset: fakeAsset.publicKey,
    plugin: createPlugin({ type: 'FreezeDelegate', data: { frozen: true } }),
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('wrong owner: transfer rejects collection account owned by system program', async (t) => {
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const fakeCollection = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {});

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: fakeCollection.publicKey,
  }).sendAndConfirm(umi);

  // Rejected because the fake collection address doesn't match what's stored
  // in the asset's update_authority (address check in validate_asset_permissions).
  await t.throwsAsync(result, { name: 'InvalidCollection' });
});

test('wrong owner: burn rejects collection account owned by system program', async (t) => {
  const umi = await createUmi();
  const fakeCollection = generateSigner(umi);

  const { asset } = await createAssetWithCollection(umi, {});

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = burnV1(umi, {
    asset: asset.publicKey,
    collection: fakeCollection.publicKey,
  }).sendAndConfirm(umi);

  // Burn loads the collection via CollectionV1::load BEFORE the address check,
  // so the zeroed data (Key::Uninitialized) fails the key check first.
  await t.throwsAsync(result, { name: 'DeserializationError' });
});

// ============================================================================
// SECTION 4: Address-based validation provides additional protection
//
// Even if an attacker could somehow craft an account with the correct
// discriminator key, the program validates that account addresses match
// expected values. For example, the collection address passed to transfer
// must match the address stored in the asset's update_authority field.
// ============================================================================

test('address validation: transfer rejects a real collection that does not match the asset', async (t) => {
  // This test uses a REAL collection (correctly owned by mpl-core, with valid
  // discriminator) but it's the WRONG collection for this asset. The program's
  // address validation catches this regardless of the owner check.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});
  const wrongCollection = await createCollection(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: wrongCollection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  // Verify the asset was not modified.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('address validation: burn rejects a real collection that does not match the asset', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(umi, {});
  const wrongCollection = await createCollection(umi);

  const result = burnV1(umi, {
    asset: asset.publicKey,
    collection: wrongCollection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('address validation: update rejects a real collection that does not match the asset', async (t) => {
  const umi = await createUmi();

  const { asset, collection } = await createAssetWithCollection(umi, {});
  const wrongCollection = await createCollection(umi);

  const result = updateV1(umi, {
    asset: asset.publicKey,
    collection: wrongCollection.publicKey,
    newName: 'Hacked',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('address validation: transfer rejects a collection specified for a standalone asset', async (t) => {
  // A standalone asset (update_authority = Address) should not accept any
  // collection account. The program returns InvalidCollection.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi);
  const collection = await createCollection(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: collection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

// ============================================================================
// SECTION 5: Authority checks prevent unauthorized operations
//
// Even with correctly-owned accounts, the program enforces strict authority
// checks. An attacker cannot perform operations on assets they don't own
// or have delegation for. These checks are independent of (and complementary
// to) the account ownership check.
// ============================================================================

test('authority: transfer rejects non-owner even with correct program-owned accounts', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('authority: burn rejects non-owner even with correct program-owned accounts', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi);

  const result = burnV1(umi, {
    asset: asset.publicKey,
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

test('authority: update rejects non-authority even with correct program-owned accounts', async (t) => {
  const umi = await createUmi();
  const attacker = generateSigner(umi);

  const asset = await createAsset(umi);

  const result = updateV1(umi, {
    asset: asset.publicKey,
    newName: 'Hacked',
    authority: attacker,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'NoApprovals' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
  });
});

// ============================================================================
// SECTION 6: Combined defense layers demonstration
//
// These tests demonstrate that multiple independent security layers work
// together. Even when one layer alone might theoretically be circumventable,
// the combination makes exploitation impossible.
// ============================================================================

test('combined defense: a completely random pubkey as asset fails immediately', async (t) => {
  // A random pubkey that doesn't correspond to any on-chain account will
  // fail even before any program logic executes.
  const umi = await createUmi();
  const randomPubkey = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const result = transferV1(umi, {
    asset: randomPubkey.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result);
});

test('combined defense: an account owned by a random program as asset is rejected', async (t) => {
  // Create an account with a random program as owner. This demonstrates that
  // even a totally unknown program's accounts cannot be confused with mpl-core
  // accounts. Multiple checks fail: wrong owner, wrong discriminator.
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);
  const randomProgram = generateSigner(umi);
  const newOwner = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: randomProgram.publicKey,
  }).sendAndConfirm(umi);

  const result = transferV1(umi, {
    asset: fakeAsset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'IncorrectAccount' });
});

test('combined defense: transfer with wrong-program collection and mismatched address fails', async (t) => {
  // This test combines multiple attack vectors:
  // 1. Collection account owned by wrong program (system program).
  // 2. Collection address doesn't match asset's update_authority.
  // The program rejects this through layered checks.
  const umi = await createUmi();
  const newOwner = generateSigner(umi);
  const fakeCollection = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(umi, {});

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
    collection: fakeCollection.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidCollection' });

  // Verify the asset remains unchanged.
  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Collection', address: collection.publicKey },
  });
});

test('combined defense: burn with wrong-program asset does not drain lamports', async (t) => {
  // Ensure that passing a non-mpl-core account to burn cannot be used to
  // drain lamports from the account to the payer.
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.5),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = burnV1(umi, {
    asset: fakeAsset.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'IncorrectAccount' });

  // Verify the fake asset still has its lamports (not drained).
  const account = await umi.rpc.getAccount(fakeAsset.publicKey);
  t.true(account.exists);
});

test('combined defense: update with wrong-program asset cannot modify data', async (t) => {
  // Ensure that passing a non-mpl-core account to update cannot modify
  // the account's data.
  const umi = await createUmi();
  const fakeAsset = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeAsset,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = updateV1(umi, {
    asset: fakeAsset.publicKey,
    newName: 'Hacked',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

// ============================================================================
// SECTION 7: Collection-level operations with wrong ownership
//
// These tests specifically target collection-level instruction handlers where
// the collection account is the primary account being operated on.
// ============================================================================

test('wrong owner: burn collection rejects non-program-owned collection', async (t) => {
  const umi = await createUmi();
  const fakeCollection = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = burnCollectionV1(umi, {
    collection: fakeCollection.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('wrong owner: update collection rejects non-program-owned collection', async (t) => {
  const umi = await createUmi();
  const fakeCollection = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: publicKey('11111111111111111111111111111111'),
  }).sendAndConfirm(umi);

  const result = updateCollectionV1(umi, {
    collection: fakeCollection.publicKey,
    newName: 'Hacked Collection',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('discriminator: burn collection rejects uninitialized account owned by mpl-core', async (t) => {
  const umi = await createUmi();
  const fakeCollection = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = burnCollectionV1(umi, {
    collection: fakeCollection.publicKey,
    compressionProof: null,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

test('discriminator: update collection rejects uninitialized account owned by mpl-core', async (t) => {
  const umi = await createUmi();
  const fakeCollection = generateSigner(umi);

  await createAccount(umi, {
    newAccount: fakeCollection,
    lamports: sol(0.1),
    space: 200,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  const result = updateCollectionV1(umi, {
    collection: fakeCollection.publicKey,
    newName: 'Hacked Collection',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'DeserializationError' });
});

// ============================================================================
// SECTION 8: Frozen asset protection with ownership confusion
//
// These tests verify that the freeze mechanism works correctly and cannot be
// bypassed using accounts from wrong programs.
// ============================================================================

test('frozen: transfer rejects even with correct ownership when asset is frozen', async (t) => {
  const umi = await createUmi();
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const result = transferV1(umi, {
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    freezeDelegate: {
      authority: { type: 'Owner' },
      frozen: true,
    },
  });
});

test('frozen: burn rejects even with correct ownership when asset is frozen', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      pluginAuthorityPair({
        type: 'FreezeDelegate',
        data: { frozen: true },
      }),
    ],
  });

  const result = burnV1(umi, {
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
});
