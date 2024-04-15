import { generateSigner, TransactionBuilder } from "@metaplex-foundation/umi";
import test from "ava";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { addPluginV1, addressPluginAuthority, createCollectionV1, createPlugin, createV1, pluginAuthorityPair, PluginType, revokePluginAuthorityV1, transferV1, updatePluginV1 } from "../src";
import { createUmi } from "./_setup";

test('list an asset', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);
    const marketplace = generateSigner(umi);

    await createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
    }).sendAndConfirm(umi);

    let builder = addPluginV1(umi, {
        asset: assetAddress.publicKey,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: true },
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    });

    builder = builder.add(addPluginV1(umi, {
        asset: assetAddress.publicKey,
        plugin: createPlugin({
            type: "TransferDelegate",
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    }));

    const tx = await builder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('sell an asset', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);
    const marketplace = generateSigner(umi);

    await createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
    }).sendAndConfirm(umi);

    let builder = addPluginV1(umi, {
        asset: assetAddress.publicKey,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: true },
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    });

    builder = builder.add(addPluginV1(umi, {
        asset: assetAddress.publicKey,
        plugin: createPlugin({
            type: "TransferDelegate",
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    }));

    await builder.sendAndConfirm(umi);

    let sellBuilder = transferV1(umi, {
        asset: assetAddress.publicKey,
        newOwner: newOwner.publicKey,
        payer: marketplace,
    });

    sellBuilder = updatePluginV1(umi, {
        asset: assetAddress.publicKey,
        payer: marketplace,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: false },
        }),
    });

    sellBuilder = sellBuilder.add(revokePluginAuthorityV1(umi, {
        asset: assetAddress.publicKey,
        payer: marketplace,
        pluginType: PluginType.TransferDelegate,
    }));

    sellBuilder = sellBuilder.add(revokePluginAuthorityV1(umi, {
        asset: assetAddress.publicKey,
        payer: marketplace,
        pluginType: PluginType.FreezeDelegate,
    }));

    const tx = await sellBuilder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('list an asset with empty collection', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);
    const marketplace = generateSigner(umi);

    let builder = new TransactionBuilder();
    builder = builder.add(createCollectionV1(umi, {
        collection: collectionAddress,
        name: "Test",
        uri: "www.test.com",
    }));
    builder = builder.add(createV1(umi, {
        asset: assetAddress,
        collection: collectionAddress.publicKey,
        name: "Test",
        uri: "www.test.com",
    }));

    await builder.sendAndConfirm(umi);

    let listBuilder = addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: true },
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    });

    listBuilder = listBuilder.add(addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "TransferDelegate",
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    }));

    const tx = await listBuilder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('sell an asset with empty collection', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);
    const marketplace = generateSigner(umi);

    let builder = new TransactionBuilder();
    builder = builder.add(createCollectionV1(umi, {
        collection: collectionAddress,
        name: "Test",
        uri: "www.test.com",
    }));
    builder = builder.add(createV1(umi, {
        asset: assetAddress,
        collection: collectionAddress.publicKey,
        name: "Test",
        uri: "www.test.com",
    }));

    await builder.sendAndConfirm(umi);

    let listBuilder = addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: true },
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    });

    listBuilder = listBuilder.add(addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "TransferDelegate",
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    }));

    await listBuilder.sendAndConfirm(umi);

    let sellBuilder = transferV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        newOwner: newOwner.publicKey,
        payer: marketplace,
    });

    sellBuilder = updatePluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        payer: marketplace,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: false },
        }),
    });

    sellBuilder = sellBuilder.add(revokePluginAuthorityV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        payer: marketplace,
        pluginType: PluginType.TransferDelegate,
    }));

    sellBuilder = sellBuilder.add(revokePluginAuthorityV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        payer: marketplace,
        pluginType: PluginType.FreezeDelegate,
    }));

    const tx = await sellBuilder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('list an asset with collection royalties', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);
    const marketplace = generateSigner(umi);

    let builder = new TransactionBuilder();
    builder = builder.add(createCollectionV1(umi, {
        collection: collectionAddress,
        name: "Test",
        uri: "www.test.com",
    }));
    builder = builder.add(createV1(umi, {
        asset: assetAddress,
        collection: collectionAddress.publicKey,
        name: "Test",
        uri: "www.test.com",
        plugins: [pluginAuthorityPair({
            type: "Royalties",
            data: {
                basisPoints: 500,
                creators: [{ address: umi.identity.publicKey, percentage: 100 }],
                ruleSet: {
                    __kind: "None"
                }
            },
        })]
    }));

    await builder.sendAndConfirm(umi);

    let listBuilder = addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: true },
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    });

    listBuilder = listBuilder.add(addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "TransferDelegate",
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    }));

    const tx = await listBuilder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});

test('sell an asset with collection royalties', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const collectionAddress = generateSigner(umi);
    const assetAddress = generateSigner(umi);
    const newOwner = generateSigner(umi);
    const marketplace = generateSigner(umi);

    let builder = new TransactionBuilder();
    builder = builder.add(createCollectionV1(umi, {
        collection: collectionAddress,
        name: "Test",
        uri: "www.test.com",
    }));
    builder = builder.add(createV1(umi, {
        asset: assetAddress,
        collection: collectionAddress.publicKey,
        name: "Test",
        uri: "www.test.com",
        plugins: [pluginAuthorityPair({
            type: "Royalties",
            data: {
                basisPoints: 500,
                creators: [{ address: umi.identity.publicKey, percentage: 100 }],
                ruleSet: {
                    __kind: "None"
                }
            },
        })]
    }));

    await builder.sendAndConfirm(umi);

    let listBuilder = addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: true },
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    });

    listBuilder = listBuilder.add(addPluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        plugin: createPlugin({
            type: "TransferDelegate",
        }),
        initAuthority: addressPluginAuthority(marketplace.publicKey),
    }));

    await listBuilder.sendAndConfirm(umi);

    let sellBuilder = transferV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        newOwner: newOwner.publicKey,
        payer: marketplace,
    });

    sellBuilder = updatePluginV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        payer: marketplace,
        plugin: createPlugin({
            type: "FreezeDelegate",
            data: { frozen: false },
        }),
    });

    sellBuilder = sellBuilder.add(revokePluginAuthorityV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        payer: marketplace,
        pluginType: PluginType.TransferDelegate,
    }));

    sellBuilder = sellBuilder.add(revokePluginAuthorityV1(umi, {
        asset: assetAddress.publicKey,
        collection: collectionAddress.publicKey,
        payer: marketplace,
        pluginType: PluginType.FreezeDelegate,
    }));

    const tx = await sellBuilder.sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const cuResult = {
        name: `CU: ${t.title}`,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(cuResult);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});