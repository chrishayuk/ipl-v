// Runs inside VS Code, started by run-vscode.js.
const assert = require("node:assert");
const path = require("node:path");
const vscode = require("vscode");

const until = async (what, test) => {
  for (let i = 0; i < 100; i++) {
    const value = await test();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`timed out waiting for ${what}`);
};

exports.run = async () => {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const uri = vscode.Uri.file(path.join(folder, "manual-f1.iplv"));
  const document = await vscode.workspace.openTextDocument(uri);
  assert.strictEqual(document.languageId, "iplv");
  const editor = await vscode.window.showTextDocument(document);
  const original = document.getText();

  // Resting on J60 says what the card does; the server needs a moment to start.
  const shown = await until("hover", async () => {
    const hovers = await vscode.commands.executeCommand("vscode.executeHoverProvider", uri, new vscode.Position(28, 52));
    const text = hovers.flatMap((h) => h.contents).map((c) => c.value ?? c).join("\n");
    return text.includes("J60") ? text : null;
  });
  assert.ok(shown.includes("LOCATE the next symbol after cell (0)"), shown);
  assert.deepStrictEqual(vscode.languages.getDiagnostics(uri), []);

  // 9-2 in `70 9-2` leads to the card named 9-2.
  const places = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", uri, new vscode.Position(29, 51));
  assert.strictEqual((places[0].range ?? places[0].targetRange).start.line, 36);

  const outline = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", uri);
  assert.deepStrictEqual(outline.map((s) => s.name), ["E1", "F1", "K1"]);

  // The basic processes come from the server and the snippets from the extension.
  const labels = await until("completions", async () => {
    const offered = await vscode.commands.executeCommand("vscode.executeCompletionItemProvider", uri, new vscode.Position(15, 20));
    const found = offered.items.map((i) => i.label.label ?? i.label);
    return found.includes("J66") && found.includes("start") ? found : null;
  });
  assert.ok(labels.includes("F1"));

  const colours = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", uri);
  assert.ok(colours.data.length >= 5 * 40, "semantic tokens");

  // A process that does not exist becomes an error on its card.
  await editor.edit((edit) => edit.replace(new vscode.Range(32, 50, 32, 52), "J139"));
  const broken = await until("the error", () => {
    const found = vscode.languages.getDiagnostics(uri);
    return found.length === 1 && found[0].severity === vscode.DiagnosticSeverity.Error ? found : null;
  });
  assert.strictEqual(broken[0].range.start.line, 32);
  assert.strictEqual(broken[0].message, "there is no basic process J139");
  await vscode.commands.executeCommand("workbench.action.files.revert");

  // Formatting leaves a deck as a keypunch would: upper case, symbols at
  // the left of their fields.
  await editor.edit((edit) => edit.replace(new vscode.Range(0, 0, document.lineCount, 0), original.toLowerCase()));
  await vscode.commands.executeCommand("editor.action.formatDocument");
  await until("the formatted text", () => editor.document.getText() === original);
  await vscode.commands.executeCommand("workbench.action.files.revert");

  // Running the deck, with the program the extension carries.
  const printed = await vscode.commands.executeCommand("iplv.run");
  assert.ok(printed.endsWith("         B1\n         C1\n         A1\n"), printed);
  const hanoi = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(folder, "hanoi.iplv")));
  await vscode.window.showTextDocument(hanoi);
  assert.strictEqual((await vscode.commands.executeCommand("iplv.run")).trim().split("\n").length, 7);
  const listing = await vscode.commands.executeCommand("iplv.listing");
  assert.ok(listing.includes("END OF LOADING. PROGRAM STARTS AT P0"), listing);
  const traced = await vscode.commands.executeCommand("iplv.runTrace");
  assert.ok(traced.split("\n").length > 200, "a trace of every instruction");

  // The keypunch: Tab goes from field to field of an empty card, filling
  // it out with blanks, and Shift+Tab comes back.
  const blank = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(folder, "new.iplv")));
  const card = await vscode.window.showTextDocument(blank);
  const type = async (text) => {
    await card.edit((edit) => edit.insert(card.selection.active, text));
    const at = card.selection.active.translate(0, 0);
    card.selection = new vscode.Selection(at, at);
  };
  const tab = () => vscode.commands.executeCommand("iplv.nextField");
  await type("PRINT THE LINE.");
  await tab(); // TYPE
  assert.strictEqual(card.selection.active.character, 40);
  await tab(); // NAME
  await type("P0");
  await tab(); // P and Q
  assert.strictEqual(card.selection.active.character, 48);
  await tab(); // SYMB
  await type("J155");
  await tab(); // LINK
  await type("0");
  assert.strictEqual(blank.getText(), `${"PRINT THE LINE.".padEnd(40)}  P0      J155  0`);
  await vscode.commands.executeCommand("iplv.previousField");
  assert.strictEqual(card.selection.active.character, 56);
  await vscode.commands.executeCommand("iplv.previousField");
  assert.strictEqual(card.selection.active.character, 50);

  console.log("extension: hover, definition, outline, completion, colours, diagnostics, formatting, the run commands and the card fields verified in VS Code");
};
