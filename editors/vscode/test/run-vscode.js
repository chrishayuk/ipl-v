// Runs the extension inside a real VS Code and drives it:  node test/run-vscode.js
// The first run downloads a copy of VS Code into .vscode-test/, unless
// VSCODE_EXECUTABLE names one to use.
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { runTests } = require("@vscode/test-electron");

(async () => {
  const root = path.join(__dirname, "..");
  if (!fs.existsSync(path.join(root, "dist/extension.js"))) throw new Error("build the extension first: npm run build");
  // No settings: the extension must work with the program it carries.
  const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "iplv-vscode-"));
  for (const name of ["manual-f1", "hanoi"]) {
    fs.copyFileSync(path.join(root, "../../examples", `${name}.iplv`), path.join(workspace, `${name}.iplv`));
  }
  fs.writeFileSync(path.join(workspace, "new.iplv"), "");
  await runTests({
    extensionDevelopmentPath: root,
    extensionTestsPath: path.join(__dirname, "suite.js"),
    launchArgs: [workspace, "--disable-extensions", "--disable-workspace-trust", `--user-data-dir=${fs.mkdtempSync(path.join(os.tmpdir(), "iplv-vscode-data-"))}`],
    ...(process.env.VSCODE_EXECUTABLE ? { vscodeExecutablePath: process.env.VSCODE_EXECUTABLE } : {}),
  });
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
