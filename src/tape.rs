//! External tapes and the card punch (manual sections 13, 14 and 17).
//!
//! Unit 0 is the installation's "normal" unit: for reading, the card reader,
//! which holds the rest of the deck after the start card; for writing, the
//! punch, whose cards are printed. Units 1 to 10 are tapes: a sequence of
//! records, each a card, an end-of-file mark, a block written by J174, or a
//! storage image written by J166, with a position that reading and writing
//! move along. A unit is chosen by the integer named in W18 (read) or W19
//! (write), and the representation by W16 and W17: only IPL standard, one
//! word to a card, is here, since the compressed and binary forms are left
//! by the manual to each machine's write-up.

use crate::card::*;
use crate::interp::{Cont, Outcome};
use crate::machine::*;
use crate::word::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Record {
    Card(Card),
    EndOfFile,
    /// The control word of the block, then its cells (J174).
    Block(Vec<Word>),
    /// A storage image, for restart (J166).
    Image(Box<crate::restart::Image>),
}

#[derive(Clone, Debug, Default)]
pub struct Tape {
    pub records: Vec<Record>,
    pub pos: usize,
    /// The program has written on it, so it is saved at the end of a run.
    pub written: bool,
}

/// How a tape is kept in a file: card images, and a line for each record
/// that is not a card.
const MARK: &str = "%%IPLV ";

impl Tape {
    pub fn from_text(text: &str) -> Res<Tape> {
        let mut records = Vec::new();
        let mut lines = text.lines().enumerate().peekable();
        while let Some((i, line)) = lines.next() {
            match line.strip_prefix(MARK) {
                None => records.push(Record::Card(Card::parse(line, i + 1))),
                Some("EOF") => records.push(Record::EndOfFile),
                Some(rest) if rest.starts_with("BLOCK") => {
                    let n: usize = rest[5..]
                        .trim()
                        .parse()
                        .map_err(|_| format!("line {}: a block needs its length", i + 1))?;
                    let mut words = Vec::with_capacity(n);
                    for _ in 0..n {
                        let (j, w) = lines.next().ok_or("a block is cut short")?;
                        words.push(parse_word(w).ok_or(format!("line {}: not a word", j + 1))?);
                    }
                    records.push(Record::Block(words));
                }
                Some(rest) if rest.starts_with("IMAGE") => {
                    let n: usize = rest[5..]
                        .trim()
                        .parse()
                        .map_err(|_| format!("line {}: an image needs its length", i + 1))?;
                    let mut body = String::new();
                    for _ in 0..n {
                        let (_, l) = lines.next().ok_or("a storage image is cut short")?;
                        body.push_str(l);
                        body.push('\n');
                    }
                    records.push(Record::Image(Box::new(crate::restart::Image::from_text(&body)?)));
                }
                Some(other) => return Err(format!("line {}: unknown tape record {}", i + 1, other)),
            }
        }
        Ok(Tape {
            records,
            pos: 0,
            written: false,
        })
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for r in &self.records {
            match r {
                Record::Card(c) => {
                    out.push_str(&c.raw);
                    out.push('\n');
                }
                Record::EndOfFile => out.push_str(&format!("{}EOF\n", MARK)),
                Record::Block(words) => {
                    out.push_str(&format!("{}BLOCK {}\n", MARK, words.len()));
                    for w in words {
                        out.push_str(&format!("{} {} {} {}\n", w.p, w.q, w.symb, w.link));
                    }
                }
                Record::Image(image) => {
                    let body = image.to_text();
                    out.push_str(&format!("{}IMAGE {}\n{}", MARK, body.lines().count(), body));
                }
            }
        }
        out
    }

    fn write(&mut self, r: Record) {
        self.records.truncate(self.pos);
        self.records.push(r);
        self.pos += 1;
        self.written = true;
    }
}

fn parse_word(line: &str) -> Option<Word> {
    let v: Vec<u32> = line
        .split_whitespace()
        .map(|x| x.parse().ok())
        .collect::<Option<_>>()?;
    (v.len() == 4).then(|| Word {
        p: v[0] as u8,
        q: v[1] as u8,
        symb: v[2],
        link: v[3],
    })
}

/// A card with these fields in their columns.
pub fn punch(name: &str, sign: u8, pq: &str, rest: &str) -> String {
    let mut c = [b' '; 80];
    c[COL_NAME - 1..COL_NAME - 1 + name.len().min(5)].copy_from_slice(&name.as_bytes()[..name.len().min(5)]);
    c[COL_SIGN - 1] = sign;
    c[COL_P - 1..COL_P - 1 + pq.len().min(2)].copy_from_slice(&pq.as_bytes()[..pq.len().min(2)]);
    let r = rest.as_bytes();
    c[COL_SYMB - 1..COL_SYMB - 1 + r.len().min(30)].copy_from_slice(&r[..r.len().min(30)]);
    String::from_utf8_lossy(&c).trim_end().to_string()
}

impl Machine {
    fn unit(&self, cell: u32, what: &str) -> Res<u32> {
        let n = self.wint(cell)?;
        if !(0..=10).contains(&n) {
            return Err(format!("{}: unit {} is not 0 to 10", what, n));
        }
        Ok(n as u32)
    }

    fn mode(&self, cell: u32, what: &str) -> Res<()> {
        match self.wint(cell)? {
            0 => Ok(()),
            n => Err(format!(
                "{}: representation mode {} (1W{}): only IPL standard, mode 0, is implemented; the compressed and binary forms are machine-dependent",
                what, n, cell
            )),
        }
    }

    pub fn tape(&mut self, n: u32) -> &mut Tape {
        self.tapes.entry(n).or_default()
    }

    /// The next record on a unit, without taking it.
    fn peek(&self, unit: u32) -> Option<Record> {
        if unit == 0 {
            return self.cards.front().cloned().map(Record::Card);
        }
        let t = self.tapes.get(&unit)?;
        t.records.get(t.pos).cloned()
    }

    fn take(&mut self, unit: u32) -> Option<Record> {
        if unit == 0 {
            return self.cards.pop_front().map(Record::Card);
        }
        let t = self.tapes.get_mut(&unit)?;
        let r = t.records.get(t.pos).cloned();
        if r.is_some() {
            t.pos += 1;
        }
        r
    }

    /// The next card on a unit, for J180: None at an end of file or the
    /// end of the tape.
    pub fn read_card(&mut self, unit: u32) -> Option<Card> {
        match self.peek(unit)? {
            Record::Card(_) => match self.take(unit) {
                Some(Record::Card(c)) => Some(c),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn write_record(&mut self, unit: u32, r: Record) {
        if unit == 0 {
            // The punch: its cards are printed.
            if let Record::Card(c) = r {
                self.out.push_str(&c.raw);
                self.out.push('\n');
            }
            return;
        }
        self.tape(unit).write(r);
    }

    /// The cards of one list structure (J140 and J167): up to a blank
    /// card, an end of file, or a card that begins another structure, with
    /// a regional or internal NAME. A blank card at the start is passed over.
    fn read_structure(&mut self, unit: u32) -> Option<Vec<Card>> {
        let mut cards: Vec<Card> = Vec::new();
        loop {
            match self.peek(unit) {
                Some(Record::Card(c)) if c.is_blank() => {
                    self.take(unit);
                    if !cards.is_empty() {
                        break;
                    }
                }
                Some(Record::Card(c)) => {
                    let begins = matches!(c.name, Field::Regional(..) | Field::Internal(_));
                    if begins && !cards.is_empty() {
                        break;
                    }
                    if c.typ == 0 {
                        cards.push(c);
                    }
                    self.take(unit);
                }
                _ => break,
            }
        }
        (!cards.is_empty()).then_some(cards)
    }

    /// The cards for list structure `head` in IPL standard form (J142):
    /// each list in the order of J101, every list cell written in turn, so
    /// that only the last of each list needs a LINK. Local symbols become
    /// 9-1, 9-2, ... in that order.
    pub fn punch_structure(&self, head: u32) -> Res<Vec<String>> {
        let lists = self.structure(head)?;
        let locals: HashMap<u32, u32> = lists
            .iter()
            .skip(1)
            .enumerate()
            .map(|(i, &l)| (l, i as u32 + 1))
            .collect();
        let name = |s: Sym| -> String {
            match locals.get(&s.a) {
                Some(n) if s.q == Q_LOCAL || s.a != head => format!("9-{}", n),
                _ => match self.regional_name(s.a) {
                    Some((ch, n)) => format!("{}{}", ch as char, n),
                    None => s.a.to_string(),
                },
            }
        };
        let mut cards = Vec::new();
        for (i, &l) in lists.iter().enumerate() {
            let list_name = if i == 0 {
                name(Sym {
                    a: l,
                    q: self.sym(l).q,
                })
            } else {
                format!("9-{}", locals[&l])
            };
            let mut cells = vec![l];
            if !self.mem[l as usize].is_data() {
                cells.extend(self.cells_after(l)?);
            }
            for (k, &c) in cells.iter().enumerate() {
                let w = self.mem[c as usize];
                let card_name = if k == 0 { list_name.as_str() } else { "" };
                let last = k == cells.len() - 1;
                cards.push(match w.datum() {
                    Some(d) => {
                        let (sign, rest) = data_fields(d);
                        punch(card_name, sign, &format!("{}1", w.p), &rest)
                    }
                    None => {
                        let symb = name(w.sym());
                        punch(
                            card_name,
                            b' ',
                            "",
                            &format!("{:<5} {}", symb, if last { "0" } else { "" }),
                        )
                    }
                });
            }
        }
        Ok(cards)
    }

    /// The storage image that is the next record on a tape (section 20).
    pub(crate) fn read_image(&mut self, unit: u32) -> Res<crate::restart::Image> {
        if !self.tapes.contains_key(&unit) {
            return Err(format!("there is no tape on unit {} to restart from", unit));
        }
        // A restart tape is read from its beginning: in a later run it is
        // mounted afresh, and in this one it is wound back.
        self.tape(unit).pos = 0;
        match self.take(unit) {
            Some(Record::Image(image)) => Ok(*image),
            Some(_) => Err(format!(
                "the next record on unit {} is not a storage image from J166",
                unit
            )),
            None => Err(format!("unit {} has no more records", unit)),
        }
    }

    /// J165 (section 19): load from the unit 1W18 names. The cards of a tape
    /// are put in the reader for the loader, and those it does not read go
    /// back, so that the tape is left after the start card.
    pub(crate) fn load_from_unit(&mut self) -> Res<Option<crate::loader::Start>> {
        let unit = self.unit(18, "J165")?;
        if unit == 0 {
            return self.load();
        }
        let tape = self.tape(unit);
        let from = tape.pos;
        let mut cards = std::collections::VecDeque::new();
        while let Some(Record::Card(c)) = tape.records.get(tape.pos + cards.len()) {
            cards.push_back(c.clone());
        }
        let total = cards.len();
        let reader = std::mem::replace(&mut self.cards, cards);
        let result = self.load();
        let left = std::mem::replace(&mut self.cards, reader).len();
        // After a restart the tape positions are as they were saved: none.
        if !matches!(result, Ok(Some(crate::loader::Start::Restart))) {
            self.tape(unit).pos = from + (total - left);
        }
        result
    }

    /// J140 to J146, J167, J173, J174 and J176.
    pub fn tape_prim(&mut self, n: u32) -> Res<Outcome> {
        let what = format!("J{}", n);
        match n {
            140 | 167 => {
                self.mode(16, &what)?;
                let unit = self.unit(18, &what)?;
                match self.read_structure(unit) {
                    None => self.set_h5(false),
                    Some(cards) if n == 140 => {
                        let loaded = self.load_structure(&cards)?;
                        let q = if self.is_regional(loaded) {
                            Q_REGIONAL
                        } else {
                            Q_INTERNAL
                        };
                        self.push(Sym { a: loaded, q })?;
                        self.set_h5(true);
                    }
                    Some(_) => self.set_h5(true),
                }
            }
            // The console is the machine's (section 14): there is none here,
            // so there is never an input.
            141 => self.set_h5(false),
            142 => {
                self.mode(17, &what)?;
                let head = self.pop().a;
                let unit = self.unit(19, &what)?;
                for (i, text) in self.punch_structure(head)?.into_iter().enumerate() {
                    self.write_record(unit, Record::Card(Card::parse(&text, i + 1)));
                }
            }
            143 => {
                let u = self.pop().a;
                match self.int(u, &what)? {
                    // The card reader cannot be rewound.
                    0 => {}
                    unit @ 1..=10 => self.tape(unit as u32).pos = 0,
                    unit => return Err(format!("{}: unit {} is not 0 to 10", what, unit)),
                }
            }
            144 => {
                let unit = self.unit(18, &what)?;
                while let Some(r) = self.take(unit) {
                    if matches!(r, Record::EndOfFile) {
                        break;
                    }
                }
            }
            145 => {
                let unit = self.unit(19, &what)?;
                self.write_record(unit, Record::EndOfFile);
            }
            146 => {
                let unit = self.unit(19, &what)?;
                self.write_record(unit, Record::Card(Card::parse("", 0)));
            }
            173 | 174 => {
                let block = self.arg(0).a;
                let control = self.mem[block as usize];
                if control.p != 7 || control.q != 7 {
                    self.pop();
                    self.set_h5(false);
                    self.warnings.push(format!(
                        "{}: {} is not a block control word",
                        what,
                        self.label(block)
                    ));
                    return self.trap_now(H0);
                }
                let unit = self.unit(19, &what)?;
                let (origin, size) = (control.symb, control.link);
                if n == 174 {
                    self.pop();
                    let mut words = vec![control];
                    words.extend((origin..origin + size).map(|a| self.mem[a as usize]));
                    self.write_record(unit, Record::Block(words));
                    self.set_h5(true);
                } else {
                    match self.take(unit) {
                        Some(Record::Block(words)) if words.first() == Some(&control) => {
                            self.pop();
                            for (k, w) in words[1..].iter().enumerate().take(size as usize) {
                                self.mem[(origin + k as u32) as usize] = *w;
                            }
                            self.set_h5(true);
                        }
                        Some(Record::Block(_)) => {
                            // The first word does not match: (0) stays in H0.
                            self.set_h5(false);
                            return self.trap_now(j(173));
                        }
                        _ => {
                            self.pop();
                            self.set_h5(false);
                        }
                    }
                }
            }
            176 => {
                let count = self.pop().a;
                let k = self.int(count, &what)?;
                let unit = self.unit(19, &what)?;
                let t = self.tape(unit);
                t.pos = (t.pos as i64 + k).clamp(0, t.records.len() as i64) as usize;
            }
            _ => return Err(format!("J{} is not implemented", n)),
        }
        Ok(Outcome::Done)
    }

    /// A trap taken by a basic process, as J170 would take it.
    fn trap_now(&mut self, symbol: u32) -> Res<Outcome> {
        let sym = self.sym(symbol);
        match self.trap_action(sym)? {
            Some(action) => Ok(Outcome::Call(action, Cont::Trap { space: false })),
            None => Ok(Outcome::Done),
        }
    }
}

/// Fig. 2 (page 140): the columns a decimal integer's digits go in (52-55
/// and 58-61), and an octal number's (51-55 and 57-61). Column 56 is never
/// used.
const INT_COLUMNS: [usize; 8] = [52, 53, 54, 55, 58, 59, 60, 61];
const OCTAL_COLUMNS: [usize; 10] = [51, 52, 53, 54, 55, 57, 58, 59, 60, 61];

/// Columns 51 to 61 with `digits` set right-justified into `columns`. A
/// number with more digits than the figure has room for (it shows eight
/// decimal and ten octal) runs across all eleven columns, which is ours:
/// the loader reads the digits of 51-61 in order either way.
fn spread(digits: &str, columns: &[usize]) -> String {
    if digits.len() > columns.len() {
        return format!("{:>11}", digits);
    }
    let mut c = [b' '; 11];
    for (&col, d) in columns.iter().rev().zip(digits.bytes().rev()) {
        c[col - 51] = d;
    }
    String::from_utf8(c.to_vec()).unwrap()
}

/// SIGN and the columns from SYMB on of a data term's card (Fig. 2).
fn data_fields(d: Datum) -> (u8, String) {
    match d {
        Datum::Int(MINUS_ZERO) => (b'-', spread("0", &INT_COLUMNS)),
        Datum::Int(i) => (
            if i < 0 { b'-' } else { b' ' },
            spread(&i.unsigned_abs().to_string(), &INT_COLUMNS),
        ),
        Datum::Octal(o) => {
            let v = o as i64;
            (
                if v < 0 { b'-' } else { b' ' },
                spread(&format!("{:o}", v.unsigned_abs()), &OCTAL_COLUMNS),
            )
        }
        Datum::Alpha(a) => (b' ', String::from_utf8_lossy(&a).into_owned()),
        Datum::Float(f) => {
            if f == 0.0 {
                return (b' ', "00000 0   00".to_string());
            }
            let mut e = f.abs().log10().floor() as i32 + 1;
            let mut m = (f.abs() / 10f64.powi(e) * 1e6).round() as i64;
            if m >= 1_000_000 {
                m /= 10;
                e += 1;
            }
            // Fig. 2: the fraction's first five digits in SYMB (51-55),
            // its sixth in column 57 past the unused column 56, the
            // exponent's sign in 59 and its digits in 60 and 61.
            let d = format!("{:06}", m);
            (
                if f < 0.0 { b'-' } else { b' ' },
                format!("{} {} {}{:02}", &d[..5], &d[5..], if e < 0 { '-' } else { ' ' }, e.abs()),
            )
        }
    }
}
