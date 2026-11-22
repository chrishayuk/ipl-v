// Drives the page in headless Chrome over the DevTools protocol: presses
// the buttons, sets a stop, opens a deck from a file, and reads the page.
//   node web/browser-test.mjs            (needs Chrome and python3)
import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";

const root = new URL("..", import.meta.url).pathname;
const chromePath = process.env.CHROME || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const [port, debug] = [8771, 9333];
const server = spawn("python3", ["-m", "http.server", "-d", root + "web", String(port)], { stdio: "ignore" });
const chrome = spawn(chromePath, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${debug}`,
  `--user-data-dir=${mkdtempSync(tmpdir() + "/iplv-chrome-")}`, "--window-size=1320,1500", "about:blank"], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const quit = (code) => (server.kill(), chrome.kill(), process.exit(code));

let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try {
    target = (await (await fetch(`http://localhost:${debug}/json`)).json()).find((t) => t.type === "page");
  } catch {}
}
if (!target) (console.log("Chrome did not start"), quit(2));
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => (socket.onopen = r));
let id = 0;
const waiting = new Map();
const errors = [];
socket.onmessage = (event) => {
  const m = JSON.parse(event.data);
  if (m.id && waiting.has(m.id)) waiting.get(m.id)(m.result ?? m);
  if (m.method === "Runtime.exceptionThrown") errors.push(m.params.exceptionDetails.exception?.description ?? m.params.exceptionDetails.text);
};
const send = (method, params = {}) => new Promise((r) => (waiting.set(++id, r), socket.send(JSON.stringify({ id, method, params }))));
const js = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
  return r.result.value;
};
const open = async (query) => {
  await send("Page.navigate", { url: `http://localhost:${port}/${query}` });
  for (let i = 0; i < 100; i++) {
    await sleep(100);
    if (await js(`document.querySelector("#now")?.textContent.length > 0`)) return;
  }
  throw new Error("the page did not come up");
};

let checked = 0, failed = 0;
const check = (ok, what) => (checked++, ok || (failed++, console.log(`FAIL ${what}`)));
const text = (sel) => js(`document.querySelector(${JSON.stringify(sel)}).textContent`);
const click = (sel) => js(`document.querySelector(${JSON.stringify(sel)}).click()`);

try {
  await send("Runtime.enable");
  await send("Page.enable");
  await open("");
  check((await text("#now")).startsWith("E1"), "the first deck opens at E1");
  check((await js(`document.querySelectorAll("#listing .row").length`)) === 48, "the listing has a row for each card");
  check((await text("#listing .row.now")).includes("13K1"), "the card of the instruction is marked");

  await click("#step");
  check((await text("#now")).includes("F1"), "Step goes on to the next instruction");
  check((await text("#machine")).includes("(0)K1"), "K1 is in H0");
  await click("#step");
  check((await text("#machine")).includes("2F1"), "a step into F1 is a level down");
  await click("#out");
  check((await text("#now")).includes("J151"), "Out comes back to the routine that executed it");
  check((await js(`document.querySelectorAll("#structure .cell").length`)) === 4, "the output list is drawn: a head and three cells");
  check((await text("#structure")).includes("B1") && (await text("#structure")).includes("C1"), "with B1 and C1 on it");
  await click("#over");
  check((await text("#now")) === "Halted." && (await text("#printed")).includes("A1"), "Over the last instruction halts, with the list printed");
  check(await js(`document.querySelector("#step").disabled`), "and the buttons are put out of use");

  // A stop, set by a click beside the card of cell 128.
  await click("#load");
  await js(`[...document.querySelectorAll("#listing .row")].find((r) => r.querySelector(".addr").textContent === "128").querySelector(".bp").click()`);
  await js(`document.querySelector("#speed").value = 10`);
  await click("#run");
  await sleep(400);
  check((await text("#listing .row.now .addr")) === "128" && (await text("#run")) === "Run", "Run stops at the card that was marked");
  await click("#run");
  await sleep(400);
  check((await text("#listing .row.now .addr")) === "128" && !(await text("#now")).startsWith("Halted"), "and at it again the next time round");

  // The map: a pointer over a cell says what is in it; a click draws it.
  check((await js(`document.querySelector("#map").width`)) === 100, "the map is 100 cells to a row for a small program");
  await js(`(() => { const m = document.querySelector("#map"), b = m.getBoundingClientRect();
    const at = (a) => ({ clientX: b.left + ((a % 100) + 0.5) * b.width / 100, clientY: b.top + (Math.floor(a / 100) + 0.5) * b.height / m.height });
    m.onmousemove(at(115)); m.onclick(); })()`);
  check((await text("#where")).startsWith("cell 115   K1"), "cell 115 is K1");
  check((await js(`document.querySelectorAll("#structure .cell").length`)) === 7, "and a click on it draws the list K1");
  await js(`(() => { const i = document.querySelector("#show"); i.value = "nonsense9"; i.oninput(); })()`);
  check((await text("#structure")).includes("not a symbol in use"), "a symbol that is not in use is said to be so");

  // Another deck, another speed, to the end.
  await js(`(() => { const s = document.querySelector("#examples"); s.value = [...s.options].findIndex((o) => o.textContent.startsWith("hanoi")); s.onchange(); })()`);
  await js(`document.querySelector("#speed").value = 8`);
  await click("#run");
  for (let i = 0; i < 50 && (await text("#run")) === "Stop"; i++) await sleep(100);
  check((await text("#printed")).trim().split("\n").length === 7 && (await text("#status")).startsWith("halted after"), "hanoi runs to its seven moves");

  // Editing: a deck that is changed and loaded again.
  await click("#editing");
  await js(`(() => { const e = document.querySelector("#edit"); e.value = e.value.replace("N0    013", "N0    014"); })()`);
  check(await js(`document.querySelector("#edit").value.includes("N0    014")`), "the deck can be edited");
  await click("#load");
  await js(`document.querySelector("#speed").value = 10`);
  await click("#run");
  for (let i = 0; i < 50 && (await text("#run")) === "Stop"; i++) await sleep(100);
  check((await text("#printed")).trim().split("\n").length === 15, "four discs take fifteen moves");

  // A deck from a file, with a file of cards behind it.
  const dir = mkdtempSync(tmpdir() + "/iplv-deck-");
  const sum = JSON.parse(await js(`fetch("examples.json").then((r) => r.text())`)).find((e) => e.name === "sum");
  writeFileSync(dir + "/sum.iplv", sum.deck);
  writeFileSync(dir + "/sum.in", "    5\n   37\n");
  const doc = await send("DOM.getDocument");
  const input = await send("DOM.querySelector", { nodeId: doc.root.nodeId, selector: "#file" });
  await send("DOM.setFileInputFiles", { nodeId: input.nodeId, files: [dir + "/sum.iplv", dir + "/sum.in"] });
  await sleep(300);
  check((await text("#machine")).includes("cards to read2"), "a deck opened from a file has its cards in the reader");
  await click("#run");
  for (let i = 0; i < 50 && (await text("#run")) === "Stop"; i++) await sleep(100);
  check((await text("#printed")).trim() === "42", "and adds them up");

  // An error is shown in words.
  await click("#editing");
  await js(`(() => { const e = document.querySelector("#edit"); e.value = e.value.replace("    J180", "    Q9  "); })()`);
  await click("#load");
  await click("#step");
  await click("#step");
  check((await text("#means")).includes("never defined"), `an undefined routine stops the run and says why (${await text("#means")})`);

  // Dark: the palette follows the colour scheme.
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "light" }] });
  const light = await js(`getComputedStyle(document.body).backgroundColor`);
  await send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-color-scheme", value: "dark" }] });
  check((await js(`getComputedStyle(document.body).backgroundColor`)) !== light, "the page has a dark form");

  await open("?deck=sentences&steps=700&show=N0");
  check((await text("#status")).includes("cycles") && (await js(`document.querySelectorAll("#structure .cell").length`)) === 10, "a link opens a deck part of the way through");
  check(errors.length === 0, `no script errors (${errors.join("; ")})`);
} catch (e) {
  failed++;
  console.log(`FAIL ${e.message}`);
}
console.log(`${checked} checks in the browser, ${failed} failed`);
quit(failed ? 1 : 0);
