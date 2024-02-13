const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instanciate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "mpl_asset_program.json")]);

// Update programs.
kinobi.update(
  new k.UpdateProgramsVisitor({
    mplAssetProgram: { name: "mplAsset" },
  })
);

// // Update accounts.
// kinobi.update(
//   new k.UpdateAccountsVisitor({
//     myPdaAccount: {
//       seeds: [
//         k.stringConstantSeed("myPdaAccount"),
//         k.programSeed(),
//         k.publicKeySeed("authority", "The address of the authority"),
//         k.stringSeed("name", "The name of the account"),
//       ],
//     },
//   })
// );

// Update instructions.
kinobi.update(
  new k.UpdateInstructionsVisitor({
    // create: {
    //   bytesCreatedOnChain: k.bytesFromAccount("assetAccount"),
    // },
  })
);

// Set ShankAccount discriminator.
const iface = (name) => ({ field: "interface", value: k.vEnum("Interface", name) });
kinobi.update(
  new k.SetAccountDiscriminatorFromFieldVisitor({
    nonFungible: iface("NonFungible"),
    // myPdaAccount: key("MyPdaAccount"),
  })
);

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(new k.RenderJavaScriptVisitor(jsDir, { prettier }));

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  new k.RenderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);
