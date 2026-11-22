// The VS Code side of IPL-V. The work is done by the iplv program, which the
// extension carries compiled to WebAssembly and runs with the editor's own
// Node: `iplv lsp` is the language server, and the commands run the same
// program on the deck being edited. What is done here is the keypunch: Tab
// goes to the next field of the card.
"use strict";

const vscode = require("vscode");
const path = require("path");
const { execFile } = require("child_process");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;
let output;

// Where the fields of a card begin, counting from 0: comment, TYPE, NAME,
// SIGN, P and Q, SYMB, LINK, and the comment at the right.
const FIELDS = [0, 40, 42, 47, 48, 50, 56, 62];
// Tab skips SIGN, which is nearly always blank.
const STOPS = [0, 40, 42, 48, 50, 56, 62];

const custom = () => (vscode.workspace.getConfiguration("iplv").get("path") || "").trim();

async function startServer(context) {
  const program = custom();
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = program ? { command: program, args: ["lsp"] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("iplv", "IPL-V", serverOptions, { documentSelector: [{ language: "iplv" }] });
  try {
    await client.start();
  } catch (error) {
    client = undefined;
    vscode.window.showWarningMessage(`IPL-V: the language server did not start (${error.message}). Highlighting and the card fields still work.`);
  }
}

async function restartServer(context) {
  const old = client;
  client = undefined;
  if (old) await old.stop().catch(() => {});
  await startServer(context);
}

/** Runs `iplv COMMAND FILE OPTIONS` and resolves with everything it printed. */
function iplv(context, file, command, options) {
  const program = custom();
  const [exe, args, env] = program
    ? [program, [command, file, ...options], process.env]
    : [process.execPath, [context.asAbsolutePath(path.join("server", "run.js")), file, command, ...options], { ...process.env, ELECTRON_RUN_AS_NODE: "1" }];
  return new Promise((resolve) => {
    const child = execFile(exe, args, { env, timeout: 60000, maxBuffer: 64 * 1024 * 1024 }, (error, stdout, stderr) => {
      const missing = error && error.code === "ENOENT" ? `could not run "${exe}": check the iplv.path setting\n` : "";
      resolve(stdout + stderr + missing);
    });
    child.stdin.end();
  });
}

/** Runs the deck in the editor and shows what it prints. */
async function run(context, options) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "iplv") {
    vscode.window.showInformationMessage("Open an IPL-V deck to run it.");
    return undefined;
  }
  const document = editor.document;
  if (document.isUntitled) {
    vscode.window.showInformationMessage("Save the deck first.");
    return undefined;
  }
  if (document.isDirty) await document.save();
  const file = document.uri.fsPath;
  output.clear();
  output.show(true);
  output.appendLine(`iplv ${["run", path.basename(file), ...options].join(" ")}`);
  const printed = await iplv(context, file, "run", options);
  output.append(printed);
  return printed;
}

/** Moves each cursor to the next field of its card, or the one before. */
async function field(direction) {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  const targets = editor.selections.map((selection) => {
    const { line, character } = selection.active;
    const stop = direction > 0 ? STOPS.find((s) => s > character) : [...STOPS].reverse().find((s) => s < character);
    return { line, column: stop ?? (direction > 0 ? character : 0) };
  });
  // A card too short to have the field is filled out with blanks.
  await editor.edit((edit) => {
    for (const { line, column } of targets) {
      const length = editor.document.lineAt(line).text.length;
      if (column > length) edit.insert(new vscode.Position(line, length), " ".repeat(column - length));
    }
  }, { undoStopBefore: false, undoStopAfter: false });
  editor.selections = targets.map(({ line, column }) => {
    const at = new vscode.Position(line, column);
    return new vscode.Selection(at, at);
  });
}

/** Shows in the status bar which column and field the cursor is in. */
function where(item, editor) {
  if (!editor || editor.document.languageId !== "iplv") return item.hide();
  const column = editor.selection.active.character;
  const names = ["comment", "TYPE", "NAME", "SIGN", "P Q", "SYMB", "LINK", "comment"];
  let index = FIELDS.findLastIndex((f) => f <= column);
  // Columns 42 and 56 lie between fields.
  const between = column === 41 || column === 55 || column >= 80;
  item.text = `col ${column + 1}  ${between ? "—" : names[index]}`;
  item.show();
}

async function activate(context) {
  output = vscode.window.createOutputChannel("IPL-V");
  const column = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 101);
  column.tooltip = "The column of the card, from 1, and the field it is in";
  context.subscriptions.push(
    output,
    column,
    vscode.commands.registerCommand("iplv.run", () => run(context, [])),
    vscode.commands.registerCommand("iplv.runTrace", () => run(context, ["--trace"])),
    vscode.commands.registerCommand("iplv.listing", () => run(context, ["--list"])),
    vscode.commands.registerCommand("iplv.nextField", () => field(1)),
    vscode.commands.registerCommand("iplv.previousField", () => field(-1)),
    vscode.commands.registerCommand("iplv.restart", () => restartServer(context)),
    vscode.window.onDidChangeTextEditorSelection((event) => where(column, event.textEditor)),
    vscode.window.onDidChangeActiveTextEditor((editor) => where(column, editor)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("iplv.path")) restartServer(context);
    }),
  );
  where(column, vscode.window.activeTextEditor);
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
