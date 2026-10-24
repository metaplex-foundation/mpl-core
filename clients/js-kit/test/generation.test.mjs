import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { getAddressEncoder, getProgramDerivedAddress, some } from '@solana/kit';
import * as core from '../dist/index.js';

const idl = JSON.parse(
  readFileSync(new URL('../../../idls/mpl_core.json', import.meta.url))
);
const camelCase = (name) =>
  name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
const owner = '11111111111111111111111111111111';
const concatenate = (parts) =>
  Buffer.concat(parts.map((part) => Buffer.from(part)));
const integer = (value, size) => {
  const bytes = Buffer.alloc(size);
  for (let index = 0; index < size; index += 1)
    bytes[index] = Number((BigInt(value) >> BigInt(index * 8)) & 255n);
  return bytes;
};

// An IDL-driven Borsh fixture encoder independently checks the generated codecs.
// Nonempty vectors/options exercise nested structures rather than only defaults.
function fixture(type) {
  if (typeof type === 'string') {
    if (type === 'bool') return { bytes: Buffer.from([1]), value: true };
    if (type === 'publicKey') return { bytes: Buffer.alloc(32), value: owner };
    if (type === 'string')
      return {
        bytes: concatenate([integer(3, 4), Buffer.from('Kit')]),
        value: 'Kit',
      };
    if (type === 'bytes')
      return {
        bytes: Buffer.from([2, 0, 0, 0, 7, 8]),
        value: new Uint8Array([7, 8]),
      };
    const match = /^(?:i|u)(8|16|32|64|128)$/.exec(type);
    if (match)
      return {
        bytes: integer(1, Number(match[1]) / 8),
        value: Number(match[1]) >= 64 ? 1n : 1,
      };
    throw new Error(`Unsupported IDL primitive in test fixture: ${type}`);
  }
  if ('defined' in type) {
    const definition = idl.types.find(
      ({ name }) =>
        name === (type.defined === 'crate' ? 'RelationshipEntry' : type.defined)
    );
    assert.ok(definition, `Missing IDL definition ${type.defined}`);
    return fixture(definition.type);
  }
  if ('tuple' in type) {
    const fields = type.tuple.map(fixture);
    return {
      bytes: concatenate(fields.map((field) => field.bytes)),
      value: fields.map((field) => field.value),
    };
  }
  if ('option' in type) {
    const inner = fixture(type.option);
    return { bytes: concatenate([[1], inner.bytes]), value: some(inner.value) };
  }
  if ('vec' in type || 'array' in type) {
    const inner = fixture(type.vec ?? type.array[0]);
    const count = type.vec ? 1 : type.array[1];
    return {
      bytes: concatenate([
        ...(type.vec ? [integer(count, 4)] : []),
        ...Array(count).fill(inner.bytes),
      ]),
      value: Array(count).fill(inner.value),
    };
  }
  if (type.kind === 'struct') {
    const fields = (type.fields ?? []).map((field) => [
      camelCase(field.name),
      fixture(field.type),
    ]);
    return {
      bytes: concatenate(fields.map(([, value]) => value.bytes)),
      value: Object.fromEntries(
        fields.map(([name, value]) => [name, value.value])
      ),
    };
  }
  if (type.kind === 'enum') {
    if (type.variants.every((variant) => !variant.fields?.length))
      return { bytes: Buffer.from([0]), value: 0 };
    const variant = type.variants[0];
    const named = variant.fields?.[0]?.name !== undefined;
    const fields = (variant.fields ?? []).map((field) =>
      fixture(named ? field.type : field)
    );
    const value = named
      ? Object.fromEntries(
          variant.fields.map((field, index) => [
            camelCase(field.name),
            fields[index].value,
          ])
        )
      : fields.length
        ? { fields: fields.map((field) => field.value) }
        : {};
    return {
      bytes: concatenate([[0], ...fields.map((field) => field.bytes)]),
      value: { __kind: variant.name, ...value },
    };
  }
  throw new Error(
    `Unsupported IDL type in test fixture: ${JSON.stringify(type)}`
  );
}

for (const instruction of [...idl.instructions].sort((a, b) =>
  a.name.localeCompare(b.name)
)) {
  test(`${instruction.name}: matches current IDL bytes and discriminator`, () => {
    const args = instruction.args.map(({ type }) => fixture(type));
    const input = Object.assign({}, ...args.map(({ value }) => value));
    const expected = concatenate([
      integer(instruction.discriminant.value, 1),
      ...args.map(({ bytes }) => bytes),
    ]);
    const encoder = core[`get${instruction.name}InstructionDataEncoder`];
    const decoder = core[`get${instruction.name}InstructionDataDecoder`];
    assert.equal(typeof encoder, 'function');
    assert.equal(typeof decoder, 'function');
    const actual = encoder().encode(input);
    assert.deepEqual(Buffer.from(actual), expected);
    assert.equal(
      core.identifyMplCoreInstruction(actual),
      core.MplCoreInstruction[instruction.name]
    );
    assert.deepEqual(
      Buffer.from(encoder().encode(decoder().decode(actual))),
      expected
    );
  });
}

for (const [name, args] of [
  ['HashedAssetV1', { hash: new Uint8Array(32), key: core.Key.HashedAssetV1 }],
  [
    'PluginHeaderV1',
    { key: core.Key.PluginHeaderV1, pluginRegistryOffset: 9n },
  ],
  [
    'PluginRegistryV1',
    { externalRegistry: [], key: core.Key.PluginRegistryV1, registry: [] },
  ],
]) {
  test(`${name}: identifies valid encoded account data`, () => {
    const encoded = core[`get${name}Encoder`]().encode(args);
    assert.equal(
      core.identifyMplCoreAccount(encoded),
      core.MplCoreAccount[name]
    );
  });
}

test('instruction defaults use the established Core Borsh representation', () => {
  assert.deepEqual(
    [...core.getTransferV1InstructionDataEncoder().encode({})],
    [14, 0]
  );
  assert.deepEqual(
    [...core.getBurnV1InstructionDataEncoder().encode({})],
    [12, 0]
  );
  assert.deepEqual(
    [...core.getUpdateV2InstructionDataEncoder().encode({})],
    [30, 0, 0, 0]
  );
  assert.deepEqual(
    [
      ...core
        .getCreateV2InstructionDataEncoder()
        .encode({ name: 'A', uri: '' }),
    ],
    [20, 0, 1, 0, 0, 0, 65, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]
  );
});

test('asset signer derivation uses the execute seed and asset address', async () => {
  const expected = await getProgramDerivedAddress({
    programAddress: core.MPL_CORE_PROGRAM_ADDRESS,
    seeds: [
      new TextEncoder().encode('mpl-core-execute'),
      getAddressEncoder().encode(owner),
    ],
  });
  assert.deepEqual(await core.findAssetSignerPda({ asset: owner }), expected);
});
