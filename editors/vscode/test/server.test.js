// Checks the two scripts the extension starts, with plain Node:
// the language server and the runner.   node test/server.test.js
const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");
const { spawn, spawnSync } = require("node:child_process");

const server = path.join(__dirname, "../server");
const examples = path.join(__dirname, "../../../examples");
const frame = (message) => {
  const body = Buffer.from(JSON.stringify({ jsonrpc: "2.0", ...message }));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};

(async () => {
  // The runner: a deck, a command and its options.
  const node = (...args) => spawnSync(process.execPath, [path.join(server, "run.js"), ...args], { input: "", encoding: "utf8" });
  for (const file of fs.readdirSync(examples).filter((f) => f.endsWith(".iplv") && !fs.existsSync(path.join(examples, f.replace(".iplv", ".in"))))) {
    const ran = node(path.join(examples, file), "run");
    assert.strictEqual(ran.stdout, fs.readFileSync(path.join(examples, file.replace(".iplv", ".out")), "utf8"), file + ran.stderr);
    assert.strictEqual(ran.status, 0);
  }
  const listed = node(path.join(examples, "hello.iplv"), "run", "--list");
  assert.ok(listed.stdout.includes("NUMBER OF CELLS ON AVAILABLE SPACE=") && listed.stdout.endsWith("HELLO WORLD\n"), listed.stdout);
  const traced = node(path.join(examples, "hello.iplv"), "run", "--trace");
  assert.ok(traced.stdout.split("\n").length > 8 && traced.stdout.includes("J157"), traced.stdout);
  assert.strictEqual(node(path.join(examples, "none.iplv"), "run").status, 2);

  // The server: a conversation over its standard input and output, with
  // the answers read as they arrive, as an editor does.
  const child = spawn(process.execPath, [path.join(server, "lsp.js")]);
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const messages = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const split = buffer.indexOf("\r\n\r\n");
      if (split < 0) return;
      const length = Number(/Content-Length: (\d+)/.exec(buffer.subarray(0, split).toString())[1]);
      if (buffer.length < split + 4 + length) return;
      const message = JSON.parse(buffer.subarray(split + 4, split + 4 + length).toString());
      buffer = buffer.subarray(split + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else messages.push(message);
    }
  });
  const read = () => new Promise((resolve) => (messages.length ? resolve(messages.shift()) : waiting.push(resolve)));
  const uri = "file:///f1.iplv";
  const text = fs.readFileSync(path.join(examples, "manual-f1.iplv"), "utf8");

  child.stdin.write(frame({ id: 1, method: "initialize", params: { capabilities: {} } }));
  const hello = await read();
  assert.strictEqual(hello.result.serverInfo.name, "iplv");
  assert.deepStrictEqual(hello.result.capabilities.semanticTokensProvider.legend.tokenTypes.slice(0, 3), ["comment", "keyword", "function"]);

  child.stdin.write(frame({ method: "textDocument/didOpen", params: { textDocument: { uri, text: text.replace("J66", "J67").replace("709-2", "709-7") } } }));
  const published = await read();
  assert.deepStrictEqual(published.params.diagnostics.map((d) => d.message), ["the local symbol 9-7 is used but never defined in this list structure"]);

  child.stdin.write(frame({ id: 2, method: "textDocument/hover", params: { textDocument: { uri }, position: { line: 28, character: 52 } } }));
  const hover = await read();
  assert.ok(hover.result.contents.value.includes("LOCATE the next symbol after cell (0)"));

  child.stdin.write(frame({ id: 3, method: "shutdown" }));
  await read();
  child.stdin.write(frame({ method: "exit" }));
  const code = await new Promise((resolve) => child.on("exit", resolve));
  assert.strictEqual(code, 0);
  console.log("server: the runner and a live language-server conversation verified under Node");
})().catch((error) => {
  console.error(error.message || error);
  process.exit(1);
});
