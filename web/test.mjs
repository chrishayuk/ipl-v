// Runs every example through the WebAssembly build, an instruction at a
// time and all at once, and checks what the page relies on.
//   node web/test.mjs
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { Iplv, H0, H1, H2 } from "./iplv.js";

const root = new URL("..", import.meta.url).pathname;
const module = new WebAssembly.Module(readFileSync(root + "web/iplv.wasm"));

let checked = 0;
let failed = 0;
const fail = (what) => {
  failed++;
  console.log(`FAIL ${what}`);
};
const check = (ok, what) => {
  checked++;
  if (!ok) fail(what);
};

const examples = JSON.parse(readFileSync(root + "web/examples.json", "utf8"));
const files = readdirSync(root + "examples").filter((f) => f.endsWith(".iplv"));
const own = examples.filter((e) => e.expected === undefined);
check(own.length === files.length, `examples.json lists ${own.length} decks of ${files.length}`);
check(examples.length - own.length >= 15, "the manual's programs are in examples.json");

for (const example of examples) {
  const expected = example.expected ?? readFileSync(`${root}examples/${example.name}.out`, "utf8");
  // All at once.
  const fast = await Iplv.create(module);
  check(fast.load(example.deck, example.input), `${example.name} loads`);
  // The deck tests run with a limit of a million cycles, and so does this.
  const why = fast.advance(1_000_000);
  const endless = expected.endsWith("! stopped after 1000000 cycles\n");
  check(why === (endless ? "done" : "halted"), `${example.name} halts (${why})`);
  const printed = fast.traced() + fast.output() + (endless ? "! stopped after 1000000 cycles\n" : "");
  check(printed === expected, `${example.name} prints what it should`);
  if (endless) continue;
  // An instruction at a time, as the Step button does; the same output and
  // the same number of cycles.
  const slow = await Iplv.create(module);
  slow.load(example.deck, example.input);
  let out = "";
  let steps = 0;
  while (slow.advance(1) === "done" && steps++ < 1_000_000) out += slow.traced() + slow.output();
  out += slow.traced() + slow.output();
  check(out === expected, `${example.name} prints its .out when stepped`);
  check(slow.state().cycles === fast.state().cycles, `${example.name} takes the same cycles stepped`);
}

const deck = (name) => examples.find((e) => e.name === name).deck;

// The state the panels show, at the start of the manual's program.
const m = await Iplv.create(module);
m.load(deck("manual-f1"));
let s = m.state();
check(s.cia.place === "E1" && s.cia.p === 1 && s.cia.q === 3 && s.cia.symb === "K1", "the first instruction is 13 K1 at E1");
check(s.level === 1 && s.h5 === "+" && s.h3 === 4 && s.free === 21858, "level 1, H5 +, H3 4, 21858 cells available");
check(s.regions.some((r) => r.ch === "K" && r.origin === 114 && r.size === 10), "region K is cells 114 to 123");
const lines = m.lines();
check(lines.some(([line, a]) => a === 95 && line === s.cia.line), "E1's card is the line of the current instruction");

// Step, over and out.
m.advance(1);
s = m.state();
check(s.cia.symb === "F1" && s.h0.items[0].name === "K1", "after one step K1 is in H0 and the instruction executes F1");
m.advance(1);
check(m.state().level === 2 && m.state().calls.length === 2, "a step into F1 descends a level");
check(m.advance(1000, 2) === "done" && m.state().cia.symb === "J151", "out of F1, at the instruction after it");
const before = m.state();
check(m.advance(1000, 1) === "halted" || m.state().level === before.level, "over does not descend");

// A breakpoint on the loop in F1 stops there once for each symbol on K1.
const b = await Iplv.create(module);
b.load(deck("manual-f1"));
b.setBreak(128, true);
let stops = 0;
while (b.advance(100_000) === "breakpoint") stops++;
check(stops === 7, `the breakpoint at cell 128 is met 7 times (${stops})`);

// Storage read where it lies agrees with the machine's own account.
const l = await Iplv.create(module);
l.load(deck("lists"));
l.advance(40);
const words = l.words();
let free = 0;
for (let c = words.link(H2); c !== 0 && free < words.length; c = words.link(c)) free++;
check(free === l.state().free, `H2 has ${free} cells on it, as the machine says`);
let depth = 0;
for (let c = H1; c !== 0 && depth < 1000; c = words.link(c)) depth++;
check(depth === l.state().level, "the push-down list of H1 is as deep as the level");
check(words.symb(H0) === l.state().h0.items[0].a, "the symbol in H0 is (0)");

// List structures, for the drawing.
const x = await Iplv.create(module);
x.load(deck("structure"));
const x0 = x.resolve("X0");
check(x0 > 0 && x.resolve("x0") === x0 && x.resolve("Q7") === -1, "symbols resolve to addresses");
const tree = x.structure(x0);
check(tree.lists.length === 4 && tree.cells === 14, "X0 is four lists of fourteen cells");
check(tree.lists[0].name === "X0" && tree.lists[0].cells[1].kind === "local", "its first list cell names a sublist");
check(tree.lists[3].cells.map((c) => c.name).join(" ") === "0 X3 -0 X4" && tree.lists[3].end === "0", "the innermost is X3 - X4");
check(x.structure(x0, 5).lists.at(-1).end === "more", "a drawing can be cut short");
const d = await Iplv.create(module);
d.load(deck("describe"));
const g1 = d.structure(d.resolve("G1"));
check(g1.lists[1].cells.some((c) => c.data === "GRASS"), "a cell that names a data term carries its text");

// Errors come back as words, not as a trap.
const e = await Iplv.create(module);
check(!e.load("                                        5       00\n                                          P0      Q9    0\n                                        5         P0\n") === false, "a deck that executes an undefined symbol still loads");
check(e.advance(10) === "error" && e.state().error.includes("never defined"), "and stops with an error that says so");
const none = await Iplv.create(module);
check(none.load("nothing\n") === false && none.state().error.includes("header card"), "a deck with no header says so");

console.log(`${checked} checks through WebAssembly, ${failed} failed`);
process.exit(failed ? 1 : 0);
