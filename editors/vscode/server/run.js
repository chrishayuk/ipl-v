// Runs one iplv command on a deck:  node run.js FILE run --trace
// The program compiled for WASI has no file system here, so the file is read
// by this script and handed over on standard input, where `-` stands for it.
// The cards behind the start card are in the same file, as in a card reader.
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const [file, command, ...rest] = process.argv.slice(2);
if (!file || !command) {
  console.error("usage: run.js FILE COMMAND [OPTIONS]");
  process.exit(2);
}
let deck;
try {
  deck = fs.readFileSync(file);
} catch (error) {
  console.error(`iplv: ${file}: ${error.message}`);
  process.exit(2);
}
process.exitCode = run(path.join(__dirname, "iplv.wasm"), { args: [command, "-", ...rest], prefix: deck });
