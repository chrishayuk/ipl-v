//! A run that can be stopped and looked at: for the page in `web/`.
//!
//! A `Session` is an IPL computer with a deck loaded, a set of breakpoints,
//! and ways of describing itself as JSON: the cells with system functions
//! (`state`), where each card was loaded (`lines`), and any list structure
//! as lists of cells (`structure`). Nothing here changes what the machine
//! does.

use crate::interp::NATIVE;
use crate::machine::*;
use crate::word::*;
use std::collections::HashSet;
use std::fmt::Write;

/// How far `advance` should go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// So many instructions.
    Steps,
    /// Until this routine's next instruction: over whatever it executes.
    Over,
    /// Until the routine that executed this one goes on.
    Out,
}

/// Why `advance` stopped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    Done = 0,
    Halted = 1,
    Error = 2,
    Breakpoint = 3,
}

pub struct Session {
    pub m: Machine,
    pub breakpoints: HashSet<u32>,
    pub error: Option<String>,
}

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 32 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o
}

impl Session {
    /// Load a deck, with the cards the program will read behind it.
    pub fn new(deck: &str, input: &str, cells: usize) -> Session {
        let mut m = Machine::new(cells);
        m.read_deck(deck);
        if !input.is_empty() {
            m.read_deck(input);
        }
        let error = match m.load() {
            Ok(Some(start)) => {
                m.start(start);
                None
            }
            Ok(None) => {
                m.halted = true;
                Some("the deck has no start card (TYPE 5 with a routine in SYMB)".to_string())
            }
            Err(e) => {
                m.halted = true;
                Some(e)
            }
        };
        Session {
            m,
            breakpoints: HashSet::new(),
            error,
        }
    }

    pub fn advance(&mut self, n: u64, mode: Mode) -> Why {
        let level = self.m.level;
        for i in 0..n {
            if self.error.is_some() {
                return Why::Error;
            }
            if self.m.halted {
                return Why::Halted;
            }
            if i > 0 {
                if self.breakpoints.contains(&self.m.mem[H1 as usize].symb) {
                    return Why::Breakpoint;
                }
                match mode {
                    Mode::Over if self.m.level <= level => return Why::Done,
                    Mode::Out if self.m.level < level => return Why::Done,
                    _ => {}
                }
            }
            if let Err(e) = self.m.step() {
                self.error = Some(self.m.post_mortem(&e));
                self.m.halted = true;
                return Why::Error;
            }
        }
        if self.m.halted {
            Why::Halted
        } else {
            Why::Done
        }
    }

    /// A symbol as typed: `L1`, `9-1350` or `1350`.
    pub fn resolve(&self, text: &str) -> Option<u32> {
        let t = text.trim().to_ascii_uppercase();
        let b = t.as_bytes();
        let first = *b.first()?;
        let digits = |s: &str| {
            s.chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u32>()
                .ok()
        };
        let a = if let Some(rest) = t.strip_prefix("9-") {
            digits(rest)?
        } else if first.is_ascii_digit() {
            t.parse().ok()?
        } else {
            self.m.lookup(first, digits(&t[1..]).unwrap_or(0))?
        };
        ((a as usize) < self.m.mem.len()).then_some(a)
    }

    fn cell(&self, a: u32) -> String {
        let m = &self.m;
        let w = m.mem[a as usize];
        let (kind, data) = match w.datum() {
            Some(d) => ("data", datum_text(d)),
            None => {
                let named = if w.symb != 0 {
                    m.datum(w.symb).map(datum_text)
                } else {
                    None
                };
                let kind = match w.q {
                    _ if w.symb == 0 => "empty",
                    Q_LOCAL if !m.is_regional(w.symb) => "local",
                    _ if m.is_regional(w.symb) => "regional",
                    _ => "internal",
                };
                (kind, named.unwrap_or_default())
            }
        };
        format!(
            "{{\"a\":{},\"p\":{},\"q\":{},\"s\":{},\"l\":{},\"name\":\"{}\",\"kind\":\"{}\",\"data\":\"{}\"}}",
            a,
            w.p,
            w.q,
            w.symb,
            w.link,
            // A local symbol is shown as the deck wrote it, where it did.
            esc(&match kind {
                "data" => String::new(),
                "local" if m.label(w.symb).starts_with("9-") => m.label(w.symb),
                _ => m.name(w.sym()),
            }),
            kind,
            esc(&data)
        )
    }

    /// A storage cell and the push-down list under it, top first.
    fn stack(&self, cell: u32, limit: usize) -> String {
        let m = &self.m;
        let mut items = Vec::new();
        let mut c = cell;
        let mut depth = 0;
        loop {
            if items.len() < limit {
                let s = m.mem[c as usize].sym();
                items.push(format!(
                    "{{\"a\":{},\"name\":\"{}\",\"text\":\"{}\"}}",
                    s.a,
                    esc(&m.name(s)),
                    esc(&m.contents(s.a))
                ));
            }
            depth += 1;
            c = m.mem[c as usize].link;
            if m.is_term(c) || depth > m.mem.len() {
                break;
            }
        }
        format!("{{\"depth\":{},\"items\":[{}]}}", depth, items.join(","))
    }

    /// The cells with system functions, and where the interpreter is.
    pub fn state(&self) -> String {
        let m = &self.m;
        let mut calls = Vec::new();
        let mut c = H1;
        while calls.len() < 200 {
            let frame = m.mem[c as usize];
            let line = m.origins.get(&frame.symb).map(|o| o.line).unwrap_or(0);
            calls.push(format!(
                "{{\"a\":{},\"cell\":{},\"place\":\"{}\",\"line\":{},\"native\":{}}}",
                frame.symb,
                c,
                esc(&m.place(frame.symb)),
                line,
                frame.p & NATIVE != 0
            ));
            c = frame.link;
            if c == 0 {
                break;
            }
        }
        let cia = m.mem[H1 as usize].symb;
        let word = m.mem[cia as usize];
        let origin = m.origins.get(&cia);
        let w: Vec<String> = (0..10).map(|i| self.stack(w(i), 6)).collect();
        let regions: Vec<String> = m
            .regions
            .iter()
            .map(|r| {
                format!(
                    "{{\"ch\":\"{}\",\"origin\":{},\"size\":{}}}",
                    esc(&(r.ch as char).to_string()),
                    r.origin,
                    r.size
                )
            })
            .collect();
        format!(
            concat!(
                "{{\"cycles\":{},\"h3\":{},\"level\":{},\"halted\":{},\"error\":{},",
                "\"free\":{},\"low\":{},\"space\":{},\"brk\":{},\"top\":{},\"h5\":\"{}\",",
                "\"cia\":{{\"a\":{},\"place\":\"{}\",\"line\":{},\"p\":{},\"q\":{},\"symb\":\"{}\",\"link\":\"{}\",\"comment\":\"{}\"}},",
                "\"calls\":[{}],\"h0\":{},\"w\":[{}],\"regions\":[{}],\"cards\":{}}}"
            ),
            m.cycles,
            m.int(H3, "H3").unwrap_or(0),
            m.level,
            m.halted,
            match &self.error {
                Some(e) => format!("\"{}\"", esc(e)),
                None => "null".to_string(),
            },
            m.free_cells,
            m.low_water,
            m.space(),
            m.brk(),
            m.top(),
            if m.h5() { '+' } else { '-' },
            cia,
            esc(&m.place(cia)),
            origin.map(|o| o.line).unwrap_or(0),
            word.p,
            word.q,
            esc(&if word.q == Q_PRIM { format!("J{}", word.symb) } else { m.label(word.symb) }),
            esc(&m.label(word.link)),
            esc(&origin.map(|o| o.comment.clone()).unwrap_or_default()),
            calls.join(","),
            self.stack(H0, 24),
            w.join(","),
            regions.join(","),
            m.cards.len(),
        )
    }

    /// Where the cards went: `[[line, address], ...]`.
    pub fn lines(&self) -> String {
        let mut v: Vec<(usize, u32)> = self.m.origins.iter().map(|(a, o)| (o.line, *a)).collect();
        v.sort();
        let items: Vec<String> = v.iter().map(|(l, a)| format!("[{},{}]", l, a)).collect();
        format!("[{}]", items.join(","))
    }

    /// One cell, for a pointer held over the map of storage.
    pub fn describe(&self, a: u32) -> String {
        if a as usize >= self.m.mem.len() {
            return String::new();
        }
        format!("{}   {}", self.m.place(a), self.m.contents(a))
    }

    /// A list structure as lists of cells: the list `root`, then each list
    /// whose local name occurs on a list already taken. Unlike J101 this
    /// does not trust the structure to be well formed: it stops at `limit`
    /// cells, and says so, and it says where a list runs into itself.
    pub fn structure(&self, root: u32, limit: usize) -> String {
        let m = &self.m;
        let mut lists = vec![root];
        let mut seen: HashSet<u32> = lists.iter().copied().collect();
        let mut out = Vec::new();
        let mut total = 0;
        let mut i = 0;
        while i < lists.len() {
            let head = lists[i];
            i += 1;
            let mut cells = Vec::new();
            let mut on_list = HashSet::new();
            let mut c = head;
            let end = loop {
                if total >= limit {
                    break "more";
                }
                if !on_list.insert(c) {
                    break "loop";
                }
                total += 1;
                cells.push(self.cell(c));
                let word = m.mem[c as usize];
                if word.is_data() {
                    break "data";
                }
                if word.q == Q_LOCAL
                    && word.symb != 0
                    && (word.symb as usize) < m.mem.len()
                    && seen.insert(word.symb)
                {
                    lists.push(word.symb);
                }
                if word.link == 0 {
                    break "0";
                }
                if m.is_term(word.link) || word.link as usize >= m.mem.len() {
                    break "cell";
                }
                c = word.link;
            };
            out.push(format!(
                "{{\"a\":{},\"name\":\"{}\",\"end\":\"{}\",\"cells\":[{}]}}",
                head,
                esc(&m.label(head)),
                end,
                cells.join(",")
            ));
            if total >= limit {
                break;
            }
        }
        format!(
            "{{\"root\":{},\"cells\":{},\"lists\":[{}]}}",
            root,
            total,
            out.join(",")
        )
    }
}
