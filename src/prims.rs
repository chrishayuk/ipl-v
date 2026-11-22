//! The basic processes, J0 to J202 (manual sections 5 to 12, 15, 19 to 24).
//!
//! Each takes its inputs from the communication cell H0, removes them, and
//! leaves there only the outputs the manual names. The print and line read
//! processes are in `print.rs`.

use crate::interp::{Cont, Outcome};
use crate::machine::*;
use crate::word::*;

/// An empty cell: the internal symbol 0 (section 12.2). Not the same word
/// as a termination cell, whose Q is 0.
pub const EMPTY: Word = Word {
    p: 0,
    q: Q_INTERNAL,
    symb: 0,
    link: 0,
};

/// A J101 in progress (manual, Part One, section 9, and Fig. 11).
pub struct StructureGen {
    pub(crate) sub: u32,
    /// Every list marked processed, to be restored at the end.
    pub(crate) record: Vec<u32>,
    /// Sublists found and not yet generated, in the order found.
    pub(crate) waiting: std::collections::VecDeque<u32>,
    /// The next cell to be generated, found before the subprocess sees
    /// the present one, so that the subprocess may erase it.
    pub(crate) next: Option<u32>,
}

enum Num {
    I(i64),
    F(f64),
}

impl Machine {
    fn num(&self, a: u32, what: &str) -> Res<Num> {
        match self.datum(a) {
            Some(Datum::Int(i)) => Ok(Num::I(i)),
            Some(Datum::Float(f)) => Ok(Num::F(f)),
            Some(Datum::Octal(o)) => Ok(Num::I(o as i64)),
            _ => Err(format!("{}: {} is not a number", what, self.label(a))),
        }
    }

    fn float(&self, a: u32, what: &str) -> Res<f64> {
        Ok(match self.num(a, what)? {
            Num::I(i) => i as f64,
            Num::F(f) => f,
        })
    }

    fn set_top(&mut self, s: Sym) {
        self.mem[H0 as usize].symb = s.a;
        self.mem[H0 as usize].q = s.q;
    }

    fn put(&mut self, cell: u32, s: Sym) {
        self.mem[cell as usize].symb = s.a;
        self.mem[cell as usize].q = s.q;
    }

    fn copy_name(&self, from: Sym, new: u32) -> Sym {
        Sym {
            a: new,
            q: if from.q == Q_LOCAL { Q_LOCAL } else { Q_INTERNAL },
        }
    }

    /// The next number of the sequence controlled by W10 (J129), as a
    /// fraction of 10^10. The multiplier is the one in W. T. Beyer's IBM
    /// 1620 system, the only one whose listing is to hand.
    fn random(&mut self) -> Res<i64> {
        let cell = self.mem[w(10) as usize].symb;
        let seed = self.int(cell, "J129")?;
        let next = ((seed as i128 * 9013).rem_euclid(10_000_000_000)) as i64;
        self.set_int(cell, next);
        Ok(next)
    }

    fn generate_symbols(&mut self, sub: u32, cell: u32) -> Res<Outcome> {
        self.set_h5(true);
        if self.is_term(cell) {
            return Ok(Outcome::Done);
        }
        let next = self.mem[cell as usize].link;
        self.push(self.mem[cell as usize].sym())?;
        Ok(Outcome::Call(sub, Cont::Symbols { sub, next }))
    }

    /// The origin and size in a block control word (section 17).
    fn block(&self, a: u32, what: &str) -> Res<(u32, u32)> {
        let w = self.mem[a as usize];
        if w.p != 7 || w.q != 7 {
            return Err(format!("{}: {} is not a block control word", what, self.label(a)));
        }
        if (w.symb + w.link) as usize > self.mem.len() {
            return Err(format!(
                "{}: the block {} runs off the end of storage",
                what,
                self.label(a)
            ));
        }
        Ok((w.symb, w.link))
    }

    /// J201 (section 24): the cells of a routine go back to available space.
    /// Its name is a regional cell, which is set empty. Any non-regional
    /// symbol in SYMB is taken to name a sublist and erased with it, and a
    /// regional LINK ends a list as 0 does.
    fn erase_routine(&mut self, routine: u32) -> Res<()> {
        let w = self.mem[routine as usize];
        if w.q == 6 || w.q == 7 {
            return Ok(());
        }
        let mut seen = std::collections::HashSet::new();
        let mut lists = vec![routine];
        let mut cells = Vec::new();
        while let Some(head) = lists.pop() {
            let mut c = head;
            loop {
                // The routine's own name is regional; any other regional
                // cell is where the list ends.
                if c == 0 || (c != routine && self.is_regional(c)) || !seen.insert(c) {
                    break;
                }
                let word = self.mem[c as usize];
                cells.push(c);
                let s = word.symb;
                if s != 0
                    && word.q != Q_PRIM
                    && !self.is_regional(s)
                    && !SYSTEM.contains(&s)
                    && (s as usize) < self.mem.len()
                    && !seen.contains(&s)
                {
                    lists.push(s);
                }
                c = word.link;
            }
        }
        for c in cells {
            self.free(c);
        }
        Ok(())
    }

    /// J60 on a cell: the next cell, returning a private termination cell
    /// to available space if it meets one.
    fn locate_next(&mut self, c: u32) -> Option<u32> {
        let l = self.mem[c as usize].link;
        if self.is_term(l) {
            if l != 0 {
                self.free(l);
                self.mem[c as usize].link = 0;
            }
            None
        } else {
            Some(l)
        }
    }

    /// J137: preserve the head and put the processed mark in it.
    fn mark_processed(&mut self, a: u32) -> Res<()> {
        self.preserve(a)?;
        self.mem[a as usize].p = 1;
        self.put(a, EMPTY.sym());
        Ok(())
    }

    fn marked(&self, a: u32) -> bool {
        let w = self.mem[a as usize];
        w.p == 1 && w.q != Q_DATA
    }

    /// Give the subprocess of the J101 on top the cell `c`, with H5 + if
    /// it is the head of a list, after finding the cell to come after it.
    fn present(&mut self, c: u32, head: bool, follow: bool) -> Res<Outcome> {
        let next = if follow { self.locate_next(c) } else { None };
        let g = self.structures.last_mut().unwrap();
        g.next = next.or_else(|| g.waiting.pop_front());
        let sub = g.sub;
        self.push(Sym { a: c, q: Q_INTERNAL })?;
        self.set_h5(head);
        Ok(Outcome::Call(sub, Cont::Structure))
    }

    /// The next cell of a J101, after the subprocess has had the last:
    /// "Diagnose current cell" in Fig. 11.
    fn generate_structure(&mut self) -> Res<Outcome> {
        let Some(c) = self.structures.last().unwrap().next else {
            return self.end_structure(true);
        };
        let w = self.mem[c as usize];
        if self.marked(c) {
            // The head of a list: its own word is in the cell after it.
            return self.present(c, true, true);
        }
        if w.is_data() {
            return self.present(c, false, false);
        }
        let s = w.symb;
        if s != 0 && w.q == Q_LOCAL && !self.is_regional(s) && !self.marked(s) {
            self.mark_processed(s)?;
            let g = self.structures.last_mut().unwrap();
            g.record.push(s);
            g.waiting.push_back(s);
        }
        self.present(c, false, true)
    }

    /// "Clean up" in Fig. 11: take the marks off, most recent first, and
    /// erase the head of any list the subprocess erased.
    fn end_structure(&mut self, completed: bool) -> Res<Outcome> {
        let g = self.structures.pop().unwrap();
        for &list in g.record.iter().rev() {
            self.restore(list);
            if self.is_term(list) {
                self.free(list);
            }
        }
        self.set_h5(completed);
        Ok(Outcome::Done)
    }

    fn generate_cells(&mut self, sub: u32) -> Res<Outcome> {
        match self.gens.last_mut().and_then(|g| g.pop()) {
            Some((cell, head)) => {
                self.push(Sym {
                    a: cell,
                    q: Q_INTERNAL,
                })?;
                self.set_h5(head);
                Ok(Outcome::Call(sub, Cont::Cells { sub }))
            }
            None => {
                self.gens.pop();
                self.set_h5(true);
                Ok(Outcome::Done)
            }
        }
    }

    /// The cells of a tree in the order of J102: each sublist straight
    /// after the cell that names it.
    fn tree(&self, list: u32, out: &mut Vec<(u32, bool)>) -> Res<()> {
        if out.len() > self.mem.len() {
            return Err("J102: the structure is not a tree".to_string());
        }
        let mut cells = vec![list];
        if !self.mem[list as usize].is_data() {
            cells.extend(self.cells_after(list)?);
        }
        for (i, c) in cells.into_iter().enumerate() {
            out.push((c, i == 0));
            let word = self.mem[c as usize];
            if word.q == Q_LOCAL && word.symb != 0 {
                self.tree(word.symb, out)?;
            }
        }
        Ok(())
    }

    fn context_size(&self) -> u32 {
        self.mem[G_WN as usize].symb.saturating_sub(W_ORG).min(9)
    }

    /// Finish a process that executed a routine.
    pub fn resume(&mut self, k: Cont) -> Res<Outcome> {
        match k {
            Cont::Subprocess => {
                // The generator comes back into context: its hideout first,
                // then its working cells.
                for cell in [G_SIGN, G_SUB, G_WN] {
                    self.preserve(cell)?;
                    let s = self.mem[G_CONTEXT as usize].sym();
                    self.put(cell, s);
                    self.restore(G_CONTEXT);
                }
                for i in (0..=self.context_size()).rev() {
                    self.preserve(w(i))?;
                    let s = self.mem[G_CONTEXT as usize].sym();
                    self.put(w(i), s);
                    self.restore(G_CONTEXT);
                }
                self.mem[G_SIGN as usize].symb = self.mem[H5 as usize].symb;
                Ok(Outcome::Done)
            }
            Cont::Symbols { sub, next } => {
                if self.h5() {
                    self.generate_symbols(sub, next)
                } else {
                    Ok(Outcome::Done)
                }
            }
            Cont::Cells { sub } => {
                if self.h5() {
                    self.generate_cells(sub)
                } else {
                    self.gens.pop();
                    Ok(Outcome::Done)
                }
            }
            Cont::Structure => {
                if self.h5() {
                    self.generate_structure()
                } else {
                    self.end_structure(false)
                }
            }
            Cont::MonitorStart => Ok(Outcome::ContinueP),
            Cont::MonitorEnd => Ok(Outcome::AscendMore),
            Cont::H5Plus => {
                self.set_h5(true);
                Ok(Outcome::Done)
            }
            Cont::Trap { space } => {
                self.restore(w(27));
                self.restore(w(28));
                if space {
                    self.withhold();
                }
                Ok(Outcome::Done)
            }
        }
    }

    pub fn prim(&mut self, n: u32) -> Res<Outcome> {
        let name = format!("J{}", n);
        let what = name.as_str();
        self.moves_in(n)?;
        match n {
            // ---- general processes, section 5 ----
            0 => {}
            1 => return Ok(Outcome::Exec(self.pop().a)),
            2 => {
                let (a, b) = (self.pop(), self.pop());
                self.set_h5(a.a == b.a);
            }
            3 => self.set_h5(false),
            4 => self.set_h5(true),
            5 => self.set_h5(!self.h5()),
            6 => {
                let below = self.mem[H0 as usize].link;
                if below != 0 {
                    let (a, b) = (self.arg(0), self.arg(1));
                    self.put(H0, b);
                    self.put(below, a);
                }
            }
            7 => return Ok(Outcome::Halt),
            8 => self.restore(H0),
            9 => {
                let s = self.pop();
                self.free(s.a);
            }

            // ---- description processes, section 6 ----
            10 => {
                let (attr, list) = (self.pop(), self.pop());
                match self.attribute(list.a, attr.a)? {
                    Some((_, _, v)) => {
                        self.push(self.mem[v as usize].sym())?;
                        self.set_h5(true);
                    }
                    None => self.set_h5(false),
                }
            }
            11 => {
                let (attr, value, list) = (self.pop(), self.pop(), self.pop());
                self.assign(list.a, attr, value)?;
            }
            12 | 13 => {
                let (attr, s, list) = (self.pop(), self.pop(), self.pop());
                let values = match self.attribute(list.a, attr.a)? {
                    Some((_, _, v)) => self.mem[v as usize].symb,
                    None => {
                        let new = self.alloc()?;
                        self.mem[new as usize] = EMPTY;
                        self.assign(list.a, attr, Sym { a: new, q: Q_LOCAL })?;
                        new
                    }
                };
                let after = if n == 12 { values } else { self.last_cell(values)? };
                self.insert_after(after, s)?;
            }
            14 => {
                let (attr, list) = (self.pop(), self.pop());
                if let Some((prev, a, v)) = self.attribute(list.a, attr.a)? {
                    let (aw, vw) = (self.mem[a as usize], self.mem[v as usize]);
                    self.mem[prev as usize].link = vw.link;
                    self.free(a);
                    self.free(v);
                    for word in [aw, vw] {
                        if word.q == Q_LOCAL && word.symb != 0 {
                            self.erase_structure(word.symb)?;
                        }
                    }
                }
            }
            15 => {
                let list = self.pop().a;
                let head = self.mem[list as usize];
                if !head.is_data() && head.symb != 0 {
                    self.erase_structure(head.symb)?;
                    self.put(list, EMPTY.sym());
                }
            }
            16 => {
                let list = self.pop().a;
                let d = self.mem[list as usize].symb;
                let mut population = Vec::new();
                if d != 0 {
                    for pair in self.cells_after(d)?.chunks(2) {
                        if pair.len() == 2 {
                            let v = self.mem[pair[1] as usize].symb;
                            if let Ok(weight) = self.float(v, what) {
                                if weight > 0.0 {
                                    population.push((self.mem[pair[0] as usize].sym(), weight));
                                }
                            }
                        }
                    }
                }
                if population.is_empty() {
                    self.set_h5(false);
                } else {
                    let total: f64 = population.iter().map(|p| p.1).sum();
                    let mut r = self.random()? as f64 / 1e10 * total;
                    let mut chosen = population[population.len() - 1].0;
                    for (attr, weight) in &population {
                        if r < *weight {
                            chosen = *attr;
                            break;
                        }
                        r -= weight;
                    }
                    self.push(chosen)?;
                    self.set_h5(true);
                }
            }

            // ---- generator housekeeping, section 7 ----
            17 => {
                let (wn, sub) = (self.pop(), self.pop());
                if wn.a < W_ORG || wn.a > w(9) {
                    return Err(format!("J17: {} is not one of W0 to W9", self.label(wn.a)));
                }
                for i in 0..=(wn.a - W_ORG) {
                    self.preserve(w(i))?;
                }
                self.preserve(G_WN)?;
                self.put(G_WN, wn);
                self.preserve(G_SUB)?;
                self.put(G_SUB, sub);
                self.preserve(G_SIGN)?;
                self.mem[G_SIGN as usize].symb = j(4);
            }
            18 => {
                // "The subprocess of a generator is part of its context, so
                // that J18 always fires the subprocess of the generator
                // currently in context" (section 7.3): the whole context,
                // hideout included, is put away while the subprocess runs.
                let sub = self.mem[G_SUB as usize].symb;
                for i in 0..=self.context_size() {
                    self.preserve(G_CONTEXT)?;
                    let s = self.mem[w(i) as usize].sym();
                    self.put(G_CONTEXT, s);
                    self.restore(w(i));
                }
                for cell in [G_WN, G_SUB, G_SIGN] {
                    self.preserve(G_CONTEXT)?;
                    let s = self.mem[cell as usize].sym();
                    self.put(G_CONTEXT, s);
                    self.restore(cell);
                }
                return Ok(Outcome::Call(sub, Cont::Subprocess));
            }
            19 => {
                for i in 0..=self.context_size() {
                    self.restore(w(i));
                }
                self.mem[H5 as usize].symb = self.mem[G_SIGN as usize].symb;
                self.restore(G_SIGN);
                self.restore(G_SUB);
                self.restore(G_WN);
            }

            // ---- working storage, section 8 ----
            20..=29 => {
                for i in 0..=(n - 20) {
                    let s = self.pop();
                    self.put(w(i), s);
                }
            }
            30..=39 => {
                for i in 0..=(n - 30) {
                    self.restore(w(i));
                }
            }
            40..=49 => {
                for i in 0..=(n - 40) {
                    self.preserve(w(i))?;
                }
            }
            50..=59 => {
                for i in 0..=(n - 50) {
                    self.preserve(w(i))?;
                }
                for i in 0..=(n - 50) {
                    let s = self.pop();
                    self.put(w(i), s);
                }
            }

            // ---- list processes, section 9 ----
            60 => {
                let c = self.arg(0).a;
                let l = self.mem[c as usize].link;
                if self.is_term(l) {
                    if l != 0 {
                        self.free(l);
                        self.mem[c as usize].link = 0;
                    }
                    self.set_h5(false);
                } else {
                    self.set_top(Sym { a: l, q: Q_INTERNAL });
                    self.set_h5(true);
                }
            }
            61 => {
                let c = self.arg(0).a;
                let last = self.last_cell(c)?;
                self.set_top(Sym {
                    a: last,
                    q: Q_INTERNAL,
                });
                self.set_h5(last != c);
            }
            62 => {
                let s = self.pop();
                let list = self.arg(0).a;
                match self.locate(list, s.a)? {
                    Some(c) => {
                        self.set_top(Sym { a: c, q: Q_INTERNAL });
                        self.set_h5(true);
                    }
                    None => {
                        let last = self.last_cell(list)?;
                        self.set_top(Sym {
                            a: last,
                            q: Q_INTERNAL,
                        });
                        self.set_h5(false);
                    }
                }
            }
            63 => {
                let (s, c) = (self.pop(), self.pop().a);
                self.preserve(c)?;
                self.put(c, s);
            }
            64 => {
                let (s, c) = (self.pop(), self.pop().a);
                if c != 0 && self.mem[c as usize] == Word::ZERO && !self.is_regional(c) {
                    // (1) is a private termination cell: the symbol goes in it.
                    self.put(c, s);
                } else {
                    self.insert_after(c, s)?;
                }
            }
            65 | 66 => {
                let (s, list) = (self.pop(), self.pop().a);
                if n == 65 || self.locate(list, s.a)?.is_none() {
                    let last = self.last_cell(list)?;
                    self.insert_after(last, s)?;
                }
            }
            67 => {
                let (new, old, list) = (self.pop(), self.pop(), self.pop().a);
                if let Some(c) = self.locate(list, old.a)? {
                    self.put(c, new);
                }
            }
            68 => {
                let c = self.pop().a;
                let more = self.delete(c);
                self.set_h5(more);
            }
            69 => {
                let (s, list) = (self.pop(), self.pop().a);
                match self.locate(list, s.a)? {
                    Some(c) => {
                        self.delete(c);
                        self.set_h5(true);
                    }
                    None => self.set_h5(false),
                }
            }
            70 => {
                let list = self.pop().a;
                match self.cells_after(list)?.last() {
                    Some(&c) => {
                        self.delete(c);
                        self.set_h5(true);
                    }
                    None => self.set_h5(false),
                }
            }
            71 => {
                let list = self.pop().a;
                self.erase_list(list)?;
            }
            72 => {
                let list = self.pop().a;
                if self.is_filed(list) {
                    self.erase_filed(list)?;
                } else {
                    self.erase_structure(list)?;
                }
            }
            74 if self.is_filed(self.arg(0).a) => {
                let s = self.pop();
                let new = self.copy_filed(s.a)?;
                self.push(self.copy_name(s, new))?;
            }
            73 | 74 => {
                let s = self.pop();
                let new = if n == 73 {
                    self.copy_list(s.a)?
                } else {
                    self.copy_structure(s.a)?
                };
                self.push(self.copy_name(s, new))?;
            }
            75 => {
                let c = self.pop().a;
                let rest = self.mem[c as usize].link;
                self.mem[c as usize].link = 0;
                let head = self.alloc()?;
                self.mem[head as usize] = Word { link: rest, ..EMPTY };
                self.push(Sym {
                    a: head,
                    q: Q_INTERNAL,
                })?;
            }
            76 => {
                let list = self.pop().a;
                let c = self.arg(0).a;
                let head = self.mem[list as usize];
                let cells = self.cells_after(list)?;
                if head.q == Q_LOCAL && head.symb != 0 {
                    self.erase_structure(head.symb)?;
                }
                match (cells.first(), cells.last()) {
                    (Some(&first), Some(&last)) => {
                        let after = self.mem[c as usize].link;
                        self.free(self.mem[last as usize].link);
                        self.mem[last as usize].link = after;
                        self.mem[c as usize].link = first;
                        self.set_top(Sym {
                            a: last,
                            q: Q_INTERNAL,
                        });
                        self.set_h5(true);
                    }
                    _ => {
                        self.free(head.link);
                        self.set_h5(false);
                    }
                }
                self.free(list);
            }
            77 => {
                let (s, list) = (self.pop(), self.pop().a);
                let found = self.locate(list, s.a)?.is_some();
                self.set_h5(found);
            }
            78 => {
                let list = self.pop().a;
                let empty = self.is_term(self.mem[list as usize].link);
                self.set_h5(!empty);
            }
            79 => {
                let c = self.pop().a;
                self.set_h5(self.mem[c as usize].symb != 0);
            }
            80 => {
                let list = self.pop().a;
                if self.is_term(list) {
                    self.set_h5(false);
                } else {
                    self.push(self.mem[list as usize].sym())?;
                    self.set_h5(true);
                }
            }
            81..=89 => {
                let list = self.pop().a;
                match self.cells_after(list)?.get((n - 81) as usize) {
                    Some(&c) => {
                        self.push(self.mem[c as usize].sym())?;
                        self.set_h5(true);
                    }
                    None => self.set_h5(false),
                }
            }
            90..=99 => {
                let symbols: Vec<Sym> = (0..n - 90).map(|_| self.pop()).collect();
                let head = self.alloc()?;
                self.mem[head as usize] = EMPTY;
                let mut c = head;
                for s in symbols.into_iter().rev() {
                    c = self.insert_after(c, s)?;
                }
                self.push(Sym {
                    a: head,
                    q: Q_INTERNAL,
                })?;
            }
            100 => {
                let (sub, list) = (self.pop().a, self.pop().a);
                let first = self.mem[list as usize].link;
                return self.generate_symbols(sub, first);
            }
            101 => {
                let (sub, list) = (self.pop().a, self.pop().a);
                self.structures.push(StructureGen {
                    sub,
                    record: vec![list],
                    waiting: Default::default(),
                    next: None,
                });
                self.mark_processed(list)?;
                return self.present(list, true, true);
            }
            103 => {
                let (sub, block) = (self.pop().a, self.pop().a);
                let (origin, size) = self.block(block, what)?;
                let mut cells: Vec<(u32, bool)> = (origin..origin + size).map(|a| (a, true)).collect();
                cells.reverse();
                self.gens.push(cells);
                return self.generate_cells(sub);
            }
            102 => {
                let (sub, list) = (self.pop().a, self.pop().a);
                let mut cells = Vec::new();
                self.tree(list, &mut cells)?;
                cells.reverse();
                self.gens.push(cells);
                return self.generate_cells(sub);
            }

            // ---- arithmetic, section 11 ----
            110..=113 => {
                let (r, a, b) = (self.pop(), self.pop().a, self.pop().a);
                let result = match (self.num(a, what)?, self.num(b, what)?) {
                    (Num::I(x), Num::I(y)) => Datum::Int(match n {
                        110 => x.wrapping_add(y),
                        111 => x.wrapping_sub(y),
                        112 => x.wrapping_mul(y),
                        _ => {
                            if y == 0 {
                                return Err("J113: division by zero".to_string());
                            }
                            self.set_int(w(11), x.wrapping_rem(y));
                            x.wrapping_div(y)
                        }
                    }),
                    _ => {
                        let (x, y) = (self.float(a, what)?, self.float(b, what)?);
                        Datum::Float(match n {
                            110 => x + y,
                            111 => x - y,
                            112 => x * y,
                            _ => x / y,
                        })
                    }
                };
                self.mem[r.a as usize] = Word::from_datum(result);
                self.push(r)?;
            }
            114 => {
                let (a, b) = (self.pop().a, self.pop().a);
                let (x, y) = (self.mem[a as usize], self.mem[b as usize]);
                self.set_h5(x.is_data() && x == y);
            }
            115 | 116 => {
                let (a, b) = (self.pop().a, self.pop().a);
                let (x, y) = (self.float(a, what)?, self.float(b, what)?);
                self.set_h5(if n == 115 { x > y } else { x < y });
            }
            117..=119 => {
                let a = self.pop().a;
                let x = self.float(a, what)?;
                self.set_h5(match n {
                    117 => x == 0.0,
                    118 => x > 0.0,
                    _ => x < 0.0,
                });
            }
            120 => {
                let s = self.pop();
                let new = self.alloc()?;
                self.mem[new as usize] = self.mem[s.a as usize];
                self.push(self.copy_name(s, new))?;
            }
            121 => {
                let (a, b) = (self.pop(), self.pop());
                self.mem[a.a as usize] = self.mem[b.a as usize];
                self.push(a)?;
            }
            122 | 123 => {
                let a = self.arg(0).a;
                let d = match self.num(a, what)? {
                    Num::I(x) => Datum::Int(if n == 122 {
                        x.wrapping_abs()
                    } else {
                        x.wrapping_neg()
                    }),
                    Num::F(x) => Datum::Float(if n == 122 { x.abs() } else { -x }),
                };
                self.mem[a as usize] = Word::from_datum(d);
            }
            124 => {
                let a = self.arg(0).a;
                let d = match self.datum(a) {
                    Some(Datum::Float(_)) => Datum::Float(0.0),
                    Some(Datum::Alpha(_)) => Datum::Alpha([b' '; 5]),
                    Some(Datum::Octal(_)) => Datum::Octal(0),
                    _ => Datum::Int(0),
                };
                self.mem[a as usize] = Word::from_datum(d);
            }
            125 => {
                let a = self.arg(0).a;
                let d = match self.num(a, what)? {
                    Num::I(x) => Datum::Int(x.wrapping_add(1)),
                    Num::F(x) => Datum::Float(x + 1.0),
                };
                self.mem[a as usize] = Word::from_datum(d);
            }
            126 => {
                let list = self.pop().a;
                let count = if list == H2 {
                    self.free_cells as usize
                } else {
                    self.cells_after(list)?.len()
                };
                let new = self.new_int(count as i64)?;
                self.push(Sym {
                    a: new,
                    q: Q_INTERNAL,
                })?;
            }
            127 => {
                let (a, b) = (self.pop().a, self.pop().a);
                self.set_h5(self.mem[a as usize].p == self.mem[b as usize].p);
            }
            128 => {
                let (a, b) = (self.pop(), self.pop().a);
                let d = match (self.num(a.a, what)?, self.mem[b as usize].p) {
                    (Num::I(x), 1) => Datum::Float(x as f64),
                    (Num::F(x), 0) => Datum::Int(x as i64),
                    (Num::I(x), _) => Datum::Int(x),
                    (Num::F(x), _) => Datum::Float(x),
                };
                self.mem[a.a as usize] = Word::from_datum(d);
                self.push(a)?;
            }
            129 => {
                let limit = self.pop().a;
                let r = self.random()?;
                let d = match self.num(limit, what)? {
                    Num::I(x) => Datum::Int((r as i128 * x as i128 / 10_000_000_000) as i64),
                    Num::F(x) => Datum::Float(r as f64 / 1e10 * x),
                };
                let new = self.alloc()?;
                self.mem[new as usize] = Word::from_datum(d);
                self.push(Sym {
                    a: new,
                    q: Q_INTERNAL,
                })?;
            }

            // ---- data prefixes, section 12 ----
            130 => {
                let s = self.pop();
                self.set_h5(self.is_regional(s.a));
            }
            131 => {
                let s = self.pop();
                self.set_h5(self.mem[s.a as usize].is_data());
            }
            132 => {
                let s = self.pop();
                self.set_h5(s.q == Q_LOCAL && !self.is_regional(s.a));
            }
            133 => {
                let a = self.pop().a;
                let word = self.mem[a as usize];
                self.set_h5(word.p == 1 && word.q != Q_DATA);
            }
            134 => {
                let s = self.pop();
                self.set_h5(s.q != Q_LOCAL && !self.is_regional(s.a));
            }
            136 => self.mem[H0 as usize].q = Q_LOCAL,
            137 => {
                let a = self.arg(0).a;
                self.preserve(a)?;
                self.mem[a as usize].p = 1;
                self.put(a, EMPTY.sym());
            }
            138 => self.mem[H0 as usize].q = Q_INTERNAL,

            // ---- monitor system, section 15 ----
            147 | 148 => {
                let a = self.pop().a;
                let mark = if n == 147 { 3 } else { 4 };
                if matches!(self.mem[a as usize].q, 0 | 3 | 4) {
                    self.mem[a as usize].q = mark;
                } else {
                    self.preserve(a)?;
                    let link = self.mem[a as usize].link;
                    self.mem[a as usize] = Word {
                        p: 0,
                        q: mark,
                        symb: j(0),
                        link,
                    };
                }
            }
            149 => {
                let a = self.pop().a;
                let word = self.mem[a as usize];
                if matches!(word.q, 3 | 4) {
                    if word.symb == j(0) && word.p == 0 {
                        self.restore(a);
                    } else {
                        self.mem[a as usize].q = 0;
                    }
                }
            }
            150..=162 | 180..=189 => return self.print_prim(n),

            // ---- in-process loading and restart, sections 19 and 20 ----
            165 => match self.load_from_unit()? {
                Some(crate::loader::Start::At(start)) => return Ok(Outcome::Exec(start)),
                Some(crate::loader::Start::Restart) => return Ok(Outcome::Restarted),
                None => {}
            },
            166 => {
                let u = self.pop().a;
                let unit = self.int(u, what)?;
                if !(1..=10).contains(&unit) {
                    return Err(format!(
                        "{}: save for restart needs a tape, 1 to 10, not unit {}",
                        what, unit
                    ));
                }
                // H5 is set plus in this run; the image has it minus, as the
                // restart sets it.
                let image = self.snapshot();
                self.write_record(unit as u32, crate::tape::Record::Image(Box::new(image)));
                self.set_h5(true);
            }
            140..=146 | 167 | 173 | 174 | 176 => return self.tape_prim(n),

            // ---- error trap, section 21 ----
            170 => {
                let s = self.pop();
                if let Some(action) = self.trap_action(s)? {
                    return Ok(Outcome::Call(action, Cont::Trap { space: false }));
                }
            }
            171 => self.return_unused_regionals(),
            172 => {
                let block = self.pop().a;
                let (origin, size) = self.block(block, what)?;
                for a in origin..origin + size {
                    self.mem[a as usize].link = if a + 1 < origin + size { a + 1 } else { 0 };
                }
                self.push(Sym {
                    a: origin,
                    q: Q_INTERNAL,
                })?;
            }
            175 => {
                let s = self.pop();
                let found = self
                    .regions
                    .iter()
                    .find(|r| s.a >= r.origin && s.a < r.origin + r.size)
                    .map(|r| r.ch);
                match found.and_then(|ch| self.region_word(ch)) {
                    Some(word) => {
                        self.push(Sym {
                            a: word,
                            q: Q_INTERNAL,
                        })?;
                        self.set_h5(true);
                    }
                    None => self.set_h5(false),
                }
            }

            // ---- partial words, section 23 ----
            190..=193 => {
                let a = self.pop().a;
                let word = self.mem[a as usize];
                let s = match n {
                    190 => Sym {
                        a: word.p as u32,
                        q: Q_INTERNAL,
                    },
                    191 => Sym {
                        a: word.q as u32,
                        q: Q_INTERNAL,
                    },
                    192 => self.sym(word.symb),
                    _ => self.sym(word.link),
                };
                self.push(s)?;
            }
            194..=197 => {
                let (c, v) = (self.pop().a as usize, self.pop().a);
                match n {
                    194 => self.mem[c].p = (v & 7) as u8,
                    195 => self.mem[c].q = (v & 7) as u8,
                    196 => self.mem[c].symb = v,
                    _ => self.mem[c].link = v,
                }
            }

            // ---- miscellaneous, section 24 ----
            200 => {
                let count = self.pop().a;
                let list = self.arg(0).a;
                let k = self.int(count, what)?.unsigned_abs() as usize;
                let cells = self.cells_after(list)?;
                match k.checked_sub(1).and_then(|i| cells.get(i)) {
                    Some(&c) => {
                        self.set_top(Sym { a: c, q: Q_INTERNAL });
                        self.set_h5(true);
                    }
                    None => {
                        let last = cells.last().copied().unwrap_or(list);
                        self.set_top(Sym {
                            a: last,
                            q: Q_INTERNAL,
                        });
                        self.set_h5(false);
                    }
                }
            }
            201 => {
                let routine = self.pop().a;
                self.erase_routine(routine)?;
            }
            105..=109 => return self.aux_prim(n),
            202 => {
                let text = self.post_mortem("POST MORTEM");
                self.out.push_str(&text);
                self.out.push('\n');
            }
            _ => return Err(format!("J{} is not implemented", n)),
        }
        Ok(Outcome::Done)
    }
}
