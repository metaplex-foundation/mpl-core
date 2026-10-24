import { getBaseGroupV1AccountDataEncoder } from '../dist/generated/types/baseGroupV1AccountData.js';
import { getBaseCollectionV1AccountDataEncoder } from '../dist/generated/types/baseCollectionV1AccountData.js';
import { getBaseAssetV1AccountDataEncoder } from '../dist/generated/types/baseAssetV1AccountData.js';
import assert from 'node:assert/strict';
import test from 'node:test';
import { encode } from '@msgpack/msgpack';
import { createSolanaRpcFromTransport, none, some } from '@solana/kit';
import {
  getAssetV1AccountDataCodec,
  getAssetV1AccountDataDecoder,
  getCollectionV1AccountDataCodec,
  getCollectionV1AccountDataDecoder,
  getGroupV1AccountDataCodec,
  getGroupV1AccountDataDecoder,
} from '../dist/hooked/index.js';
import {
  ExternalPluginAdapterSchema,
  ExternalPluginAdapterType,
  getExternalPluginAdapterEncoder,
  getPluginEncoder,
  HookableLifecycleEvent,
  Key,
  PluginType,
} from '../dist/generated/types/index.js';
import {
  fetchAllAssetV1,
  fetchAllCollectionV1,
  fetchAllGroupV1,
  MPL_CORE_PROGRAM_ADDRESS,
} from '../dist/index.js';
import { getPluginHeaderV1Encoder } from '../dist/generated/accounts/pluginHeaderV1.js';
import { getPluginRegistryV1Encoder } from '../dist/generated/accounts/pluginRegistryV1.js';

const owner = '11111111111111111111111111111111';
const asset = () =>
  getBaseAssetV1AccountDataEncoder().encode({
    name: 'Asset',
    owner,
    seq: none(),
    updateAuthority: { __kind: 'Address', fields: [owner] },
    uri: 'asset.json',
  });
const collection = () =>
  getBaseCollectionV1AccountDataEncoder().encode({
    currentSize: 1,
    name: 'Collection',
    numMinted: 1,
    updateAuthority: owner,
    uri: 'collection.json',
  });
const group = () =>
  getBaseGroupV1AccountDataEncoder().encode({
    assets: [],
    collections: [],
    groups: [],
    name: 'Group',
    parentGroups: [],
    updateAuthority: owner,
    uri: 'group.json',
  });
const concat = (...bytes) => new Uint8Array(Buffer.concat(bytes));

function accountFixture(base = asset(), plugins = [], external = []) {
  let offset = base.length + 9;
  const payloads = [];
  const registry = plugins.map((plugin) => {
    const bytes = getPluginEncoder().encode(plugin);
    const record = {
      authority: { __kind: 'Address', address: owner },
      offset: BigInt(offset),
      pluginType: PluginType[plugin.__kind],
    };
    offset += bytes.length;
    payloads.push(bytes);
    return record;
  });
  const externalRegistry = external.map(
    ({ adapter: inputAdapter, bytes, lifecycleChecks = none(), schema }) => {
      const adapter = inputAdapter ?? {
        __kind: 'AppData',
        fields: [{ dataAuthority: { __kind: 'Owner' }, schema }],
      };
      const encoded = getExternalPluginAdapterEncoder().encode(adapter);
      const record = {
        authority: { __kind: 'UpdateAuthority' },
        dataLen: bytes ? some(BigInt(bytes.length)) : none(),
        dataOffset: bytes ? some(BigInt(offset + encoded.length)) : none(),
        lifecycleChecks,
        offset: BigInt(offset),
        pluginType: ExternalPluginAdapterType[adapter.__kind],
      };
      offset += encoded.length + (bytes?.length ?? 0);
      payloads.push(encoded, ...(bytes ? [bytes] : []));
      return record;
    }
  );
  const header = getPluginHeaderV1Encoder().encode({
    key: Key.PluginHeaderV1,
    pluginRegistryOffset: BigInt(offset),
  });
  const registryBytes = getPluginRegistryV1Encoder().encode({
    externalRegistry,
    key: Key.PluginRegistryV1,
    registry,
  });
  return {
    bytes: concat(base, header, ...payloads, registryBytes),
    externalRegistry,
    headerOffset: base.length,
    registry,
    registryOffset: offset,
  };
}

for (const [name, base, decoder, fetchBatch] of [
  ['asset', asset, getAssetV1AccountDataDecoder, fetchAllAssetV1],
  [
    'collection',
    collection,
    getCollectionV1AccountDataDecoder,
    fetchAllCollectionV1,
  ],
  ['group', group, getGroupV1AccountDataDecoder, fetchAllGroupV1],
]) {
  test(`${name}: decodes base accounts and rejects other account keys`, () => {
    const bytes = base();
    const decoded = decoder().decode(bytes);
    assert.deepEqual(decoded.externalPluginAdapters, []);
    assert.deepEqual(decoded.plugins, []);
    assert.equal(decoded.pluginHeader, undefined);
    const wrong = Uint8Array.from(bytes);
    wrong[0] = Key.HashedAssetV1;
    assert.throws(() => decoder().decode(wrong), /account key/i);
  });

  test(`${name}: rejects missing registry length prefixes`, () => {
    const fixture = accountFixture(base());
    for (const registryLength of [1, 5]) {
      assert.throws(() =>
        decoder().decode(
          fixture.bytes.slice(0, fixture.registryOffset + registryLength)
        )
      );
    }
  });

  test(`${name}: decodes multiple plugins and registry authorities`, () => {
    const plugins = [
      { __kind: 'FreezeDelegate', fields: [{ frozen: true }] },
      {
        __kind: 'Attributes',
        fields: [{ attributeList: [{ key: 'color', value: 'blue' }] }],
      },
    ];
    const fixture = accountFixture(base(), plugins);
    const decoded = decoder().decode(fixture.bytes);
    assert.deepEqual(
      decoded.plugins.map(({ plugin }) => plugin),
      plugins
    );
    assert.deepEqual(
      decoded.plugins.map(({ authority }) => authority),
      fixture.registry.map(({ authority }) => authority)
    );
    assert.equal(
      decoded.pluginHeader.pluginRegistryOffset,
      BigInt(fixture.registryOffset)
    );
  });

  test(`${name}: decodes external plugins through the same account decoder`, () => {
    const fixture = accountFixture(
      base(),
      [],
      [
        {
          bytes: new TextEncoder().encode('{"account":true}'),
          schema: ExternalPluginAdapterSchema.Json,
        },
      ]
    );
    assert.deepEqual(
      decoder().decode(fixture.bytes).externalPluginAdapters[0].data,
      { account: true }
    );
  });
  for (const [label, bytes, schema] of [
    [
      'invalid JSON',
      new TextEncoder().encode('{'),
      ExternalPluginAdapterSchema.Json,
    ],
    [
      'invalid MessagePack',
      new Uint8Array([0xc1]),
      ExternalPluginAdapterSchema.MsgPack,
    ],
    ['invalid UTF-8', new Uint8Array([0xff]), ExternalPluginAdapterSchema.Json],
  ]) {
    test(`${name}: preserves account data when an external payload has ${label}`, () => {
      const plugin = { __kind: 'FreezeDelegate', fields: [{ frozen: false }] };
      const fixture = accountFixture(
        base(),
        [plugin],
        [
          { bytes, schema },
          {
            bytes: new TextEncoder().encode('{"valid":true}'),
            schema: ExternalPluginAdapterSchema.Json,
          },
        ]
      );
      const decoded = decoder().decode(fixture.bytes);
      const failed = decoded.externalPluginAdapters[0];
      assert.equal(failed.adapter.__kind, 'AppData');
      assert.deepEqual(failed.rawData, bytes);
      assert.equal(failed.data, undefined);
      assert.ok(failed.dataError instanceof Error);
      assert.deepEqual(decoded.plugins[0].plugin, plugin);
      assert.deepEqual(decoded.externalPluginAdapters[1].data, { valid: true });
      assert.equal(decoded.externalPluginAdapters[1].dataError, undefined);
    });
  }

  test(`${name}: batch fetching survives one account with invalid external data`, async () => {
    const invalidData = new TextEncoder().encode('{');
    const fixture = accountFixture(
      base(),
      [],
      [{ bytes: invalidData, schema: ExternalPluginAdapterSchema.Json }]
    );
    const rpc = createSolanaRpcFromTransport(async ({ payload }) => {
      assert.equal(payload.method, 'getMultipleAccounts');
      return {
        id: payload.id,
        jsonrpc: '2.0',
        result: {
          context: { slot: 1 },
          value: [fixture.bytes, base()].map((bytes) => ({
            data: [Buffer.from(bytes).toString('base64'), 'base64'],
            executable: false,
            lamports: 1,
            owner: MPL_CORE_PROGRAM_ADDRESS,
            rentEpoch: 0,
            space: bytes.length,
          })),
        },
      };
    });
    const accounts = await fetchBatch(rpc, [owner, MPL_CORE_PROGRAM_ADDRESS]);
    assert.equal(accounts.length, 2);
    assert.deepEqual(
      accounts[0].data.externalPluginAdapters[0].rawData,
      invalidData
    );
    assert.ok(
      accounts[0].data.externalPluginAdapters[0].dataError instanceof Error
    );
    assert.deepEqual(accounts[1].data.externalPluginAdapters, []);
  });
}

test('decodes binary, JSON, and MessagePack external data with raw bytes', () => {
  const external = [
    {
      bytes: new Uint8Array([0, 255, 5]),
      schema: ExternalPluginAdapterSchema.Binary,
    },
    {
      bytes: new TextEncoder().encode('{"hello":"json"}'),
      schema: ExternalPluginAdapterSchema.Json,
    },
    {
      bytes: encode({ hello: 'msgpack' }),
      schema: ExternalPluginAdapterSchema.MsgPack,
    },
  ];
  const decoded = getAssetV1AccountDataDecoder().decode(
    accountFixture(asset(), [], external).bytes
  );
  assert.deepEqual(
    decoded.externalPluginAdapters.map(({ rawData }) => rawData),
    external.map(({ bytes }) => bytes)
  );
  assert.deepEqual(
    decoded.externalPluginAdapters.map(({ data }) => data),
    [external[0].bytes, { hello: 'json' }, { hello: 'msgpack' }]
  );
  assert.deepEqual(
    decoded.externalPluginAdapters.map(({ adapter }) => adapter.__kind),
    ['AppData', 'AppData', 'AppData']
  );
});

test('reads registry offsets relative to the account at a nonzero decoder offset', () => {
  const fixture = accountFixture(asset(), [
    { __kind: 'BurnDelegate', fields: [{}] },
  ]);
  const prefix = new Uint8Array([99, 98, 97]);
  const [decoded, offset] = getAssetV1AccountDataDecoder().read(
    concat(prefix, fixture.bytes),
    prefix.length
  );
  assert.equal(decoded.plugins[0].plugin.__kind, 'BurnDelegate');
  assert.equal(offset, prefix.length + fixture.bytes.length);
});

test('rejects malformed headers and registries', () => {
  const decoder = getAssetV1AccountDataDecoder();
  const fixture = accountFixture();
  assert.throws(() =>
    decoder.decode(concat(asset(), new Uint8Array([Key.PluginHeaderV1])))
  );
  for (const registryOffset of [
    0n,
    BigInt(fixture.bytes.length),
    9007199254740993n,
  ]) {
    const bytes = Uint8Array.from(fixture.bytes);
    new DataView(bytes.buffer).setBigUint64(
      fixture.headerOffset + 1,
      registryOffset,
      true
    );
    assert.throws(() => decoder.decode(bytes), /registry offset/i);
  }
  const wrongHeader = Uint8Array.from(fixture.bytes);
  wrongHeader[fixture.headerOffset] = Key.AssetV1;
  assert.throws(() => decoder.decode(wrongHeader), /header key/i);
  const wrongRegistry = Uint8Array.from(fixture.bytes);
  wrongRegistry[fixture.registryOffset] = Key.AssetV1;
  assert.throws(() => decoder.decode(wrongRegistry), /registry key/i);
  assert.throws(() => decoder.decode(fixture.bytes.slice(0, -1)));
});

test('rejects an Attributes payload missing its length prefix', () => {
  const base = asset();
  const fixture = accountFixture(base, [
    { __kind: 'Attributes', fields: [{ attributeList: [] }] },
  ]);
  const payload = new Uint8Array([PluginType.Attributes]);
  const header = getPluginHeaderV1Encoder().encode({
    key: Key.PluginHeaderV1,
    pluginRegistryOffset: BigInt(base.length + 9 + payload.length),
  });
  assert.throws(() =>
    getAssetV1AccountDataDecoder().decode(
      concat(base, header, payload, fixture.bytes.slice(fixture.registryOffset))
    )
  );
});

test('rejects group accounts missing terminal vector length prefixes', () => {
  const bytes = group();
  for (const missingBytes of [4, 8, 12, 16]) {
    assert.throws(() =>
      getGroupV1AccountDataDecoder().decode(bytes.slice(0, -missingBytes))
    );
  }
});

test('rejects mismatched and out of bounds plugin payloads', () => {
  const fixture = accountFixture(asset(), [
    { __kind: 'FreezeDelegate', fields: [{ frozen: true }] },
  ]);
  const mismatch = Uint8Array.from(fixture.bytes);
  mismatch[Number(fixture.registry[0].offset)] = PluginType.BurnDelegate;
  assert.throws(
    () => getAssetV1AccountDataDecoder().decode(mismatch),
    /discriminant/i
  );
  const badRegistry = getPluginRegistryV1Encoder().encode({
    externalRegistry: [],
    key: Key.PluginRegistryV1,
    registry: [{ ...fixture.registry[0], offset: 0n }],
  });
  assert.throws(
    () =>
      getAssetV1AccountDataDecoder().decode(
        concat(fixture.bytes.slice(0, fixture.registryOffset), badRegistry)
      ),
    /plugin offset/i
  );
  const unknown = Uint8Array.from(fixture.bytes);
  unknown[fixture.registryOffset + 5] = 255;
  assert.throws(() => getAssetV1AccountDataDecoder().decode(unknown));
});

test('rejects external data that overlaps the registry', () => {
  const fixture = accountFixture(
    asset(),
    [],
    [{ bytes: new Uint8Array([1]), schema: ExternalPluginAdapterSchema.Binary }]
  );
  const badRegistry = getPluginRegistryV1Encoder().encode({
    externalRegistry: [
      { ...fixture.externalRegistry[0], dataLen: some(1000n) },
    ],
    key: Key.PluginRegistryV1,
    registry: [],
  });
  assert.throws(
    () =>
      getAssetV1AccountDataDecoder().decode(
        concat(fixture.bytes.slice(0, fixture.registryOffset), badRegistry)
      ),
    /data range/i
  );
});

test('decodes every current internal plugin variant', () => {
  const values = {
    AddBlocker: {},
    Attributes: { attributeList: [] },
    Autograph: { signatures: [] },
    BubblegumV2: {},
    BurnDelegate: {},
    Edition: { number: 1 },
    FreezeDelegate: { frozen: false },
    FreezeExecute: { frozen: false },
    Groups: { groups: [owner] },
    ImmutableMetadata: {},
    MasterEdition: { maxSupply: none(), name: none(), uri: none() },
    PermanentBurnDelegate: {},
    PermanentFreezeDelegate: { frozen: true },
    PermanentFreezeExecute: { frozen: true },
    PermanentTransferDelegate: {},
    Royalties: { basisPoints: 500, creators: [], ruleSet: { __kind: 'None' } },
    TransferDelegate: {},
    UpdateDelegate: { additionalDelegates: [] },
    VerifiedCreators: { signatures: [] },
  };
  const plugins = Object.entries(values).map(([__kind, value]) => ({
    __kind,
    fields: [value],
  }));
  const decoded = getAssetV1AccountDataDecoder().decode(
    accountFixture(asset(), plugins).bytes
  );
  assert.deepEqual(
    decoded.plugins.map(({ plugin }) => plugin),
    plugins
  );
  assert.equal(
    decoded.plugins.length,
    Object.keys(PluginType).filter((key) => Number.isNaN(Number(key))).length
  );
});

test('decodes every current external adapter variant and lifecycle checks', () => {
  const schema = ExternalPluginAdapterSchema.Binary;
  const authority = { __kind: 'Owner' };
  const hook = {
    dataAuthority: none(),
    extraAccounts: none(),
    hookedProgram: owner,
    schema,
  };
  const values = {
    AgentIdentity: { uri: 'agent.json' },
    AppData: { dataAuthority: authority, schema },
    DataSection: {
      parentKey: { __kind: 'LinkedAppData', fields: [authority] },
      schema,
    },
    LifecycleHook: hook,
    LinkedAppData: { dataAuthority: authority, schema },
    LinkedLifecycleHook: hook,
    Oracle: {
      baseAddress: owner,
      baseAddressConfig: none(),
      resultsOffset: { __kind: 'NoOffset' },
    },
  };
  const adapters = Object.entries(values).map(([__kind, value]) => ({
    __kind,
    fields: [value],
  }));
  const lifecycleChecks = some([
    [HookableLifecycleEvent.Transfer, { flags: 5 }],
  ]);
  const fixture = accountFixture(
    asset(),
    [],
    adapters.map((adapter) => ({ adapter, lifecycleChecks }))
  );
  const decoded = getAssetV1AccountDataDecoder().decode(fixture.bytes);
  assert.deepEqual(
    decoded.externalPluginAdapters.map(({ adapter }) => adapter),
    adapters
  );
  assert.deepEqual(
    decoded.externalPluginAdapters.map(({ lifecycleChecks: checks }) => checks),
    adapters.map(() => lifecycleChecks)
  );
  assert.equal(
    decoded.externalPluginAdapters.length,
    Object.keys(ExternalPluginAdapterType).filter((key) =>
      Number.isNaN(Number(key))
    ).length
  );
});

test('handles empty external data and rejects incomplete data ranges', () => {
  const external = [
    { bytes: new Uint8Array(), schema: ExternalPluginAdapterSchema.Binary },
    { bytes: new Uint8Array(), schema: ExternalPluginAdapterSchema.Json },
    { bytes: new Uint8Array(), schema: ExternalPluginAdapterSchema.MsgPack },
  ];
  const fixture = accountFixture(asset(), [], external);
  const decoded = getAssetV1AccountDataDecoder().decode(fixture.bytes);
  assert.deepEqual(
    decoded.externalPluginAdapters.map(({ data }) => data),
    [new Uint8Array(), undefined, null]
  );
  const record = fixture.externalRegistry[0];
  const invalidRegistry = getPluginRegistryV1Encoder().encode({
    externalRegistry: [{ ...record, dataLen: none() }],
    key: Key.PluginRegistryV1,
    registry: [],
  });
  assert.throws(
    () =>
      getAssetV1AccountDataDecoder().decode(
        concat(fixture.bytes.slice(0, fixture.registryOffset), invalidRegistry)
      ),
    /data range/i
  );
});

for (const [name, base, codecFactory] of [
  ['asset', asset, getAssetV1AccountDataCodec],
  ['collection', collection, getCollectionV1AccountDataCodec],
  ['group', group, getGroupV1AccountDataCodec],
]) {
  test(`${name}: complete account codec preserves plugin payloads and recalculates offsets`, () => {
    const fixture = accountFixture(
      base(),
      [{ __kind: 'FreezeDelegate', fields: [{ frozen: true }] }],
      [
        {
          bytes: encode({ level: 3 }),
          schema: ExternalPluginAdapterSchema.MsgPack,
        },
      ]
    );
    const codec = codecFactory();
    const decoded = codec.decode(fixture.bytes);
    assert.deepEqual(codec.encode(decoded), fixture.bytes);
    const stale = {
      ...decoded,
      externalPluginAdapters: decoded.externalPluginAdapters.map((adapter) => ({
        ...adapter,
        dataLen: some(999n),
        dataOffset: some(999n),
        offset: 999n,
      })),
      pluginHeader: { ...decoded.pluginHeader, pluginRegistryOffset: 999n },
      plugins: decoded.plugins.map((entry) => ({ ...entry, offset: 999n })),
    };
    assert.deepEqual(codec.decode(codec.encode(stale)), decoded);
    assert.deepEqual(codec.encode(codec.decode(base())), base());
    assert.throws(
      () =>
        codec.encode({
          ...decoded,
          externalPluginAdapters: decoded.externalPluginAdapters.map(
            (adapter) => ({ ...adapter, rawData: undefined })
          ),
        }),
      /rawData/
    );
  });
}
