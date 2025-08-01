import { generateSigner, publicKey } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  createExternalPluginAdapterInitInfo,
  ExternalPluginAdapterSchema,
  writeGroupData,
} from '../src';
import { addGroupExternalPluginAdapterV1 } from '../src/generated';
import { createGroup, createUmi } from './_setupRaw';

// Re-use the SPL Noop log wrapper program used elsewhere in the suite.
const LOG_WRAPPER = publicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');

// -----------------------------------------------------------------------------
// Error Handling – writeGroupData
// -----------------------------------------------------------------------------

test('writeGroupData fails for unsupported adapter types', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  await t.throwsAsync(
    writeGroupData(umi, {
      group: group.publicKey,
      payer: umi.identity,
      authority: umi.identity,
      key: { type: 'Oracle', baseAddress: umi.identity.publicKey },
      data: new Uint8Array([1, 2, 3]),
    }).sendAndConfirm(umi)
  );
});

test('writeGroupData fails when no data sources are provided', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  // Add a valid AppData adapter first so that the key exists on-chain.
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

  // Attempt to write with neither `data` nor `buffer` provided.
  await t.throwsAsync(
    writeGroupData(umi, {
      group: group.publicKey,
      payer: umi.identity,
      authority: umi.identity,
      key: { type: 'AppData', dataAuthority: { type: 'UpdateAuthority' } },
      data: null,
    }).sendAndConfirm(umi)
  );
});

test('writeGroupData fails when signer is not the plugin data authority', async (t) => {
  const umi = await createUmi();
  const group = await createGroup(umi);

  const externalSigner = generateSigner(umi);

  // Add adapter with an explicit external signer set as data authority.
  const initInfo = createExternalPluginAdapterInitInfo({
    type: 'AppData',
    dataAuthority: { type: 'Address', address: externalSigner.publicKey },
    schema: ExternalPluginAdapterSchema.Json,
  });

  await addGroupExternalPluginAdapterV1(umi, {
    group: group.publicKey,
    payer: umi.identity,
    authority: umi.identity,
    logWrapper: LOG_WRAPPER,
    initInfo,
  }).sendAndConfirm(umi);

  const bytes = new Uint8Array([1, 2, 3]);

  // Attempt to write using the update authority (umi.identity) instead of the
  // designated data authority.
  await t.throwsAsync(
    writeGroupData(umi, {
      group: group.publicKey,
      payer: umi.identity,
      authority: umi.identity, // Not authorised
      key: {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: externalSigner.publicKey },
      },
      data: bytes,
    }).sendAndConfirm(umi)
  );
});
