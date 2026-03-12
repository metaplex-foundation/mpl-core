import test from 'ava';
import { generateKeyPairSigner } from '@solana/signers';
import * as msgpack from '@msgpack/msgpack';
import {
  ExternalPluginAdapterSchema,
  getWriteExternalPluginAdapterDataV1Instruction,
  getUpdatePluginV1Instruction,
  getRemoveExternalPluginAdapterV1Instruction,
  getCreateV1Instruction,
  getCreateCollectionV1Instruction,
  getUpdateCollectionPluginV1Instruction,
  getRemovePluginV1Instruction,
} from '../../src';
import {
  createAsset,
  createCollection,
  createRpc,
  createRpcSubscriptions,
  generateSignerWithSol,
  assertAsset,
  assertCollection,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
  sendAndConfirmInstructions,
} from '../_setup';
import { pluginAuthorityPair } from '../../src/plugins';

const DATA_AUTHORITIES = ['Owner', 'UpdateAuthority', 'Address'] as const;
const SCHEMAS = [
  ExternalPluginAdapterSchema.Binary,
  ExternalPluginAdapterSchema.Json,
  ExternalPluginAdapterSchema.MsgPack,
];

type DataAuthorityType = (typeof DATA_AUTHORITIES)[number];

type TestContext = {
  rpc: ReturnType<typeof createRpc>;
  payer: Awaited<ReturnType<typeof generateSignerWithSol>>;
  owner: Awaited<ReturnType<typeof generateSignerWithSol>>;
  dataAuthoritySigner: Awaited<ReturnType<typeof generateSignerWithSol>>;
  dataAuthority: { __kind: string; address?: string };
  wrongDataAuthoritySigner?: Awaited<ReturnType<typeof generateSignerWithSol>>;
  wrongDataAuthority?: { __kind: string; address?: string };
  data: Uint8Array;
  otherData: Uint8Array;
};

async function generateTestContext(
  dataAuthorityType: DataAuthorityType,
  schema: ExternalPluginAdapterSchema,
  wrongDataAuthorityType?: DataAuthorityType
): Promise<TestContext> {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const owner = await generateSignerWithSol(rpc);

  let dataAuthoritySigner;
  let dataAuthority: { __kind: string; address?: string };

  if (dataAuthorityType === 'Address') {
    dataAuthoritySigner = await generateSignerWithSol(rpc);
    dataAuthority = {
      __kind: 'Address',
      address: dataAuthoritySigner.address,
    };
  } else if (dataAuthorityType === 'UpdateAuthority') {
    dataAuthoritySigner = payer;
    dataAuthority = { __kind: 'UpdateAuthority' };
  } else if (dataAuthorityType === 'Owner') {
    dataAuthoritySigner = owner;
    dataAuthority = { __kind: 'Owner' };
  } else {
    throw new Error('Invalid data authority type');
  }

  let wrongDataAuthoritySigner;
  let wrongDataAuthority;
  if (wrongDataAuthorityType) {
    if (wrongDataAuthorityType === 'Address') {
      wrongDataAuthoritySigner = await generateSignerWithSol(rpc);
      wrongDataAuthority = {
        __kind: 'Address',
        address: wrongDataAuthoritySigner.address,
      };
    } else if (wrongDataAuthorityType === 'UpdateAuthority') {
      wrongDataAuthoritySigner = payer;
      wrongDataAuthority = { __kind: 'UpdateAuthority' };
    } else if (wrongDataAuthorityType === 'Owner') {
      wrongDataAuthoritySigner = owner;
      wrongDataAuthority = { __kind: 'Owner' };
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
    test(`it can create a secure app data with ${dataAuthorityType} as data authority and ${ExternalPluginAdapterSchema[schema]} as schema`, async (t) => {
      const { rpc, payer, owner, dataAuthority } = await generateTestContext(
        dataAuthorityType,
        schema
      );

      const asset = await createAsset(rpc, payer, {
        owner: owner.address,
        plugins: [
          pluginAuthorityPair({
            type: 'AppData',
            data: {
              dataAuthority,
              schema,
            },
          }),
        ],
      });

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
          },
        ],
      });
    });

    test(`it can write ${ExternalPluginAdapterSchema[schema]} data to a secure app data as ${dataAuthorityType} data authority`, async (t) => {
      const { rpc, payer, owner, dataAuthoritySigner, dataAuthority, data } =
        await generateTestContext(dataAuthorityType, schema);

      const asset = await createAsset(rpc, payer, {
        owner: owner.address,
        plugins: [
          pluginAuthorityPair({
            type: 'AppData',
            data: {
              dataAuthority,
              schema,
            },
          }),
        ],
      });

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
          },
        ],
      });

      const instruction = getWriteExternalPluginAdapterDataV1Instruction({
        key: {
          __kind: 'AppData',
          fields: [dataAuthority],
        },
        authority: dataAuthoritySigner,
        data,
        asset: asset.address,
        payer: payer,
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

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
            data: assertData,
          },
        ],
      });
    });

    test(`it can write ${ExternalPluginAdapterSchema[schema]} data to a secure app data as ${dataAuthorityType} data authority multiple times`, async (t) => {
      const {
        rpc,
        payer,
        owner,
        dataAuthoritySigner,
        dataAuthority,
        data,
        otherData,
      } = await generateTestContext(dataAuthorityType, schema);

      const asset = await createAsset(rpc, payer, {
        owner: owner.address,
        plugins: [
          pluginAuthorityPair({
            type: 'AppData',
            data: {
              dataAuthority,
              schema,
            },
          }),
        ],
      });

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
          },
        ],
      });

      const instruction1 = getWriteExternalPluginAdapterDataV1Instruction({
        key: {
          __kind: 'AppData',
          fields: [dataAuthority],
        },
        authority: dataAuthoritySigner,
        data: Uint8Array.from(Buffer.from(data)),
        asset: asset.address,
        payer: payer,
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

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
            data: assertData,
          },
        ],
      });

      const instruction2 = getWriteExternalPluginAdapterDataV1Instruction({
        key: {
          __kind: 'AppData',
          fields: [dataAuthority],
        },
        authority: dataAuthoritySigner,
        data: Uint8Array.from(Buffer.from(otherData)),
        asset: asset.address,
        payer: payer,
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

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
            data: assertData,
          },
        ],
      });
    });
  });

  DATA_AUTHORITIES.filter(
    (da) => da === 'Address' || da !== dataAuthorityType
  ).forEach((otherDataAuthorityType) => {
    test(`it cannot write data to a secure app data with ${dataAuthorityType} data authority using ${otherDataAuthorityType} data authority`, async (t) => {
      const { rpc, payer, owner, dataAuthority, wrongDataAuthoritySigner, data } =
        await generateTestContext(
          dataAuthorityType,
          ExternalPluginAdapterSchema.Binary,
          otherDataAuthorityType
        );

      const asset = await createAsset(rpc, payer, {
        owner: owner.address,
        plugins: [
          pluginAuthorityPair({
            type: 'AppData',
            data: {
              dataAuthority,
              schema: ExternalPluginAdapterSchema.Binary,
            },
          }),
        ],
      });

      await assertAsset(t, rpc, {
        ...DEFAULT_ASSET,
        asset: asset.address,
        owner: owner.address,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      });

      const instruction = getWriteExternalPluginAdapterDataV1Instruction({
        key: {
          __kind: 'AppData',
          fields: [dataAuthority],
        },
        authority: wrongDataAuthoritySigner,
        data: Uint8Array.from(Buffer.from(data)),
        asset: asset.address,
        payer: payer,
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

  test(`it cannot write data to a secure app data as ${dataAuthorityType} data authority if the data authority is None`, async (t) => {
    const { rpc, payer, owner, dataAuthoritySigner, data } = await generateTestContext(
      dataAuthorityType,
      ExternalPluginAdapterSchema.Binary
    );

    const asset = await createAsset(rpc, payer, {
      owner: owner.address,
      plugins: [
        pluginAuthorityPair({
          type: 'AppData',
          data: {
            dataAuthority: { __kind: 'None' },
            schema: ExternalPluginAdapterSchema.Binary,
          },
        }),
      ],
    });

    await assertAsset(t, rpc, {
      ...DEFAULT_ASSET,
      asset: asset.address,
      owner: owner.address,
      appDatas: [
        {
          type: 'AppData',
          authority: { type: 'UpdateAuthority' },
          dataAuthority: { type: 'None' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    });

    const instruction = getWriteExternalPluginAdapterDataV1Instruction({
      key: {
        __kind: 'AppData',
        fields: [{ __kind: 'None' }],
      },
      authority: dataAuthoritySigner,
      data: Uint8Array.from(Buffer.from(data)),
      asset: asset.address,
      payer: payer,
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
      ExternalPluginAdapterSchema.Json
    );

  const asset = await createAsset(rpc, payer, {
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
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority,
          schema: ExternalPluginAdapterSchema.Json,
        },
      }),
    ],
  });

  const writeInstruction = getWriteExternalPluginAdapterDataV1Instruction({
    key: {
      __kind: 'AppData',
      fields: [dataAuthority],
    },
    authority: dataAuthoritySigner,
    data: Uint8Array.from(Buffer.from(data)),
    asset: asset.address,
    payer: payer,
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction],
    [dataAuthoritySigner, payer]
  );

  const assertData = JSON.parse(Buffer.from(data).toString());

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    attributes: {
      authority: { type: 'UpdateAuthority' },
      attributeList: [{ key: 'Test', value: 'Test' }],
    },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Json,
        data: assertData,
      },
    ],
  });

  const updateInstruction1 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer: payer,
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

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    attributes: {
      authority: { type: 'UpdateAuthority' },
      attributeList: [{ key: 'Updated Test', value: 'Updated Test' }],
    },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Json,
        data: assertData,
      },
    ],
  });

  const updateInstruction2 = getUpdatePluginV1Instruction({
    asset: asset.address,
    payer: payer,
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

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: owner.address,
    attributes: {
      authority: { type: 'UpdateAuthority' },
      attributeList: [{ key: '', value: '' }],
    },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Json,
        data: assertData,
      },
    ],
  });
});

test('it can update app data with external plugin authority different than asset update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Address', address: dataAuthority.address },
          schema: ExternalPluginAdapterSchema.Json,
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      }),
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAddress, payer]
  );

  await assertAsset(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: payer.address,
    asset: assetAddress.address,
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: assetAddress.address,
    payer,
    authority: appDataUpdateAuthority,
    plugin: {
      __kind: 'AppData',
      fields: [{
        dataAuthority: { __kind: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      }],
    },
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Address', address: dataAuthority.address }],
    },
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [appDataUpdateAuthority, payer]
  );

  await assertAsset(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: payer.address,
    asset: assetAddress.address,
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot update app data using update authority when different from external plugin authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const assetAddress = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const instruction = getCreateV1Instruction({
    asset: assetAddress,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Address', address: dataAuthority.address },
          schema: ExternalPluginAdapterSchema.Json,
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      }),
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [assetAddress, payer]
  );

  await assertAsset(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: payer.address,
    asset: assetAddress.address,
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const updateInstruction = getUpdatePluginV1Instruction({
    asset: assetAddress.address,
    payer,
    authority: payer,
    plugin: {
      __kind: 'AppData',
      fields: [{
        dataAuthority: { __kind: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      }],
    },
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Address', address: dataAuthority.address }],
    },
  });

  const result = sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [updateInstruction],
    [payer]
  );

  await t.throwsAsync(result);

  await assertAsset(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: payer.address,
    asset: assetAddress.address,
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});

test('it can update app data on collection with external plugin authority different than asset update authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const instruction = getCreateCollectionV1Instruction({
    collection: collectionAddress,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Address', address: dataAuthority.address },
          schema: ExternalPluginAdapterSchema.Json,
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      }),
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [collectionAddress, payer]
  );

  await assertCollection(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collectionAddress.address,
    updateAuthority: payer.address,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const updateInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collectionAddress.address,
    payer,
    authority: appDataUpdateAuthority,
    plugin: {
      __kind: 'AppData',
      fields: [{
        dataAuthority: { __kind: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      }],
    },
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Address', address: dataAuthority.address }],
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
    collection: collectionAddress.address,
    updateAuthority: payer.address,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot update app data on collection using update authority when different from external plugin authority', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);
  const collectionAddress = await generateKeyPairSigner();
  const dataAuthority = await generateKeyPairSigner();
  const appDataUpdateAuthority = await generateKeyPairSigner();

  const instruction = getCreateCollectionV1Instruction({
    collection: collectionAddress,
    payer,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Address', address: dataAuthority.address },
          schema: ExternalPluginAdapterSchema.Json,
        },
        authority: {
          __kind: 'Address',
          address: appDataUpdateAuthority.address,
        },
      }),
    ],
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [instruction],
    [collectionAddress, payer]
  );

  await assertCollection(t, rpc, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collectionAddress.address,
    updateAuthority: payer.address,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const updateInstruction = getUpdateCollectionPluginV1Instruction({
    collection: collectionAddress.address,
    payer,
    authority: payer,
    plugin: {
      __kind: 'AppData',
      fields: [{
        dataAuthority: { __kind: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Binary,
      }],
    },
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Address', address: dataAuthority.address }],
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
    collection: collectionAddress.address,
    updateAuthority: payer.address,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.address,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.address },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});

test('Data offsets are correctly bumped when removing other plugins', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({ type: 'FreezeDelegate', data: { frozen: false } }),
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'UpdateAuthority' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
    ],
  });

  const writeInstruction = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'UpdateAuthority' }],
    },
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: {
      frozen: false,
      authority: { type: 'Owner' },
      offset: 119n,
    },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 121n,
        dataOffset: 124n,
        dataLen: 4n,
      },
    ],
  });

  const removeInstruction = getRemovePluginV1Instruction({
    asset: asset.address,
    payer,
    plugin: { __kind: 'FreezeDelegate' },
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
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
    ],
  });

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
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
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
    ],
  });
});

test('Data offsets are correctly bumped when moving other external plugins', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Owner' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'UpdateAuthority' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
    ],
  });

  const writeInstruction = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'UpdateAuthority' }],
    },
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction],
    [payer]
  );

  await assertAsset(t, rpc, {
    ...DEFAULT_ASSET,
    asset: asset.address,
    owner: payer.address,
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 0n,
      },
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 122n,
        dataOffset: 125n,
        dataLen: 4n,
      },
    ],
  });

  const removeInstruction = getRemoveExternalPluginAdapterV1Instruction({
    asset: asset.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
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
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
    ],
  });
});

test('Data offsets are correctly bumped when removing other external plugins with data', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Owner' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'UpdateAuthority' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
    ],
  });

  const writeInstruction1 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction1],
    [payer]
  );

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
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
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 126n,
        dataOffset: 129n,
        dataLen: 4n,
      },
    ],
  });

  const removeInstruction = getRemoveExternalPluginAdapterV1Instruction({
    asset: asset.address,
    payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
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
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
    ],
  });
});

test('Data offsets are correctly bumped when rewriting other external plugins to be smaller', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Owner' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'UpdateAuthority' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
    ],
  });

  const writeInstruction1 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction1],
    [payer]
  );

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
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
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 126n,
        dataOffset: 129n,
        dataLen: 4n,
      },
    ],
  });

  const writeInstruction3 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
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
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 2n,
      },
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 124n,
        dataOffset: 127n,
        dataLen: 4n,
      },
    ],
  });
});

test('Data offsets are correctly bumped when rewriting other external plugins to be larger', async (t) => {
  const rpc = createRpc();
  const payer = await generateSignerWithSol(rpc);

  const asset = await createAsset(rpc, payer, {
    plugins: [
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'Owner' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
      pluginAuthorityPair({
        type: 'AppData',
        data: {
          dataAuthority: { __kind: 'UpdateAuthority' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      }),
    ],
  });

  const writeInstruction1 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
    data: new Uint8Array([1, 2, 3, 4]),
  });

  await sendAndConfirmInstructions(
    rpc,
    createRpcSubscriptions(),
    [writeInstruction1],
    [payer]
  );

  const writeInstruction2 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
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
    updateAuthority: { type: 'Address', address: payer.address },
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 4n,
      },
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 126n,
        dataOffset: 129n,
        dataLen: 4n,
      },
    ],
  });

  const writeInstruction3 = getWriteExternalPluginAdapterDataV1Instruction({
    asset: asset.address,
    payer,
    authority: payer,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
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
    updateAuthority: { type: 'Address', address: payer.address },
    freezeDelegate: undefined,
    appDatas: [
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([1, 2, 3, 4, 1, 2]),
        offset: 119n,
        dataOffset: 122n,
        dataLen: 6n,
      },
      {
        type: 'AppData',
        authority: { type: 'UpdateAuthority' },
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
        data: new Uint8Array([5, 6, 7, 8]),
        offset: 128n,
        dataOffset: 131n,
        dataLen: 4n,
      },
    ],
  });
});
