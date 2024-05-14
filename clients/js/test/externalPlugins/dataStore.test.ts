import test from "ava";
import { assertAsset, createUmi, DEFAULT_ASSET } from "../_setupRaw";
import { createAsset } from "../_setupSdk";
import { basePluginAuthority, ExternalPluginAdapterSchema, writeData } from "../../src";

test('it can use fixed address oracle to deny update', async (t) => {
    const umi = await createUmi();

    // create asset referencing the oracle account
    const asset = await createAsset(umi, {
        plugins: [
            {
                type: 'DataStore',
                dataAuthority: {
                    type: 'UpdateAuthority',
                },
                schema: ExternalPluginAdapterSchema.Binary,
            },
        ],
    });

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        dataStores: [
            {
                type: 'DataStore',
                authority: { type: 'UpdateAuthority' },
                dataAuthority: {
                    type: 'UpdateAuthority',
                },
                schema: ExternalPluginAdapterSchema.Binary,
            },
        ],
    });

    const result = await writeData(umi, {
        key: {
            __kind: "DataStore",
            fields: [basePluginAuthority("UpdateAuthority")],
        },
        data: Buffer.from("Hello"),
        asset: asset.publicKey,
    }).sendAndConfirm(umi);

    console.log(await umi.rpc.getTransaction(result.signature));

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        dataStores: [
            {
                type: 'DataStore',
                authority: { type: 'UpdateAuthority' },
                dataAuthority: {
                    type: 'UpdateAuthority',
                },
                schema: ExternalPluginAdapterSchema.Binary,
            },
        ],
    });
});