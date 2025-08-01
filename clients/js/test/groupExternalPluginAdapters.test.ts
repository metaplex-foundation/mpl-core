import { publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  createExternalPluginAdapterInitInfo,
  externalPluginAdapterKeyToBase,
  ExternalPluginAdapterSchema,
} from '../src';
import {
  addGroupExternalPluginAdapterV1,
  removeGroupExternalPluginAdapterV1,
} from '../src/generated';
import {
  assertGroup,
  createGroup,
  createUmi,
  DEFAULT_GROUP,
} from './_setupRaw';

// Re-use the same SPL Noop log wrapper program used across existing plugin tests.
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Group External Plugin Adapters – AppData
// -----------------------------------------------------------------------------

test('it can add, write, and remove an AppData external plugin adapter on a group', async (t) => {
  // ---------------------------------------------------------------------------
  // 1. Setup – Create a group.
  // ---------------------------------------------------------------------------
  const umi = await createUmi();
  const group = await createGroup(umi);

  // ---------------------------------------------------------------------------
  // 2. Add an AppData external plugin adapter (schema: JSON).
  // ---------------------------------------------------------------------------
  const initInfo = createExternalPluginAdapterInitInfo({
    type: 'AppData',
    dataAuthority: { type: 'UpdateAuthority' },
    schema: ExternalPluginAdapterSchema.Json,
  });

  await addGroupExternalPluginAdapterV1(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    initInfo,
  }).sendAndConfirm(umi);

  // ---------------------------------------------------------------------------
  // 4. Remove the adapter and ensure it no longer exists.
  // ---------------------------------------------------------------------------
  await removeGroupExternalPluginAdapterV1(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    key: externalPluginAdapterKeyToBase({
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    }),
  }).sendAndConfirm(umi);

  await assertGroup(t, umi, {
    ...DEFAULT_GROUP,
    group: group.publicKey,
    updateAuthority: umi.identity.publicKey,
  });
});
