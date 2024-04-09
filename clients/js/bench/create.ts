import { generateSigner } from "@metaplex-foundation/umi";
import test from "ava";
import { existsSync, readFileSync, writeFileSync } from "fs";
import { createV1 } from "../src";
import { createUmi } from "./_setup";

test('create a new, empty asset', async (t) => {
    // Given an Umi instance and a new signer.
    const umi = await createUmi();
    const assetAddress = generateSigner(umi);

    const tx = await createV1(umi, {
        asset: assetAddress,
        name: "Test",
        uri: "www.test.com",
    }).sendAndConfirm(umi);

    const compute = Number((await umi.rpc.getTransaction(tx.signature))?.meta.computeUnitsConsumed);

    const result = {
        name: t.title,
        unit: "Compute Units",
        value: compute,
    }

    // Read the results array from output.json
    let output = [];
    if (existsSync("./output.json")) {
        output = JSON.parse(readFileSync("./output.json", 'utf-8'));
    }

    // Push the result to the array
    output.push(result);
    // Write the array to output.json
    writeFileSync("./output.json", JSON.stringify(output, null, 2));

    t.pass();
});