import test from 'ava';
import { generateKeyPairSigner, type KeyPairSigner } from '@solana/signers';
import type { Rpc, SolanaRpcApi } from '@solana/rpc';
import * as msgpack from '@msgpack/msgpack';
import {
  getAddCollectionExternalPluginAdapterV1Instruction,
  getAddExternalPluginAdapterV1Instruction,
  getAddPluginV1Instruction,
  getCreateCollectionV1Instruction,
  getCreateV1Instruction,
  getRemoveExternalPluginAdapterV1Instruction,
  getUpdateCollectionExternalPluginAdapterV1Instruction,
  getUpdatePluginV1Instruction,
  getWriteExternalPluginAdapterDataV1Instruction,
  ExternalPluginAdapterSchema,
  type BaseExternalPluginAdapterKey,
} from '../../src';
import {
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  createAssetWithCollection,
  assertAsset,
  assertCollection,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../_setup';
import {
  pluginAuthority,
  type PluginAuthority,
  type PluginAuthorityType,
  pluginAuthorityPair,
  externalPluginAdapterManifest,
  linkedDataKeyToBase,
} from '../../src/plugins';

const DATA_AUTHORITIES: PluginAuthorityType[] = [
  'Owner',
  'UpdateAuthority',
  'Address',
];
const SCHEMAS: ExternalPluginAdapterSchema[] = [
  ExternalPluginAdapterSchema.Binary,
  ExternalPluginAdapterSchema.Json,
  ExternalPluginAdapterSchema.MsgPack,
];

type TestContext = {
  rpc: Rpc<SolanaRpcApi>;
  payer: KeyPairSigner;
  owner: KeyPairSigner;
  dataAuthoritySigner: KeyPairSigner;
  dataAuthority: PluginAuthority;
  wrongDataAuthoritySigner?: KeyPairSigner;
  wrongDataAuthority?: PluginAuthority;
  data: Uint8Array;
  otherData: Uint8Array;
};

async function generateTestContext(
  dataAuthorityType: PluginAuthorityType,
  schema: ExternalPluginAdapterSchema,
  wrongDataAuthorityType?: PluginAuthorityType
): Promise<TestContext> {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const owner = await generateSignerWithSol(rpc);
  let dataAuthoritySigner: KeyPairSigner;
  let dataAuthority: PluginAuthority;

  if (dataAuthorityType === 'Address') {
    dataAuthoritySigner = await generateSignerWithSol(rpc);
    dataAuthority = pluginAuthority('Address', {
      address: dataAuthoritySigner.address,
    });
  } else if (dataAuthorityType === 'UpdateAuthority') {
    dataAuthoritySigner = payer;
    dataAuthority = pluginAuthority('UpdateAuthority');
  } else if (dataAuthorityType === 'Owner') {
    dataAuthoritySigner = owner;
    dataAuthority = pluginAuthority('Owner');
  } else {
    throw new Error('Invalid data authority type');
  }

  let wrongDataAuthoritySigner: KeyPairSigner | undefined;
  let wrongDataAuthority: PluginAuthority | undefined;

  if (wrongDataAuthorityType) {
    if (wrongDataAuthorityType === 'Address') {
      wrongDataAuthoritySigner = await generateSignerWithSol(rpc);
      wrongDataAuthority = pluginAuthority('Address', {
        address: wrongDataAuthoritySigner.address,
      });
    } else if (wrongDataAuthorityType === 'UpdateAuthority') {
      wrongDataAuthoritySigner = payer;
      wrongDataAuthority = pluginAuthority('UpdateAuthority');
    } else if (wrongDataAuthorityType === 'Owner') {
      wrongDataAuthoritySigner = owner;
      wrongDataAuthority = pluginAuthority('Owner');
    }
  }

  let data = new Uint8Array();
  let otherData = new Uint8Array();

  if (schema === ExternalPluginAdapterSchema.Binary) {
    const binaryData = 'Hello, world!';
    const binaryOtherData = 'Hello, world! Hello, world!';
    data = Uint8Array.from(Buffer.from(binaryData));
    otherData = Uint8Array.from(Buffer.from(binaryOtherData));
  } else if (schema === ExternalPluginAdapterSchema.Json) {
    const dataJson = { message: 'Hello', target: 'world' };
    const otherDataJson = { message: 'Hello hello', target: 'big wide world' };
    data = Uint8Array.from(Buffer.from(JSON.stringify(dataJson)));
    otherData = Uint8Array.from(Buffer.from(JSON.stringify(otherDataJson)));
  } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
    data = msgpack.encode({ message: 'Hello', target: 'msgpack' });
    otherData = msgpack.encode({ message: 'Hello hello', target: 'msgpack' });
  }

  return {
    rpc,
    payer,
    owner,
    dataAuthoritySigner,
    dataAuthority,
    wrongDataAuthoritySigner,
    wrongDataAuthority,
    data,
    otherData,
  };
}

DATA_AUTHORITIES.forEach((dataAuthorityType) => {
  SCHEMAS.forEach((schema) => {
    test(`it can create a linked secure app data with ${dataAuthorityType} as data authority and ${ExternalPluginAdapterSchema[schema]} as schema`, async (t) => {
      const { rpc, payer, owner, dataAuthority } = await generateTestContext(
        dataAuthorityType,
        schema
      );

      // create collection with linked secure app data
      const { collection } = await createAssetWithCollection(
        rpc,
        payer,
        {
          owner: owner.address,
        },
        {
          plugins: [
            externalPluginAdapterManifest('LinkedAppData', {
              dataAuthority,
              schema,
            }),
          ],
        }
      );

      await assertCollection(t, rpc, {
        ...DEFAULT_COLLECTION,
        collection: collection.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema,
          },
        ],
      });
    });

    test(`it can write ${ExternalPluginAdapterSchema[schema]} data to a linked secure app data as ${dataAuthorityType} data authority`, async (t) => {
      const { rpc, payer, owner, dataAuthoritySigner, dataAuthority, data } =
        await generateTestContext(dataAuthorityType, schema);

      // create collection with linked secure app data
      const { asset, collection } = await createAssetWithCollection(
        rpc,
        payer,
        {
          owner: owner.address,
        },
        {
          plugins: [
            externalPluginAdapterManifest('LinkedAppData', {
              dataAuthority,
              schema,
            }),
          ],
        }
      );

      await assertCollection(t, rpc, {
        ...DEFAULT_COLLECTION,
        collection: collection.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema,
          },
        ],
      });

      const instruction = getWriteExternalPluginAdapterDataV1Instruction({
        asset: asset.address,
        collection: collection.address,
        payer,
        authority: dataAuthoritySigner,
        key: linkedDataKeyToBase({
          type: 'LinkedAppData',
          dataAuthority,
        }),
        data: Uint8Array.from(Buffer.from(data)),
      });

      await sendAndConfirmInstructions(
        rpc,
        createRpcSubscriptions(),
        [instruction],
        [dataAuthoritySigner, payer]
      );

      let assertData = null;
      if (schema === ExternalPluginAdapterSchema.Binary) {
        assertData = data;
      } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
        assertData = msgpack.decode(data);
      } else if (schema === ExternalPluginAdapterSchema.Json) {
        assertData = JSON.parse(Buffer.from(data).toString());
      }

      // check the derived asset sdk correctly injects the data
      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema,
            data: assertData,
          },
        ],
        dataSections: [
          {
            type: 'DataSection',
            parentKey: {
              type: 'LinkedAppData',
              dataAuthority,
            },
            authority: pluginAuthority('None'),
            data: assertData,
            schema,
          },
        ],
      });
    });

    test(`it can write ${ExternalPluginAdapterSchema[schema]} data to a linked secure app data as ${dataAuthorityType} data authority multiple times`, async (t) => {
      const {
        rpc,
        payer,
        owner,
        dataAuthoritySigner,
        dataAuthority,
        data,
        otherData,
      } = await generateTestContext(dataAuthorityType, schema);

      // create collection with linked secure app data
      const { asset, collection } = await createAssetWithCollection(
        rpc,
        payer,
        {
          owner: owner.address,
        },
        {
          plugins: [
            externalPluginAdapterManifest('LinkedAppData', {
              dataAuthority,
              schema,
            }),
          ],
        }
      );

      await assertCollection(t, rpc, {
        ...DEFAULT_COLLECTION,
        collection: collection.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema,
          },
        ],
      });

      const instruction1 = getWriteExternalPluginAdapterDataV1Instruction({
        asset: asset.address,
        collection: collection.address,
        payer,
        authority: dataAuthoritySigner,
        key: linkedDataKeyToBase({
          type: 'LinkedAppData',
          dataAuthority,
        }),
        data: Uint8Array.from(Buffer.from(data)),
      });

      await sendAndConfirmInstructions(
        rpc,
        createRpcSubscriptions(),
        [instruction1],
        [dataAuthoritySigner, payer]
      );

      let assertData = null;
      if (schema === ExternalPluginAdapterSchema.Binary) {
        assertData = data;
      } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
        assertData = msgpack.decode(data);
      } else if (schema === ExternalPluginAdapterSchema.Json) {
        assertData = JSON.parse(Buffer.from(data).toString());
      }

      // check the derived asset sdk correctly injects the data
      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema,
            data: assertData,
          },
        ],
        dataSections: [
          {
            type: 'DataSection',
            parentKey: {
              type: 'LinkedAppData',
              dataAuthority,
            },
            authority: pluginAuthority('None'),
            data: assertData,
            schema,
          },
        ],
      });

      const instruction2 = getWriteExternalPluginAdapterDataV1Instruction({
        asset: asset.address,
        collection: collection.address,
        payer,
        authority: dataAuthoritySigner,
        key: linkedDataKeyToBase({
          type: 'LinkedAppData',
          dataAuthority,
        }),
        data: Uint8Array.from(Buffer.from(otherData)),
      });

      await sendAndConfirmInstructions(
        rpc,
        createRpcSubscriptions(),
        [instruction2],
        [dataAuthoritySigner, payer]
      );

      if (schema === ExternalPluginAdapterSchema.Binary) {
        assertData = otherData;
      } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
        assertData = msgpack.decode(otherData);
      } else if (schema === ExternalPluginAdapterSchema.Json) {
        assertData = JSON.parse(Buffer.from(otherData).toString());
      }

      // check the derived asset sdk correctly injects the data
      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema,
            data: assertData,
          },
        ],
        dataSections: [
          {
            type: 'DataSection',
            parentKey: {
              type: 'LinkedAppData',
              dataAuthority,
            },
            authority: pluginAuthority('None'),
            data: assertData,
            schema,
          },
        ],
      });
    });
  });

  DATA_AUTHORITIES.filter(
    (da) => da === 'Address' || da !== dataAuthorityType
  ).forEach((otherDataAuthorityType) => {
    test(`it cannot write data to a linked secure app data with ${dataAuthorityType} data authority using ${otherDataAuthorityType} data authority`, async (t) => {
      const { rpc, payer, owner, dataAuthority, wrongDataAuthoritySigner, data } =
        await generateTestContext(
          dataAuthorityType,
          ExternalPluginAdapterSchema.Binary,
          otherDataAuthorityType
        );

      // create collection with linked secure app data
      const { asset, collection } = await createAssetWithCollection(
        rpc,
        payer,
        {
          owner: owner.address,
        },
        {
          plugins: [
            externalPluginAdapterManifest('LinkedAppData', {
              dataAuthority,
              schema: ExternalPluginAdapterSchema.Binary,
            }),
          ],
        }
      );

      await assertCollection(t, rpc, {
        ...DEFAULT_COLLECTION,
        collection: collection.address,
        linkedAppDatas: [
          {
            type: 'LinkedAppData',
            authority: pluginAuthority('UpdateAuthority'),
            dataAuthority,
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      });

      const instruction = getWriteExternalPluginAdapterDataV1Instruction({
        asset: asset.address,
        collection: collection.address,
        payer,
        authority: wrongDataAuthoritySigner,
        key: linkedDataKeyToBase({
          type: 'LinkedAppData',
          dataAuthority,
        }),
        data: Uint8Array.from(Buffer.from(data)),
      });

      const result = sendAndConfirmInstructions(
        rpc,
        createRpcSubscriptions(),
        [instruction],
        [wrongDataAuthoritySigner!, payer]
      );

      await t.throwsAsync(result);
    });
  });

  test(`it cannot write data to a linked secure app data as ${dataAuthorityType} data authority if the data authority is None`, async (t) => {
    const { rpc, payer, owner, dataAuthoritySigner, data } = await generateTestContext(
      dataAuthorityType,
      ExternalPluginAdapterSchema.Binary
    );

    // create collection with linked secure app data
    const { asset, collection } = await createAssetWithCollection(
      rpc,
      payer,
      {
        owner: owner.address,
      },
      {
        plugins: [
          externalPluginAdapterManifest('LinkedAppData', {
            dataAuthority: pluginAuthority('None'),
            schema: ExternalPluginAdapterSchema.Binary,
          }),
        ],
      }
    );

    await assertCollection(t, rpc, {
      ...DEFAULT_COLLECTION,
      collection: collection.address,
      linkedAppDatas: [
        {
          type: 'LinkedAppData',
          authority: pluginAuthority('UpdateAuthority'),
          dataAuthority: pluginAuthority('None'),
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    });

    const instruction = getWriteExternalPluginAdapterDataV1Instruction({
      asset: asset.address,
      collection: collection.address,
      payer,
      authority: dataAuthoritySigner,
      key: linkedDataKeyToBase({
        type: 'LinkedAppData',
        dataAuthority: pluginAuthority('None'),
      }),
      data: Uint8Array.from(Buffer.from(data)),
    });

    const result = sendAndConfirmInstructions(
      rpc,
      createRpcSubscriptions(),
      [instruction],
      [dataAuthoritySigner, payer]
    );

    await t.throwsAsync(result);
  });
});

test(`updating a plugin before a secure app data does not corrupt the data`, async (t) => {
  const { rpc, payer, owner, dataAuthoritySigner, dataAuthority, data } =
    await generateTestContext(
      'UpdateAuthority',
      ExternalPluginAdapterSchema.Binary
    );

  // create collection with linked secure app data
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {
      owner: owner.address,
      plugins: [
        pluginAuthorityPair({
          type: 'Attributes',
          data: {
            attributeList: [
              {
                key: 'Test',
                value: 'Test',
              },
            ],
          },
        }),
      ],
    },
    {
      plugins: [
        externalPluginAdapterManifest('LinkedAppData', {
          dataAuthority,
          schema: ExternalPluginAdapterSchema.Binary,
        }),
      ],
    }
  );

  const writeInstruction = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    authority: dataAuthoritySigner,
    key: linkedDataKeyToBase({
      type: 'LinkedAppData',
      dataAuthority,
    }),
    data: Uint8Array.from(Buffer.from(data)),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction],
    [dataAuthoritySigner, payer]
  );

  await assertCollection(t, rpc, {
    ...DEFAULT_COLLECTION,
    collection: collection.address,
    linkedAppDatas: [
      {
        type: 'LinkedAppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  const assertData = data;

  // check the derived asset sdk correctly injects the data
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    attributes: {
      authority: pluginAuthority('UpdateAuthority'),
      attributeList: [{ key: 'Test', value: 'Test' }],
    },
    linkedAppDatas: [
      {
        type: 'LinkedAppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Binary,
        data: assertData,
      },
    ],
    dataSections: [
      {
        type: 'DataSection',
        parentKey: {
          type: 'LinkedAppData',
          dataAuthority,
        },
        authority: pluginAuthority('None'),
        data: assertData,
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'Attributes',
      fields: [{ attributeList: [{ key: 'Updated Test', value: 'Updated Test' }] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction1],
    [payer]
  );

  // check the derived asset sdk correctly injects the data
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    attributes: {
      authority: pluginAuthority('UpdateAuthority'),
      attributeList: [{ key: 'Updated Test', value: 'Updated Test' }],
    },
    linkedAppDatas: [
      {
        type: 'LinkedAppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Binary,
        data: assertData,
      },
    ],
    dataSections: [
      {
        type: 'DataSection',
        parentKey: {
          type: 'LinkedAppData',
          dataAuthority,
        },
        authority: pluginAuthority('None'),
        data: assertData,
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    plugin: {
      __kind: 'Attributes',
      fields: [{ attributeList: [{ key: '', value: '' }] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction2],
    [payer]
  );

  // check the derived asset sdk correctly injects the data
  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    attributes: {
      authority: pluginAuthority('UpdateAuthority'),
      attributeList: [{ key: '', value: '' }],
    },
    linkedAppDatas: [
      {
        type: 'LinkedAppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Binary,
        data: assertData,
      },
    ],
    dataSections: [
      {
        type: 'DataSection',
        parentKey: {
          type: 'LinkedAppData',
          dataAuthority,
        },
        authority: pluginAuthority('None'),
        data: assertData,
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot create an asset with linked app data', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        plugin: {
          __kind: 'LinkedAppData',
          fields: [
            {
              dataAuthority: { __kind: 'Address', address: dataAuthority.address },
              schema: ExternalPluginAdapterSchema.Json,
            },
          ],
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      },
    ],
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [asset, payer]
  );

  await t.throwsAsync(result);
});

test('it cannot add linked app data to an asset', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const asset = await generateKeyPairSigner();

  const createInstruction = getCreateV1Instruction({
    asset,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [createInstruction],
    [asset, payer]
  );

  await assertAsset(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: payer.address,
    asset: asset.address,
    updateAuthority: { type: 'Address', address: payer.address },
    linkedAppDatas: undefined,
  });

  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const addInstruction = getAddPluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: {
      __kind: 'LinkedAppData',
      fields: [
        {
          dataAuthority: { __kind: 'Address', address: dataAuthority.address },
          schema: ExternalPluginAdapterSchema.Json,
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: payer.address,
    asset: asset.address,
    updateAuthority: { type: 'Address', address: payer.address },
    linkedAppDatas: undefined,
  });
});

test('it can update linked app data on collection with external plugin authority different than asset update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const createInstruction = getCreateCollectionV1Instruction({
    collection,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        plugin: {
          __kind: 'LinkedAppData',
          fields: [
            {
              dataAuthority: { __kind: 'Address', address: dataAuthority.address },
              schema: ExternalPluginAdapterSchema.Json,
            },
          ],
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [createInstruction],
    [collection, payer]
  );

  await assertCollection(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.address,
    updateAuthority: payer.address,
    linkedAppDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'LinkedAppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const updateInstruction = getUpdateCollectionExternalPluginAdapterV1Instruction({
    collection: collection.address,
    payer,
    authority: appDataUpdateAuthority,
    key: {
      __kind: 'LinkedAppData',
      fields: [{ __kind: 'Address', address: dataAuthority.address }],
    },
    updateInfo: {
      __kind: 'LinkedAppData',
      fields: [
        {
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [appDataUpdateAuthority, payer]
  );

  await assertCollection(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.address,
    updateAuthority: payer.address,
    linkedAppDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'LinkedAppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot update linked app data on collection using update authority when different from external plugin authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collection = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const createInstruction = getCreateCollectionV1Instruction({
    collection,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        plugin: {
          __kind: 'LinkedAppData',
          fields: [
            {
              dataAuthority: { __kind: 'Address', address: dataAuthority.address },
              schema: ExternalPluginAdapterSchema.Json,
            },
          ],
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      },
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [createInstruction],
    [collection, payer]
  );

  await assertCollection(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.address,
    updateAuthority: payer.address,
    linkedAppDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'LinkedAppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const updateInstruction = getUpdateCollectionExternalPluginAdapterV1Instruction({
    collection: collection.address,
    payer,
    authority: payer,
    key: {
      __kind: 'LinkedAppData',
      fields: [{ __kind: 'Address', address: dataAuthority.address }],
    },
    updateInfo: {
      __kind: 'LinkedAppData',
      fields: [
        {
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertCollection(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.address,
    updateAuthority: payer.address,
    linkedAppDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'LinkedAppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});

test('Data offsets are correctly bumped when removing Data Section with data', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        externalPluginAdapterManifest('LinkedAppData', {
          dataAuthority: pluginAuthority('Owner'),
          schema: ExternalPluginAdapterSchema.Binary,
        }),
      ],
    }
  );

  const writeInstruction1 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: linkedDataKeyToBase({
      type: 'LinkedAppData',
      dataAuthority: pluginAuthority('Owner'),
    }),
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction1],
    [payer]
  );

  const addInstruction = getAddExternalPluginAdapterV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    initInfo: {
      __kind: 'AppData',
      fields: [
        {
          dataAuthority: { __kind: 'UpdateAuthority' },
          initPluginAuthority: null,
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'UpdateAuthority' }],
    },
    data: new Uint8Array([5, 6, 7, 8]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction2],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    dataSections: [
      {
        type: 'DataSection',
        parentKey: { type: 'LinkedAppData', dataAuthority: pluginAuthority('Owner') },
        authority: pluginAuthority('None'),
        dataAuthority: pluginAuthority('Owner'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 123n,
        dataLen: 4n,
      },
    ],
    appDatas: [
      {
        type: 'AppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority: pluginAuthority('UpdateAuthority'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 127n,
        dataOffset: 130n,
        dataLen: 4n,
      },
    ],
  });

  const removeInstruction = getRemoveExternalPluginAdapterV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: {
      __kind: 'DataSection',
      fields: [{ __kind: 'LinkedAppData', fields: [{ __kind: 'Owner' }] }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [removeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    freezeDelegate: undefined,
    dataSections: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority: pluginAuthority('UpdateAuthority'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
    ],
  });
});

test('Data offsets are correctly bumped when rewriting Data Section to be smaller', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        externalPluginAdapterManifest('LinkedAppData', {
          dataAuthority: pluginAuthority('Owner'),
          schema: ExternalPluginAdapterSchema.Binary,
        }),
      ],
    }
  );

  const writeInstruction1 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: linkedDataKeyToBase({
      type: 'LinkedAppData',
      dataAuthority: pluginAuthority('Owner'),
    }),
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction1],
    [payer]
  );

  const addInstruction = getAddExternalPluginAdapterV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    initInfo: {
      __kind: 'AppData',
      fields: [
        {
          dataAuthority: { __kind: 'UpdateAuthority' },
          initPluginAuthority: null,
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'UpdateAuthority' }],
    },
    data: new Uint8Array([5, 6, 7, 8]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction2],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    dataSections: [
      {
        type: 'DataSection',
        parentKey: { type: 'LinkedAppData', dataAuthority: pluginAuthority('Owner') },
        authority: pluginAuthority('None'),
        dataAuthority: pluginAuthority('Owner'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 123n,
        dataLen: 4n,
      },
    ],
    appDatas: [
      {
        type: 'AppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority: pluginAuthority('UpdateAuthority'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 127n,
        dataOffset: 130n,
        dataLen: 4n,
      },
    ],
  });

  const writeInstruction3 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: linkedDataKeyToBase({
      type: 'LinkedAppData',
      dataAuthority: pluginAuthority('Owner'),
    }),
    data: new Uint8Array([1, 2]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction3],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    dataSections: [
      {
        type: 'DataSection',
        parentKey: { type: 'LinkedAppData', dataAuthority: pluginAuthority('Owner') },
        authority: pluginAuthority('None'),
        dataAuthority: pluginAuthority('Owner'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2]),
        offset: 119n,
        dataOffset: 123n,
        dataLen: 2n,
      },
    ],
    appDatas: [
      {
        type: 'AppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority: pluginAuthority('UpdateAuthority'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 125n,
        dataOffset: 128n,
        dataLen: 4n,
      },
    ],
  });
});

test('Data offsets are correctly bumped when rewriting other Data Section to be larger', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const { asset, collection } = await createAssetWithCollection(
    rpc,
    payer,
    {},
    {
      plugins: [
        externalPluginAdapterManifest('LinkedAppData', {
          dataAuthority: pluginAuthority('Owner'),
          schema: ExternalPluginAdapterSchema.Binary,
        }),
      ],
    }
  );

  const writeInstruction1 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: linkedDataKeyToBase({
      type: 'LinkedAppData',
      dataAuthority: pluginAuthority('Owner'),
    }),
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction1],
    [payer]
  );

  const addInstruction = getAddExternalPluginAdapterV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    initInfo: {
      __kind: 'AppData',
      fields: [
        {
          dataAuthority: { __kind: 'UpdateAuthority' },
          initPluginAuthority: null,
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [addInstruction],
    [payer]
  );

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'UpdateAuthority' }],
    },
    data: new Uint8Array([5, 6, 7, 8]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction2],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    dataSections: [
      {
        type: 'DataSection',
        parentKey: { type: 'LinkedAppData', dataAuthority: pluginAuthority('Owner') },
        authority: pluginAuthority('None'),
        dataAuthority: pluginAuthority('Owner'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 123n,
        dataLen: 4n,
      },
    ],
    appDatas: [
      {
        type: 'AppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority: pluginAuthority('UpdateAuthority'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 127n,
        dataOffset: 130n,
        dataLen: 4n,
      },
    ],
  });

  const writeInstruction3 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    collection: collection.address,
    payer,
    key: linkedDataKeyToBase({
      type: 'LinkedAppData',
      dataAuthority: pluginAuthority('Owner'),
    }),
    data: new Uint8Array([1, 2, 3, 4, 1, 2]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction3],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Collection', address: collection.address },
    dataSections: [
      {
        type: 'DataSection',
        parentKey: { type: 'LinkedAppData', dataAuthority: pluginAuthority('Owner') },
        authority: pluginAuthority('None'),
        dataAuthority: pluginAuthority('Owner'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4, 1, 2]),
        offset: 119n,
        dataOffset: 123n,
        dataLen: 6n,
      },
    ],
    appDatas: [
      {
        type: 'AppData',
        authority: pluginAuthority('UpdateAuthority'),
        dataAuthority: pluginAuthority('UpdateAuthority'),
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 129n,
        dataOffset: 132n,
        dataLen: 4n,
      },
    ],
  });
});
