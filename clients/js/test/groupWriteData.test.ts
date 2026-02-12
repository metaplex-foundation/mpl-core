import { createAccount } from '@metaplex-foundation/mpl-toolbox';
import {
  assertAccountExists,
  generateSigner,
  publicKey,
  sol,
} from '@metaplex-foundation/umi';
import test from 'ava';
import {
  createExternalPluginAdapterInitInfo,
  ExternalPluginAdapterSchema,
  writeGroupData,
} from '../src';
import { addGroupExternalPluginAdapterV1 } from '../src/generated';
import { getGroupV1AccountDataSerializer } from '../src/hooked';
import { createGroup, createUmi } from './_setupRaw';

// Re-use the SPL Noop log wrapper program used elsewhere in the suite.
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Write Data – AppData External Plugin Adapter
// -----------------------------------------------------------------------------

test('it can write JSON data to an AppData external plugin adapter on a group', async (t) => {
  // ---------------------------------------------------------------------------
  // 1. Setup – create a group and add an AppData external plugin adapter.
  // ---------------------------------------------------------------------------
  const umi = await createUmi();
  const group = await createGroup(umi);

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
  // 2. Write some JSON data to the adapter via a buffer account.
  // ---------------------------------------------------------------------------
  const json = JSON.stringify({ hello: 'world' });
  const dataBytes = new TextEncoder().encode(json);

  // Create a temporary buffer account containing the data.
  const buffer = generateSigner(umi);
  await createAccount(umi, {
    newAccount: buffer,
    lamports: sol(0.1),
    space: dataBytes.length,
    programId: umi.programs.get('mplCore').publicKey,
  }).sendAndConfirm(umi);

  // Write data using the buffer account (no inline bytes).
  await writeGroupData(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    key: { type: 'AppData', dataAuthority: { type: 'UpdateAuthority' } },
    buffer: buffer.publicKey,
    data: null,
  }).sendAndConfirm(umi);

  // ---------------------------------------------------------------------------
  // 3. Verify the data has been written (dataLen should match length).
  // ---------------------------------------------------------------------------
  const account = await umi.rpc.getAccount(group.publicKey);
  assertAccountExists(account, 'Group');
  const [decoded] = getGroupV1AccountDataSerializer().deserialize(account.data);

  t.true(Array.isArray(decoded.appDatas) && decoded.appDatas.length === 1);
  if (decoded.appDatas) {
    t.is(decoded.appDatas[0].dataLen, BigInt(dataBytes.length));
  }
});
