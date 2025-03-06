import test from 'ava';
import { generateSignerWithSol } from '@metaplex-foundation/umi-bundle-tests';
import { Signer, Umi, generateSigner } from '@metaplex-foundation/umi';
import * as msgpack from '@msgpack/msgpack';
import {
  assertAsset,
  createUmi,
  DEFAULT_ASSET,
  assertCollection,
} from '../_setupRaw';
import { createAsset } from '../_setupSdk';
import {
  ExternalPluginAdapterSchema,
  PluginAuthority,
  PluginAuthorityType,
  updatePlugin,
  writeData,
  create,
  createCollection,
  updateCollectionPlugin,
  removePlugin,
  removeExternalPluginAdapterV1,
} from '../../src';

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
  umi: Umi;
  owner: Signer;
  dataAuthoritySigner: Signer;
  dataAuthority: PluginAuthority;
  wrongDataAuthoritySigner?: Signer;
  wrongDataAuthority?: PluginAuthority;
  data: Uint8Array;
  otherData: Uint8Array;
};

async function generateTestContext(
  dataAuthorityType: PluginAuthorityType,
  schema: ExternalPluginAdapterSchema,
  wrongDataAuthorityType?: PluginAuthorityType
): Promise<TestContext> {
  const umi = await createUmi();

  const owner = await generateSignerWithSol(umi);
  let dataAuthoritySigner = null;
  let dataAuthority = null;
  if (dataAuthorityType === 'Address') {
    dataAuthoritySigner = await generateSignerWithSol(umi);
    dataAuthority = {
      type: dataAuthorityType,
      address: dataAuthoritySigner.publicKey,
    };
  } else if (dataAuthorityType === 'UpdateAuthority') {
    dataAuthoritySigner = umi.identity;
    dataAuthority = { type: dataAuthorityType };
  } else if (dataAuthorityType === 'Owner') {
    dataAuthoritySigner = owner;
    dataAuthority = { type: dataAuthorityType };
  }

  let wrongDataAuthoritySigner;
  let wrongDataAuthority;
  if (wrongDataAuthorityType) {
    if (wrongDataAuthorityType === 'Address') {
      wrongDataAuthoritySigner = await generateSignerWithSol(umi);
      wrongDataAuthority = {
        type: wrongDataAuthorityType,
        address: wrongDataAuthoritySigner.publicKey,
      };
    } else if (wrongDataAuthorityType === 'UpdateAuthority') {
      wrongDataAuthoritySigner = umi.identity;
      wrongDataAuthority = { type: wrongDataAuthorityType };
    } else if (wrongDataAuthorityType === 'Owner') {
      wrongDataAuthoritySigner = owner;
      wrongDataAuthority = { type: wrongDataAuthorityType };
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

  if (!dataAuthoritySigner) {
    throw new Error('Data authority signer not set');
  }
  if (!dataAuthority) {
    throw new Error('Data authority not set');
  }

  return {
    umi,
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
      const { umi, dataAuthority, owner } = await generateTestContext(
        dataAuthorityType,
        schema
      );

      // create asset referencing the oracle account
      const asset = await createAsset(umi, {
        owner: owner.publicKey,
        plugins: [
          {
            type: 'AppData',
            dataAuthority,
            schema,
          },
        ],
      });

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
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
      const { umi, owner, dataAuthoritySigner, dataAuthority, data } =
        await generateTestContext(dataAuthorityType, schema);

      // create asset with the Secure App Data
      const asset = await createAsset(umi, {
        owner: owner.publicKey,
        plugins: [
          {
            type: 'AppData',
            dataAuthority,
            schema,
          },
        ],
      });

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
          },
        ],
      });

      await writeData(umi, {
        key: {
          type: 'AppData',
          dataAuthority,
        },
        authority: dataAuthoritySigner,
        data,
        asset: asset.publicKey,
      }).sendAndConfirm(umi);

      let assertData = null;
      if (schema === ExternalPluginAdapterSchema.Binary) {
        assertData = data;
      } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
        assertData = msgpack.decode(data);
      } else if (schema === ExternalPluginAdapterSchema.Json) {
        assertData = JSON.parse(Buffer.from(data).toString());
      }

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
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
        umi,
        owner,
        dataAuthoritySigner,
        dataAuthority,
        data,
        otherData,
      } = await generateTestContext(dataAuthorityType, schema);

      // create asset with the Secure App Data
      const asset = await createAsset(umi, {
        owner: owner.publicKey,
        plugins: [
          {
            type: 'AppData',
            dataAuthority,
            schema,
          },
        ],
      });

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema,
          },
        ],
      });

      await writeData(umi, {
        key: {
          type: 'AppData',
          dataAuthority,
        },
        authority: dataAuthoritySigner,
        data: Uint8Array.from(Buffer.from(data)),
        asset: asset.publicKey,
      }).sendAndConfirm(umi);

      let assertData = null;
      if (schema === ExternalPluginAdapterSchema.Binary) {
        assertData = data;
      } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
        assertData = msgpack.decode(data);
      } else if (schema === ExternalPluginAdapterSchema.Json) {
        assertData = JSON.parse(Buffer.from(data).toString());
      }

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
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

      await writeData(umi, {
        key: {
          type: 'AppData',
          dataAuthority,
        },
        authority: dataAuthoritySigner,
        data: Uint8Array.from(Buffer.from(otherData)),
        asset: asset.publicKey,
      }).sendAndConfirm(umi);

      if (schema === ExternalPluginAdapterSchema.Binary) {
        assertData = otherData;
      } else if (schema === ExternalPluginAdapterSchema.MsgPack) {
        assertData = msgpack.decode(otherData);
      } else if (schema === ExternalPluginAdapterSchema.Json) {
        assertData = JSON.parse(Buffer.from(otherData).toString());
      }

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
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
      const { umi, owner, dataAuthority, wrongDataAuthoritySigner, data } =
        await generateTestContext(
          dataAuthorityType,
          ExternalPluginAdapterSchema.Binary,
          otherDataAuthorityType
        );
      // create asset with the Secure App Data
      const asset = await createAsset(umi, {
        owner: owner.publicKey,
        plugins: [
          {
            type: 'AppData',
            dataAuthority,
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      });

      await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: owner.publicKey,
        appDatas: [
          {
            type: 'AppData',
            authority: { type: 'UpdateAuthority' },
            dataAuthority,
            schema: ExternalPluginAdapterSchema.Binary,
          },
        ],
      });

      const res = writeData(umi, {
        key: {
          type: 'AppData',
          dataAuthority,
        },
        authority: wrongDataAuthoritySigner,
        data: Uint8Array.from(Buffer.from(data)),
        asset: asset.publicKey,
      }).sendAndConfirm(umi);

      await t.throwsAsync(res, { name: 'InvalidAuthority' });
    });
  });

  test(`it cannot write data to a secure app data as ${dataAuthorityType} data authority if the data authority is None`, async (t) => {
    const { umi, owner, dataAuthoritySigner, data } = await generateTestContext(
      dataAuthorityType,
      ExternalPluginAdapterSchema.Binary
    );

    // create asset with the Secure App Data
    const asset = await createAsset(umi, {
      owner: owner.publicKey,
      plugins: [
        {
          type: 'AppData',
          dataAuthority: { type: 'None' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    });

    await assertAsset(t, umi, {
      ...DEFAULT_ASSET,
      asset: asset.publicKey,
      owner: owner.publicKey,
      appDatas: [
        {
          type: 'AppData',
          authority: { type: 'UpdateAuthority' },
          dataAuthority: { type: 'None' },
          schema: ExternalPluginAdapterSchema.Binary,
        },
      ],
    });

    const res = writeData(umi, {
      key: {
        type: 'AppData',
        dataAuthority: { type: 'None' },
      },
      authority: dataAuthoritySigner,
      data: Uint8Array.from(Buffer.from(data)),
      asset: asset.publicKey,
    }).sendAndConfirm(umi);

    await t.throwsAsync(res, { name: 'InvalidAuthority' });
  });
});

test(`updating a plugin before a secure app data does not corrupt the data`, async (t) => {
  const { umi, owner, dataAuthoritySigner, dataAuthority, data } =
    await generateTestContext(
      'UpdateAuthority',
      ExternalPluginAdapterSchema.Json
    );

  // create asset with the Secure App Data
  const asset = await createAsset(umi, {
    owner: owner.publicKey,
    plugins: [
      {
        type: 'Attributes',
        attributeList: [
          {
            key: 'Test',
            value: 'Test',
          },
        ],
      },
      {
        type: 'AppData',
        dataAuthority,
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await writeData(umi, {
    key: {
      type: 'AppData',
      dataAuthority,
    },
    authority: dataAuthoritySigner,
    data: Uint8Array.from(Buffer.from(data)),
    asset: asset.publicKey,
  }).sendAndConfirm(umi);

  const assertData = JSON.parse(Buffer.from(data).toString());

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
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

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: {
      type: 'Attributes',
      attributeList: [{ key: 'Updated Test', value: 'Updated Test' }],
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
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

  await updatePlugin(umi, {
    asset: asset.publicKey,
    plugin: { type: 'Attributes', attributeList: [{ key: '', value: '' }] },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: owner.publicKey,
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
  const umi = await createUmi();
  const asset = generateSigner(umi);
  const dataAuthority = generateSigner(umi);
  const appDataUpdateAuthority = generateSigner(umi);

  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
        initPluginAuthority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await updatePlugin(umi, {
    asset: asset.publicKey,
    authority: appDataUpdateAuthority,
    plugin: {
      key: {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
      },
      type: 'AppData',
      schema: ExternalPluginAdapterSchema.Binary,
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot update app data using update authority when different from external plugin authority', async (t) => {
  const umi = await createUmi();
  const asset = generateSigner(umi);
  const dataAuthority = generateSigner(umi);
  const appDataUpdateAuthority = generateSigner(umi);

  await create(umi, {
    asset,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
        initPluginAuthority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const result = updatePlugin(umi, {
    asset: asset.publicKey,
    authority: umi.identity,
    plugin: {
      key: {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
      },
      type: 'AppData',
      schema: ExternalPluginAdapterSchema.Binary,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertAsset(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    owner: umi.identity.publicKey,
    asset: asset.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});

test('it can update app data on collection with external plugin authority different than asset update authority', async (t) => {
  const umi = await createUmi();
  const collection = generateSigner(umi);
  const dataAuthority = generateSigner(umi);
  const appDataUpdateAuthority = generateSigner(umi);

  await createCollection(umi, {
    collection,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
        initPluginAuthority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  await updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    authority: appDataUpdateAuthority,
    plugin: {
      key: {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
      },
      type: 'AppData',
      schema: ExternalPluginAdapterSchema.Binary,
    },
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });
});

test('it cannot update app data on collection using update authority when different from external plugin authority', async (t) => {
  const umi = await createUmi();
  const collection = generateSigner(umi);
  const dataAuthority = generateSigner(umi);
  const appDataUpdateAuthority = generateSigner(umi);

  await createCollection(umi, {
    collection,
    name: 'Test name',
    uri: 'https://example.com',
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
        initPluginAuthority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
      },
    ],
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });

  const result = updateCollectionPlugin(umi, {
    collection: collection.publicKey,
    authority: umi.identity,
    plugin: {
      key: {
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
      },
      type: 'AppData',
      schema: ExternalPluginAdapterSchema.Binary,
    },
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await assertCollection(t, umi, {
    uri: 'https://example.com',
    name: 'Test name',
    collection: collection.publicKey,
    updateAuthority: umi.identity.publicKey,
    appDatas: [
      {
        authority: {
          type: 'Address',
          address: appDataUpdateAuthority.publicKey,
        },
        type: 'AppData',
        dataAuthority: { type: 'Address', address: dataAuthority.publicKey },
        schema: ExternalPluginAdapterSchema.Json,
      },
    ],
  });
});

test('Data offsets are correctly bumped when removing other plugins', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      { type: 'FreezeDelegate', frozen: false },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    },
    data: new Uint8Array([1, 2, 3, 4]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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

  await removePlugin(umi, {
    asset: asset.publicKey,
    plugin: { type: 'FreezeDelegate' },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    },
    data: new Uint8Array([5, 6, 7, 8]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    },
    data: new Uint8Array([1, 2, 3, 4]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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

  await removeExternalPluginAdapterV1(umi, {
    asset: asset.publicKey,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'Owner' },
    },
    data: new Uint8Array([1, 2, 3, 4]),
  }).sendAndConfirm(umi);

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    },
    data: new Uint8Array([5, 6, 7, 8]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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

  await removeExternalPluginAdapterV1(umi, {
    asset: asset.publicKey,
    key: {
      __kind: 'AppData',
      fields: [{ __kind: 'Owner' }],
    },
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'Owner' },
    },
    data: new Uint8Array([1, 2, 3, 4]),
  }).sendAndConfirm(umi);

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    },
    data: new Uint8Array([5, 6, 7, 8]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'Owner' },
    },
    data: new Uint8Array([1, 2]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'AppData',
        dataAuthority: { type: 'Owner' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
      {
        type: 'AppData',
        dataAuthority: { type: 'UpdateAuthority' },
        schema: ExternalPluginAdapterSchema.Binary,
      },
    ],
  });

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'Owner' },
    },
    data: new Uint8Array([1, 2, 3, 4]),
  }).sendAndConfirm(umi);

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'UpdateAuthority' },
    },
    data: new Uint8Array([5, 6, 7, 8]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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

  await writeData(umi, {
    asset: asset.publicKey,
    key: {
      type: 'AppData',
      dataAuthority: { type: 'Owner' },
    },
    data: new Uint8Array([1, 2, 3, 4, 1, 2]),
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    updateAuthority: { type: 'Address', address: umi.identity.publicKey },
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
