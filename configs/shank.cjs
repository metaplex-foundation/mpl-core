const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "mpl_asset_program",
  programId: "ASSETp3DinZKfiAyvdQG16YWWLJ2X3ZKjg9zku7n1sZD",
  idlDir,
  binaryInstallDir,
  programDir: path.join(programDir, "mpl-asset"),
});
