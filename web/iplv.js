// The IPL computer in iplv.wasm, as an object. Used by the page and by
// test.mjs. See src/web.rs for what each call does.

const WORD = 12; // bytes to a cell: P at 0, Q at 1, SYMB at 4, LINK at 8

export const H0 = 24574, H1 = 24575, H2 = 24576, H3 = 24577, H5 = 24579, W0 = 24587;
export const SYSTEM = [24574, 24900];

export class Iplv {
  // `source` is a WebAssembly.Module, or the bytes of one.
  static async create(source) {
    const module = source instanceof WebAssembly.Module ? source : await WebAssembly.compile(source);
    const instance = await WebAssembly.instantiate(module, {});
    return new Iplv(instance.exports);
  }

  constructor(exports) {
    this.x = exports;
    this.encoder = new TextEncoder();
    this.decoder = new TextDecoder();
  }

  #put(...texts) {
    const parts = texts.map((t) => this.encoder.encode(t));
    const total = parts.reduce((n, p) => n + p.length, 0);
    const at = this.x.iplv_buf(total);
    const bytes = new Uint8Array(this.x.memory.buffer, at, total);
    let offset = 0;
    for (const p of parts) {
      bytes.set(p, offset);
      offset += p.length;
    }
    return parts.map((p) => p.length);
  }

  #text(length) {
    return this.decoder.decode(new Uint8Array(this.x.memory.buffer, this.x.iplv_result(), length));
  }

  // Load a deck, with the cards the program will read. True if it loaded.
  load(deck, input = "", cells = 22000) {
    if (input && !deck.endsWith("\n")) deck += "\n";
    const [d, i] = this.#put(deck, input);
    return this.x.iplv_load(d, i, cells) === 1;
  }

  // Interpret up to n instructions. mode: 0 that many, 1 over, 2 out.
  // Returns "done", "halted", "error" or "breakpoint".
  advance(n = 1, mode = 0) {
    return ["done", "halted", "error", "breakpoint"][this.x.iplv_advance(n, mode)];
  }

  state() { return JSON.parse(this.#text(this.x.iplv_state())); }
  lines() { return JSON.parse(this.#text(this.x.iplv_lines())); }
  structure(root, limit = 300) { return JSON.parse(this.#text(this.x.iplv_structure(root, limit))); }
  describe(a) { return this.#text(this.x.iplv_describe(a)); }
  output() { return this.#text(this.x.iplv_take(0)); }
  traced() { return this.#text(this.x.iplv_take(1)); }
  warnings() { return this.#text(this.x.iplv_take(2)); }
  setBreak(a, on) { this.x.iplv_break(a, on ? 1 : 0); }
  trace(mode) { this.x.iplv_trace(mode); }

  // The address of a symbol as written (L1, 9-1350, 1350), or -1.
  resolve(text) {
    const [n] = this.#put(text);
    return this.x.iplv_resolve(n);
  }

  // Storage as it stands. The view is good until the next call into the
  // module, which may move it.
  words() {
    const length = this.x.iplv_mem_len();
    const view = new DataView(this.x.memory.buffer, this.x.iplv_mem(), length * WORD);
    return {
      length,
      bytes: new Uint8Array(this.x.memory.buffer, this.x.iplv_mem(), length * WORD),
      p: (a) => view.getUint8(a * WORD),
      q: (a) => view.getUint8(a * WORD + 1),
      symb: (a) => view.getUint32(a * WORD + 4, true),
      link: (a) => view.getUint32(a * WORD + 8, true),
    };
  }
}
