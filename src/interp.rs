//! The interpreter (manual sections 3.10 to 3.16).
//!
//! `step` carries out the interpretation cycle of section 3.15 for the
//! instruction whose address is in H1, and stops when H1 names the next
//! one:
//!
//! ```text
//! INTERPRET Q   apply Q to SYMB to get the designated symbol S
//! INTERPRET P   0 execute S, 1 input S, 2 output to S, 3 restore S,
//!               4 preserve S, 5 replace (0) by S, 6 copy (0) in S,
//!               7 branch to S if H5 is -
//! DESCEND       preserve H1, put S in H1
//! ADVANCE       put LINK in H1, or ASCEND if LINK is 0
//! ASCEND        restore H1 and ADVANCE; halt if H1 is empty
//! ```
//!
//! Nothing is kept outside the IPL computer except the continuations of
//! those basic processes that execute a routine and have work to do when it
//! returns (J18, J100 and the monitor and trap actions); the level they
//! return to is marked in the P of its H1 word, where the manual keeps the
//! trace marks (section 15.6).

use crate::machine::*;
use crate::word::*;

/// Marks carried in H1 beside the current instruction address.
pub const TRACE: u8 = 1;
pub const MONITOR: u8 = 2;
pub const NATIVE: u8 = 4;

/// What a basic process asks of the interpreter when it finishes.
pub enum Outcome {
    Done,
    /// Execute this symbol in place of the process (J1, J165).
    Exec(u32),
    /// Execute this symbol as a subroutine, and afterwards resume.
    Call(u32, Cont),
    Halt,
    ContinueP,
    AscendMore,
}

/// Work waiting for a routine to finish.
#[derive(Clone, Copy, Debug)]
pub enum Cont {
    /// J18: bring the generator's context back.
    Subprocess,
    /// J100: the subprocess, and the next cell of the list.
    Symbols {
        sub: u32,
        next: u32,
    },
    /// J102: the cells still to be generated are in `gens`.
    Cells {
        sub: u32,
    },
    /// J101: its state is in `structures`.
    Structure,
    MonitorStart,
    MonitorEnd,
    /// A trap action, from J170 or taken by the system; `space` if it was
    /// the trap on H2, after which the cells are withheld again.
    Trap {
        space: bool,
    },
}

/// How the process was reached: named in an instruction with P = 0, or as
/// the LINK of the instruction before, where it is the last process of its
/// program list (section 3.14, rule 6).
#[derive(Clone, Copy, Debug)]
pub enum Ctx {
    Instruction,
    Link,
    None,
    /// A trap taken between instructions: afterwards interpretation goes
    /// on with the instruction it was about to interpret.
    Resume,
}

enum Next {
    Q,
    P,
    Advance,
    Ascend,
    Out(Outcome, Ctx),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Stop {
    Halted,
    Limit,
}

impl Machine {
    /// Begin interpretation at a routine.
    pub fn start(&mut self, routine: u32) {
        self.mem[H1 as usize] = Word {
            p: 0,
            q: 0,
            symb: routine,
            link: 0,
        };
        // The first instruction of the manual's traced run is the fifth
        // cycle: the system spends four in starting.
        self.set_int(H3, 4);
        self.withhold();
        self.halted = false;
        self.level = 1;
    }

    pub fn run(&mut self, limit: u64) -> Res<Stop> {
        while !self.halted {
            if self.cycles >= limit {
                return Ok(Stop::Limit);
            }
            self.step().map_err(|e| self.post_mortem(&e))?;
        }
        Ok(Stop::Halted)
    }

    /// H3 counts the words interpreted (section 3.16): one for each
    /// instruction, and one more for a primitive it executes, whose own
    /// cell is interpreted in turn. That is how the H3 column of the
    /// manual's trace runs (Part One, Fig. 2): 11 at `J60`, 13 at the
    /// instruction after it.
    fn tally(&mut self) {
        self.cycles += 1;
        let h3 = &mut self.mem[H3 as usize];
        if h3.q == Q_DATA && h3.p == 0 {
            *h3 = Word::data(0, h3.bits().wrapping_add(1));
            // Section 21: the trap on H3, when it equals the integer in W33.
            let w33 = self.mem[w(33) as usize];
            if w33.q == Q_DATA && w33.bits() != 0 && w33.bits() == self.mem[H3 as usize].bits() {
                self.pending_trap = Some(H3);
            }
        }
    }

    pub fn descend(&mut self, s: u32, marks: u8) -> Res<()> {
        self.preserve(H1)?;
        self.mem[H1 as usize].symb = s;
        self.mem[H1 as usize].p = marks;
        self.level += 1;
        Ok(())
    }

    fn primitive(&mut self, n: u32, ctx: Ctx) -> Res<Next> {
        if let Some(c) = self.prim_counts.get_mut(n as usize) {
            *c += 1;
        }
        Ok(Next::Out(self.prim(n)?, ctx))
    }

    /// One instruction.
    pub fn step(&mut self) -> Res<()> {
        if let Some(t) = self.pending_trap.take() {
            if !self.halted {
                let space = t == H2;
                let sym = Sym { a: t, q: Q_REGIONAL };
                // With no action, the program may use the withheld cells; if
                // it runs out again, that is an error.
                if let Some(action) = self.trap_action(sym)? {
                    self.conts.push((Cont::Trap { space }, Ctx::Resume));
                    let marks = NATIVE | (self.mem[H1 as usize].p & TRACE);
                    return self.descend(action, marks);
                }
            }
        }
        let mut next = Next::Q;
        loop {
            next = match next {
                Next::Q => {
                    self.tally();
                    let cia = self.mem[H1 as usize].symb;
                    let word = self.mem[cia as usize];
                    match word.q {
                        // Part of the step of the instruction that led here.
                        5 => self.primitive(word.symb, Ctx::Link)?,
                        6 | 7 => return Err("routines in auxiliary storage are not implemented".to_string()),
                        3 => {
                            // A monitor point (section 15.1).
                            self.mem[H1 as usize].p |= TRACE | MONITOR;
                            self.preserve(w(29))?;
                            self.mem[w(29) as usize].symb = cia;
                            let action = self.mem[w(12) as usize].symb;
                            if action != j(0) && action != 0 {
                                self.conts.push((Cont::MonitorStart, Ctx::None));
                                return self.descend(action, NATIVE);
                            }
                            Next::P
                        }
                        4 => {
                            let up = self.mem[H1 as usize].link;
                            if self.mem[up as usize].p & TRACE != 0 {
                                self.mem[H1 as usize].p |= TRACE;
                            }
                            Next::P
                        }
                        _ => Next::P,
                    }
                }
                Next::P => {
                    let cia = self.mem[H1 as usize].symb;
                    let word = self.mem[cia as usize];
                    let s = self.designate(word);
                    self.trace(cia, word, Some(s));
                    match word.p {
                        0 => Next::Out(Outcome::Exec(s.a), Ctx::Instruction),
                        1 => {
                            self.push(s)?;
                            Next::Advance
                        }
                        2 => {
                            let top = self.pop();
                            self.mem[s.a as usize].symb = top.a;
                            self.mem[s.a as usize].q = top.q;
                            Next::Advance
                        }
                        3 => {
                            self.restore(s.a);
                            Next::Advance
                        }
                        4 => {
                            self.preserve(s.a)?;
                            Next::Advance
                        }
                        5 => {
                            self.mem[H0 as usize].symb = s.a;
                            self.mem[H0 as usize].q = s.q;
                            Next::Advance
                        }
                        6 => {
                            let top = self.arg(0);
                            self.mem[s.a as usize].symb = top.a;
                            self.mem[s.a as usize].q = top.q;
                            Next::Advance
                        }
                        _ => {
                            if self.h5() {
                                Next::Advance
                            } else {
                                if s.a == 0 {
                                    Next::Ascend
                                } else {
                                    self.mem[H1 as usize].symb = s.a;
                                    return self.go_on();
                                }
                            }
                        }
                    }
                }
                Next::Advance => {
                    let cia = self.mem[H1 as usize].symb;
                    let link = self.mem[cia as usize].link;
                    if link == 0 {
                        Next::Ascend
                    } else {
                        self.mem[H1 as usize].symb = link;
                        return self.go_on();
                    }
                }
                Next::Ascend => {
                    let top = self.mem[H1 as usize];
                    if top.p & MONITOR != 0 {
                        self.mem[H1 as usize].p &= !MONITOR;
                        self.restore(w(29));
                        let action = self.mem[w(13) as usize].symb;
                        if action != j(0) && action != 0 {
                            self.conts.push((Cont::MonitorEnd, Ctx::None));
                            return self.descend(action, NATIVE);
                        }
                    }
                    if top.link == 0 {
                        self.halted = true;
                        return Ok(());
                    }
                    self.restore(H1);
                    self.level -= 1;
                    if top.p & NATIVE != 0 {
                        let (k, ctx) = self.conts.pop().ok_or("the interpreter lost a continuation")?;
                        Next::Out(self.resume(k)?, ctx)
                    } else {
                        Next::Advance
                    }
                }
                Next::Out(outcome, ctx) => match outcome {
                    Outcome::Done => match ctx {
                        Ctx::Link => Next::Ascend,
                        Ctx::Resume => return Ok(()),
                        _ => Next::Advance,
                    },
                    Outcome::Exec(s) => {
                        if s == 0 {
                            return Err(
                                "an instruction executes the symbol 0, which names no routine".to_string()
                            );
                        }
                        let word = self.mem[s as usize];
                        match ctx {
                            Ctx::Link => {
                                self.mem[H1 as usize].symb = s;
                                return self.go_on();
                            }
                            _ if word.q == Q_PRIM => {
                                self.tally();
                                self.primitive(word.symb, Ctx::Instruction)?
                            }
                            _ => {
                                if word.symb == 0 && word.link == 0 && word.p == 0 {
                                    return Err(format!(
                                        "{} is executed but was never defined",
                                        self.label(s)
                                    ));
                                }
                                return self.descend(s, 0);
                            }
                        }
                    }
                    Outcome::Call(s, k) => {
                        if s == 0 {
                            return Err("a process is asked to execute the symbol 0".to_string());
                        }
                        self.conts.push((k, ctx));
                        let marks = NATIVE | (self.mem[H1 as usize].p & TRACE);
                        return self.descend(s, marks);
                    }
                    Outcome::Halt => {
                        self.halted = true;
                        return Ok(());
                    }
                    Outcome::ContinueP => Next::P,
                    Outcome::AscendMore => Next::Ascend,
                },
            }
        }
    }

    /// The interpreter has moved to a new word. If the word names a basic
    /// process, reached as a LINK or by a branch, it is done as part of the
    /// same step: the manual's Fig. 3 counts `30HO J30` as one step, and
    /// its traces show no line for the process.
    fn go_on(&mut self) -> Res<()> {
        let cia = self.mem[H1 as usize].symb;
        if self.mem[cia as usize].q == Q_PRIM {
            self.step()
        } else {
            Ok(())
        }
    }

    /// The designation operation (section 3.10).
    fn designate(&self, word: Word) -> Sym {
        match word.q {
            1 => self.mem[word.symb as usize].sym(),
            2 => self.mem[self.mem[word.symb as usize].symb as usize].sym(),
            _ => self.sym(word.symb),
        }
    }

    fn tracing(&self) -> bool {
        if let Some(all) = self.trace_all {
            return all;
        }
        match self.wint(31) {
            Ok(0) => false,
            Ok(1) => true,
            _ => self.mem[H1 as usize].p & TRACE != 0,
        }
    }

    /// The trace line of section 15.5: level, CIA, H5, the instruction, S,
    /// (0) and its contents, and H3.
    fn trace(&mut self, cia: u32, word: Word, s: Option<Sym>) {
        if !self.tracing() {
            return;
        }
        let top = self.arg(0);
        // A LINK to the next card is the usual case and is left out.
        let follows = match (self.origins.get(&cia), self.origins.get(&word.link)) {
            (Some(a), Some(b)) => a.structure == b.structure && b.offset == a.offset + 1 && b.local.is_none(),
            _ => false,
        };
        let line = format!(
            "{:>3} {:<12} {} {}{} {:<6}{:<6} {:<7} {:<7} {:<16} {}\n",
            self.level,
            self.place(cia),
            if self.h5() { '+' } else { '-' },
            word.p,
            word.q,
            if word.q == Q_PRIM {
                format!("J{}", word.symb)
            } else {
                self.label(word.symb)
            },
            if follows {
                String::new()
            } else {
                self.label(word.link)
            },
            // S is shown where the designation operation changed it.
            match s {
                Some(s) if matches!(word.q, 1 | 2) => self.label(s.a),
                _ => String::new(),
            },
            self.name(top),
            self.contents(top.a),
            self.int(H3, "H3").unwrap_or(0),
        );
        self.trace_out.push_str(&line);
    }

    /// A word as a listing shows it.
    pub fn contents(&self, a: u32) -> String {
        let word = self.mem[a as usize];
        match word.datum() {
            Some(d) => format!("{}{} {}", word.p, word.q, datum_text(d)),
            None => format!(
                "{}{} {:<5} {}",
                word.p,
                word.q,
                self.name(word.sym()),
                self.label(word.link)
            ),
        }
    }

    /// What the machine was doing when it stopped on an error
    /// (section 15.4).
    pub fn post_mortem(&self, error: &str) -> String {
        let mut s = format!("{}\n", error);
        let mut c = H1;
        let mut depth = 0;
        while c != 0 && depth < 12 {
            let cia = self.mem[c as usize].symb;
            let word = self.mem[cia as usize];
            let comment = self
                .origins
                .get(&cia)
                .map(|o| o.comment.clone())
                .unwrap_or_default();
            s.push_str(&format!(
                "    {} {:<12} {}{} {:<6}{:<6} {}\n",
                if depth == 0 { "at" } else { "in" },
                self.place(cia),
                word.p,
                word.q,
                if word.q == Q_PRIM {
                    format!("J{}", word.symb)
                } else {
                    self.label(word.symb)
                },
                self.label(word.link),
                comment
            ));
            c = self.mem[c as usize].link;
            depth += 1;
        }
        s.push_str(&format!("    after {} interpretation cycles", self.cycles));
        s
    }
}
