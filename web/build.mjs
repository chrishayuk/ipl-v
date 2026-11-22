// Builds the files the page needs:  node web/build.mjs
//   web/iplv.wasm      the IPL computer, compiled for WebAssembly
//   web/examples.json  the example decks, with the cards they read
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url).pathname;
execSync("cargo build --release --lib --target wasm32-unknown-unknown", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-unknown-unknown/release/iplv.wasm`, `${root}web/iplv.wasm`);

// Shown in this order; any others follow alphabetically.
const first = ["manual-f1", "hello", "pushdown", "factorial", "lists", "describe", "structure",
  "generator", "hanoi", "sentences", "sum"];

const examples = readdirSync(`${root}examples`).filter((f) => f.endsWith(".iplv")).map((f) => {
  const name = f.slice(0, -5);
  const input = `${root}examples/${name}.in`;
  const deck = readFileSync(`${root}examples/${f}`, "utf8");
  return {
    name,
    title: deck.slice(0, 40).trim(),
    deck,
    input: existsSync(input) ? readFileSync(input, "utf8") : "",
  };
});
// The programs of the manual's Part One, from manual/*.md: each ```iplv
// block followed by an ```output block, which the test compares.
const fence = /```iplv\n([\s\S]*?)```\n\n(?:```input\n([\s\S]*?)```\n\n)?```output\n([\s\S]*?)```/g;
const manual = [];
for (const file of readdirSync(`${root}manual`).filter((f) => /^\d.*\.md$/.test(f)).sort()) {
  const chapter = readFileSync(`${root}manual/${file}`, "utf8");
  let n = 0;
  for (const [, deck, input, expected] of chapter.matchAll(fence)) {
    n++;
    manual.push({
      name: `manual-${file.slice(0, 2)}-${n}`,
      title: deck.slice(0, 40).trim(),
      deck,
      input: input || "",
      expected,
    });
  }
}

examples.sort((a, b) => {
  const [i, j] = [first.indexOf(a.name), first.indexOf(b.name)];
  if (i !== -1 || j !== -1) return (i === -1 ? 99 : i) - (j === -1 ? 99 : j);
  return a.name.localeCompare(b.name);
});
writeFileSync(`${root}web/examples.json`, JSON.stringify([...examples, ...manual]));
console.log(`web/iplv.wasm and web/examples.json (${examples.length} examples, ${manual.length} from the manual) written`);
