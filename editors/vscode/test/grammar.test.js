// Checks the highlighting grammar by tokenising real cards with the same
// engine VS Code uses:  node test/grammar.test.js
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const oniguruma = require("vscode-oniguruma");
const { Registry, parseRawGrammar } = require("vscode-textmate");

const root = path.join(__dirname, "..");
const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm")).buffer;

(async () => {
  await oniguruma.loadWASM(wasm);
  const registry = new Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
      createOnigString: (text) => new oniguruma.OnigString(text),
    }),
    loadGrammar: async () => {
      const file = path.join(root, "syntaxes/iplv.tmLanguage.json");
      return parseRawGrammar(fs.readFileSync(file, "utf8"), file);
    },
  });
  const grammar = await registry.loadGrammar("source.iplv");

  // The scope of each piece of a card that is not blank, without the
  // `source.iplv` every piece has.
  const scopes = (line) =>
    grammar.tokenizeLine(line, null).tokens
      .map((t) => [line.slice(t.startIndex, t.endIndex).trim(), t.scopes.slice(1).join(" ").replace(/\.iplv/g, "")])
      .filter(([text]) => text);
  const expect = (line, want) => assert.deepStrictEqual(scopes(line), want, line);
  const deck = fs.readFileSync(path.join(root, "../../examples/manual-f1.iplv"), "utf8").split("\n");

  expect(deck[0], [["EXAMPLE RUN OF F1, FROM THE MANUAL", "comment.line"], ["9", "keyword.control.type"]]);
  expect(deck[1], [["IPL-V MANUAL, PART ONE, SECTION 2:", "comment.line"], ["1", "keyword.control.type"]]);
  expect(deck[3], [["2", "keyword.control.type"], ["A", "entity.name.function"], ["10", "constant.numeric.symbol"]]);
  expect(deck[9], [["5", "keyword.control.type"], ["00", "constant.numeric.prefix"]]);
  expect(deck[15], [["E1", "entity.name.function"], ["13", "constant.numeric.prefix"], ["K1", "variable.other.regional"]]);
  expect(deck[28], [
    ["LOCATE NEXT CELL OF INPUT LIST.", "comment.line"],
    ["9-1", "variable.other.local"],
    ["J60", "support.function.process"],
  ]);
  expect(deck[29], [["GO TO 9-2 IF NO NEXT CELL.", "comment.line"], ["70", "constant.numeric.prefix"], ["9-2", "variable.other.local"]]);
  expect(deck[30], [["INPUT THE SYMBOL IN THE CELL.", "comment.line"], ["12", "constant.numeric.prefix"], ["H0", "support.variable.system"]]);
  expect(deck[33], [
    ["ADD THE SYMBOL TO THE OUTPUT LIST", "comment.line"],
    ["J66", "support.function.process"],
    ["9-1", "variable.other.local"],
  ]);
  expect(deck[37], [
    ["IN H0 AND RESTORE W0 BEFORE", "comment.line"],
    ["30", "constant.numeric.prefix"],
    ["W0", "support.variable.system"],
    ["0", "constant.numeric.symbol"],
  ]);
  expect(deck[47], [["5", "keyword.control.type"], ["E1", "variable.other.regional"]]);
  // A data term in a routine, marked with a sign; and identification at the right.
  expect(`${" ".repeat(40)}  K1   +01     12         K001D000`, [
    ["K1", "entity.name.function"],
    ["+", "keyword.operator.sign"],
    ["01", "storage.type.data"],
    ["12", "constant.other.data"],
    ["K001D000", "comment.line.identification"],
  ]);
  expect("A SHORT CARD", [["A SHORT CARD", "comment.line"]]);

  // Every card of every example is taken by some rule.
  const examples = path.join(root, "../../examples");
  let cards = 0;
  for (const file of fs.readdirSync(examples).filter((f) => f.endsWith(".iplv"))) {
    for (const line of fs.readFileSync(path.join(examples, file), "utf8").split("\n")) {
      if (!line.trim()) continue;
      cards++;
      const plain = grammar.tokenizeLine(line, null).tokens.filter((t) => t.scopes.length === 1 && line.slice(t.startIndex, t.endIndex).trim());
      assert.deepStrictEqual(plain, [], `${file}: ${line}`);
    }
  }
  console.log(`grammar: ${cards} cards of the examples tokenised with the engine VS Code uses`);
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
