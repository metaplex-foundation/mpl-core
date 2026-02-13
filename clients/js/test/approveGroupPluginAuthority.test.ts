import {
  assertAccountExists,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi';
import test from 'ava';
import {
  addGroupPlugin,
  approveGroupPluginAuthority,
} from '../src';
import {
  assertGroup,
  createGroup,
  createUmi,
  DEFAULT_GROUP,
} from './_setupRaw';

// Same log-wrapper constant used in existing tests.
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Approve & Revoke Plugin Authority
// -----------------------------------------------------------------------------

test('it can approve plugin authority on a group', async (t: any) => {
  // ---------------------------------------------------------------------------
  // Setup: create a group with an Attributes plugin whose authority is None.
  // ---------------------------------------------------------------------------
  const umi = await createUmi();
  const group = await createGroup(umi);

  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'init', value: 'value' }],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // ---------------------------------------------------------------------------
  // Approve: delegate plugin authority to a new signer.
  // ---------------------------------------------------------------------------
  const newAuthority = generateSigner(umi);

  await approveGroupPluginAuthority(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    plugin: { type: 'Attributes' },
    newAuthority: { type: 'Address', address: newAuthority.publicKey },
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});

test('re-approving group plugin authority with the same authority shape does not resize the account', async (t: any) => {
  // ---------------------------------------------------------------------------
  // Setup: create a group and install an Attributes plugin.
  // ---------------------------------------------------------------------------
  const umi = await createUmi();
  const group = await createGroup(umi);

  await addGroupPlugin(umi, {
    group: group.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'init', value: 'value' }],
    },
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  // ---------------------------------------------------------------------------
  // First approval: transition from default manager authority to Address authority.
  // ---------------------------------------------------------------------------
  const firstAuthority = generateSigner(umi);
  await approveGroupPluginAuthority(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    plugin: { type: 'Attributes' },
    newAuthority: { type: 'Address', address: firstAuthority.publicKey },
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  const firstApprovalAccount = await umi.rpc.getAccount(group.publicKey);
  assertAccountExists(firstApprovalAccount, 'Group');
  const accountSizeAfterFirstApproval = firstApprovalAccount.data.length;

  // ---------------------------------------------------------------------------
  // Second approval: another Address authority has the same serialized size.
  // Account size must remain unchanged.
  // ---------------------------------------------------------------------------
  const secondAuthority = generateSigner(umi);
  await approveGroupPluginAuthority(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    plugin: { type: 'Attributes' },
    newAuthority: { type: 'Address', address: secondAuthority.publicKey },
    logWrapper: LOG_WRAPPER,
  }).sendAndConfirm(umi);

  const secondApprovalAccount = await umi.rpc.getAccount(group.publicKey);
  assertAccountExists(secondApprovalAccount, 'Group');
  t.is(secondApprovalAccount.data.length, accountSizeAfterFirstApproval);
});
