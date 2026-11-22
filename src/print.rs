//! Printing and line reading (manual sections 16 and 22).
//!
//! J150 to J153 print IPL units of data. J154 to J161 compose a print line
//! and print it. J180 to J186 read a card into the same kind of line and
//! take symbols, data terms and characters out of it. Columns are numbered
//! from 1.

use crate::interp::Outcome;
use crate::machine::*;
use crate::word::*;
use std::collections::HashMap;

impl Machine {
    /// A symbol as J156 enters it: compactly, with `A0` as `A`.
    fn compact(&self, s: Sym) -> String {
        match self.regional_name(s.a) {
            Some((ch, 0)) => (ch as char).to_string(),
            Some((ch, n)) => format!("{}{}", ch as char, n),
            None => s.a.to_string(),
        }
    }

    /// A data term as J157 enters it (section 16.2).
    fn entry(&self, a: u32) -> Option<String> {
        Some(match self.datum(a)? {
            Datum::Alpha(t) => {
                let text = String::from_utf8_lossy(&t).into_owned();
                if text.trim().is_empty() {
                    text
                } else {
                    text.trim_end().to_string()
                }
            }
            d => datum_text(d),
        })
    }

    fn enter(&mut self, text: &str, left: bool) -> Res<()> {
        let col = self.wint(25)?;
        let len = text.len() as i64;
        let start = if left { col } else { col - len + 1 };
        let size = self.line().len() as i64;
        if start < 1 || start + len - 1 > size {
            self.set_h5(false);
            return Ok(());
        }
        let at = (start - 1) as usize;
        self.line()[at..at + text.len()].copy_from_slice(text.as_bytes());
        self.set_wint(25, start + len);
        self.set_h5(true);
        Ok(())
    }

    /// One list in the format of J150: NAME, PQ, SYMB, and the data term
    /// that SYMB names, if it names one. Local symbols are printed as the
    /// small integers in `locals`, or as their addresses if there is none.
    fn list_lines(&self, head: u32, locals: Option<&HashMap<u32, u32>>) -> Res<Vec<String>> {
        let show = |s: Sym| -> String {
            match locals.and_then(|l| l.get(&s.a)) {
                Some(n) => format!("9-{}", n),
                None => self.name(s),
            }
        };
        let mut cells = vec![head];
        if !self.mem[head as usize].is_data() {
            cells.extend(self.cells_after(head)?);
        }
        let mut lines = Vec::new();
        for (i, c) in cells.into_iter().enumerate() {
            let word = self.mem[c as usize];
            let name = if i == 0 {
                show(Sym {
                    a: head,
                    q: if locals.is_some_and(|l| l.contains_key(&head)) {
                        Q_LOCAL
                    } else {
                        Q_INTERNAL
                    },
                })
            } else {
                String::new()
            };
            // PQ is printed for data terms only, as in the manual's own
            // printed lists (Part One, Fig. 2).
            let body = match word.datum() {
                Some(d) => format!("{}{} {}", word.p, word.q, datum_text(d)),
                None => {
                    let named = match self.datum(word.symb) {
                        Some(d) if word.symb != 0 => {
                            let t = self.mem[word.symb as usize];
                            format!("{}{} {}", t.p, t.q, datum_text(d))
                        }
                        _ => String::new(),
                    };
                    format!("{:<9}{}", show(word.sym()), named)
                }
            };
            lines.push(format!("{:<9}{}", name, body));
        }
        Ok(lines)
    }

    fn field(&mut self) -> Res<Vec<u8>> {
        let (col, size) = (self.wint(25)?, self.wint(30)?);
        let line = self.line().clone();
        let from = (col.max(1) - 1) as usize;
        let to = ((col + size - 1).max(0) as usize).min(line.len());
        self.set_wint(25, col + size);
        Ok(if from < to {
            line[from..to].to_vec()
        } else {
            Vec::new()
        })
    }

    /// J183 to J185: count columns after 1W25 up to one that satisfies
    /// `wanted`, adding the count to the integer `counter`.
    fn scan(&mut self, counter: u32, wanted: impl Fn(u8) -> bool) -> Res<()> {
        let col = self.wint(25)?.max(0) as usize;
        let line = self.line().clone();
        let found = (col..line.len()).find(|&i| wanted(line[i]));
        let count = match found {
            Some(i) => i + 1 - col,
            None => line.len().saturating_sub(col),
        };
        let v = self.int(counter, "line scan")?;
        self.set_int(counter, v + count as i64);
        self.set_h5(found.is_some());
        Ok(())
    }

    pub fn print_prim(&mut self, n: u32) -> Res<Outcome> {
        match n {
            150 => {
                let head = self.pop().a;
                let lists = self.structure(head)?;
                let locals: HashMap<u32, u32> = lists
                    .iter()
                    .skip(1)
                    .enumerate()
                    .map(|(i, &l)| (l, i as u32 + 1))
                    .collect();
                for l in lists {
                    for line in self.list_lines(l, Some(&locals))? {
                        self.print(&line);
                    }
                }
            }
            151 => {
                let head = self.pop().a;
                for line in self.list_lines(head, None)? {
                    self.print(&line);
                }
            }
            152 => {
                let s = self.pop();
                let named = match self.datum(s.a) {
                    Some(d) => {
                        let t = self.mem[s.a as usize];
                        format!("{}{} {}", t.p, t.q, datum_text(d))
                    }
                    None => String::new(),
                };
                let line = format!("{:<9}{:<9}{}", "", self.name(s), named);
                self.print(&line);
            }
            153 => {
                let s = self.pop();
                let text = self.datum(s.a).map(datum_text).unwrap_or_default();
                self.print(&text);
            }
            154 => {
                self.line().fill(b' ');
                let margin = self.wint(21)?;
                self.set_wint(25, margin);
            }
            155 => {
                let text = String::from_utf8_lossy(self.line()).into_owned();
                self.print(&text);
            }
            156 | 158 => {
                let s = self.pop();
                let text = self.compact(s);
                self.enter(&text, n == 156)?;
            }
            157 | 159 => {
                let s = self.pop();
                match self.entry(s.a) {
                    Some(text) => self.enter(&text, n == 157)?,
                    None => return Err(format!("J{}: {} is not a data term", n, self.label(s.a))),
                }
            }
            160 => {
                let s = self.pop();
                let v = self.int(s.a, "J160")? + self.wint(21)?;
                self.set_wint(25, v);
            }
            161 => {
                let s = self.pop();
                let v = self.int(s.a, "J161")? + self.wint(25)?;
                self.set_wint(25, v);
            }
            162 => {
                // The formats are the machine system's (section 16.2); here
                // 1W43 = 0 is octal and anything else decimal: the cell's
                // name, then P, Q, SYMB and LINK.
                let a = self.pop().a;
                let word = self.mem[a as usize];
                let name = self.label(a);
                let text = if self.wint(43)? == 0 {
                    format!(
                        "{} {}{} {:05o} {:05o}",
                        name, word.p, word.q, word.symb, word.link
                    )
                } else {
                    format!("{} {}{} {:5} {:5}", name, word.p, word.q, word.symb, word.link)
                };
                self.enter(&text, true)?;
            }
            189 => {
                let to = self.pop().a;
                let field = self.field()?;
                let col = self.wint(21)?.max(1) as usize - 1;
                let from_col = self.wint(25)? - field.len() as i64;
                let line = self.lines.entry(to).or_insert_with(|| vec![b' '; LINE_SIZE]);
                let room = line.len().saturating_sub(col).min(field.len());
                line[col..col + room].copy_from_slice(&field[..room]);
                let size = self.wint(30)?;
                // 1W25 is left after the last column transferred.
                self.set_wint(25, from_col + room as i64);
                self.set_h5(room as i64 == size);
            }
            180 => match self.read_card(self.wint(18)?.clamp(0, 10) as u32) {
                Some(card) => {
                    let line = self.line();
                    line.fill(b' ');
                    let k = line.len().min(80);
                    line[..k].copy_from_slice(&card.cols[..k]);
                    self.set_h5(true);
                }
                None => self.set_h5(false),
            },
            181 => {
                let f = self.field()?;
                let digits: Vec<u8> = f.iter().skip(1).copied().filter(u8::is_ascii_digit).collect();
                let number = |d: &[u8]| {
                    d.iter()
                        .fold(0u32, |n, c| n.wrapping_mul(10).wrapping_add((c - b'0') as u32))
                };
                match f.first() {
                    Some(&c) if c != b' ' && !c.is_ascii_digit() => {
                        let a = self.regional(c, number(&digits))?;
                        self.push(Sym { a, q: Q_REGIONAL })?;
                        self.set_h5(true);
                    }
                    _ => {
                        let all: Vec<u8> = f.iter().copied().filter(u8::is_ascii_digit).collect();
                        if all.is_empty() {
                            self.set_h5(false);
                        } else {
                            let a = number(&all);
                            if a as usize >= self.mem.len() {
                                return Err(format!(
                                    "J181: the internal symbol {} is not an address in storage",
                                    a
                                ));
                            }
                            self.push(Sym { a, q: Q_INTERNAL })?;
                            self.set_h5(true);
                        }
                    }
                }
            }
            182 => {
                let target = self.arg(0).a;
                let f = self.field()?;
                let digits: String = f
                    .iter()
                    .filter(|c| c.is_ascii_digit())
                    .map(|&c| c as char)
                    .collect();
                let (d, found) = match self.datum(target) {
                    Some(Datum::Int(_)) => {
                        let tail = &digits[digits.len().saturating_sub(18)..];
                        (Datum::Int(tail.parse().unwrap_or(0)), !digits.is_empty())
                    }
                    Some(Datum::Octal(_)) => {
                        let tail = &digits[digits.len().saturating_sub(20)..];
                        (
                            Datum::Octal(u64::from_str_radix(tail, 8).unwrap_or(0)),
                            !digits.is_empty(),
                        )
                    }
                    _ => {
                        let f = if f.len() > 5 { &f[f.len() - 5..] } else { &f[..] };
                        let mut text = [b' '; 5];
                        text[..f.len()].copy_from_slice(f);
                        (Datum::Alpha(text), f.iter().any(|&c| c != b' '))
                    }
                };
                self.mem[target as usize] = Word::from_datum(d);
                self.set_h5(found);
            }
            183 => {
                let counter = self.arg(0).a;
                self.scan(counter, |c| c == b' ')?;
            }
            184 => {
                let counter = self.arg(0).a;
                self.scan(counter, |c| c != b' ')?;
            }
            185 => {
                let s = self.pop();
                let counter = self.arg(0).a;
                let wanted = match self.regional_name(s.a) {
                    Some((ch, _)) => ch,
                    None => b'0' + (s.a % 10) as u8,
                };
                self.scan(counter, |c| c == wanted)?;
            }
            186 => {
                let col = self.wint(25)?;
                let c = if col >= 1 {
                    self.line().get(col as usize - 1).copied().unwrap_or(b' ')
                } else {
                    b' '
                };
                if c == b' ' {
                    self.set_h5(false);
                } else {
                    let s = if c.is_ascii_digit() {
                        Sym {
                            a: (c - b'0') as u32,
                            q: Q_INTERNAL,
                        }
                    } else {
                        Sym {
                            a: self.regional(c, 0)?,
                            q: Q_REGIONAL,
                        }
                    };
                    self.push(s)?;
                    self.set_h5(true);
                }
            }
            _ => return Err(format!("J{} is not implemented", n)),
        }
        Ok(Outcome::Done)
    }
}
