const path = require("path");
const { generateIdl } = require("@metaplex-foundation/shank-js");

const idlDir = path.join(__dirname, "..", "idls");
const binaryInstallDir = path.join(__dirname, "..", ".crates");
const programDir = path.join(__dirname, "..", "programs");

generateIdl({
  generator: "shank",
  programName: "mpl_core",
  programId: "CoREzp6dAdLVRKf3EM5tWrsXM2jQwRFeu5uhzsAyjYXL",
  idlDir,
  idlName: "mpl_core",
  binaryInstallDir,
  programDir: path.join(programDir, "mpl-core"),
});
