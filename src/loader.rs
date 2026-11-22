//! Initial and in-process loading (manual sections 18 and 19).
//!
//! Loading is the one-pass symbolic assembly the manual describes: region
//! cards first, then sets of routines or data, each behind a header card,
//! and last a start card naming the routine to execute. The cards after the
//! start card stay in the reader for the program itself to read.

use crate::card::*;
use crate::machine::*;
use crate::word::*;
use std::collections::HashMap;

/// What a header card says about the set that follows it (section 18.5).
/// What a deck says to do after loading.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Start {
    /// Interpret the routine named on the start card.
    At(u32),
    /// Go on from the J166 that wrote the storage image just reloaded.
    Restart,
}

#[derive(Clone, Copy)]
pub(crate) struct Header {
    data: bool,
    absolute: bool,
    skip: bool,
    /// TYPE 6 or 7: data list structures go to the fast or slow auxiliary
    /// store once loaded (section 18.5). Routines stay in main storage.
    aux: Option<usize>,
}

impl Machine {
    /// Put a deck in the card reader.
    pub fn read_deck(&mut self, text: &str) {
        let from = self.cards.len();
        for (i, l) in text.lines().enumerate() {
            self.cards.push_back(Card::parse(l, from + i + 1));
        }
    }

    /// Load sets until a start card, or a restart card. Returns what to do
    /// next, or None if the cards run out first.
    pub fn load(&mut self) -> Res<Option<Start>> {
        let mut header: Option<Header> = None;
        let mut set: Vec<Card> = Vec::new();
        while let Some(card) = self.cards.pop_front() {
            // A header ends the set before it.
            if (5..=8).contains(&card.typ) {
                if let Some(h) = header {
                    self.load_header_set(&set, h)?;
                }
                set.clear();
            }
            let in_set = header.is_some() && matches!(card.typ, 0 | 1 | 4);
            if let (Some(l), false) = (self.listing.as_mut(), in_set) {
                l.push_str(&format!("{:>25}{}\n", "", card.raw));
            }
            match card.typ {
                0 => {
                    if card.is_blank() {
                        continue;
                    }
                    if header.is_none() {
                        return Err(at(&card, "a word before any header card (TYPE 5)"));
                    }
                    set.push(card);
                }
                // Comment cards inside a set are kept for the listing.
                1 | 4 if header.is_some() => set.push(card),
                1 | 4 | 9 => {}
                2 => {
                    let ch = match card.name {
                        Field::Regional(ch, _) => ch,
                        _ => return Err(at(&card, "a region card needs the region's letter in NAME")),
                    };
                    let size = card.count(COL_LINK).unwrap_or(1);
                    let origin = self.origin_of(card.symb).map_err(|e| at(&card, &e))?;
                    if self.region(ch).is_some() {
                        self.warnings.push(at(
                            &card,
                            &format!("region {} is already defined; this card is ignored", ch as char),
                        ));
                    } else {
                        self.define_region(ch, size, origin).map_err(|e| at(&card, &e))?;
                    }
                }
                3 => self.block_card(&card)?,
                // Restart mode (section 20): storage is reloaded from the
                // tape named in SYMB; the cards after this one are left for
                // the program, which normally reads them with J165.
                5 if card.p == 4 => {
                    if self.w34_preserved {
                        self.restore(w(34));
                        self.w34_preserved = false;
                    }
                    let unit = match card.symb {
                        Field::Internal(n) if (1..=10).contains(&n) => n,
                        _ => return Err(at(&card, "a restart card needs the tape, 1 to 10, in SYMB")),
                    };
                    let image = self.read_image(unit).map_err(|e| at(&card, &e))?;
                    self.restore_image(&image);
                    return Ok(Some(Start::Restart));
                }
                5..=8 => {
                    if let Field::Regional(ch, n) = card.symb {
                        if self.w34_preserved {
                            self.restore(w(34));
                            self.w34_preserved = false;
                        }
                        let start = self.regional(ch, n)?;
                        let free = self.available();
                        if let Some(l) = self.listing.as_mut() {
                            l.push_str(&format!(
                                "END OF LOADING. PROGRAM STARTS AT {}{}\n",
                                ch as char, n
                            ));
                            l.push_str(&format!("NUMBER OF CELLS ON AVAILABLE SPACE={}\n", free));
                        }
                        return Ok(Some(Start::At(start)));
                    }
                    // NAME = a block made into a list: load the set into its
                    // cells, until the next header (section 18.5).
                    if self.w34_preserved {
                        self.restore(w(34));
                        self.w34_preserved = false;
                    }
                    if card.typ == 5 {
                        if let Field::Regional(ch, n) = card.name {
                            let block = self.regional(ch, n)?;
                            let list = self.mem[block as usize].symb;
                            self.preserve(w(34))?;
                            self.mem[w(34) as usize].symb = list;
                            self.w34_preserved = true;
                        }
                    }
                    header = Some(Header {
                        data: card.q % 2 == 1,
                        absolute: card.q >= 4,
                        // P = 4 on Types 6 to 8 is restart mode too, which only a
                        // Type-5 card can start: the set is passed over.
                        skip: card.typ == 8 || card.p == 4,
                        aux: match card.typ {
                            6 => Some(crate::auxiliary::FAST),
                            7 => Some(crate::auxiliary::SLOW),
                            _ => None,
                        },
                    });
                    if card.q == 2 || card.q == 3 {
                        self.internals.clear();
                    }
                }
                _ => unreachable!(),
            }
        }
        if let Some(h) = header {
            self.load_header_set(&set, h)?;
        }
        Ok(None)
    }

    /// A set under its header: loaded, and for a set of data under a Type-6
    /// or Type-7 header, each list structure then filed in auxiliary.
    fn load_header_set(&mut self, set: &[Card], h: Header) -> Res<()> {
        let internals = self.internals.clone();
        self.load_set(set, h)?;
        let (Some(store), true, false) = (h.aux, h.data, h.skip) else {
            return Ok(());
        };
        for card in set.iter().filter(|c| c.typ == 0) {
            let head = match card.name {
                Field::Regional(ch, n) => self.lookup(ch, n),
                Field::Internal(n) if h.absolute => Some(n),
                Field::Internal(n) => self.internals.get(&n).or(internals.get(&n)).copied(),
                _ => None,
            };
            if let Some(head) = head {
                self.file_structure(head, store).map_err(|e| at(card, &e))?;
            }
        }
        Ok(())
    }

    /// A list structure read by J140 (section 14): its cards are loaded as
    /// data, into cells from 1W34, with internal symbols taken to be
    /// addresses that already exist. Returns the name of its head.
    pub(crate) fn load_structure(&mut self, cards: &[Card]) -> Res<u32> {
        let h = Header {
            data: true,
            absolute: true,
            skip: false,
            aux: None,
        };
        let loaded = self.load_set(cards, h)?;
        loaded
            .first()
            .copied()
            .ok_or_else(|| "an empty list structure".to_string())
    }

    /// The origin in SYMB of a Type-2 or Type-3 card: an absolute address,
    /// or a regional symbol already defined.
    fn origin_of(&mut self, f: Field) -> Res<Option<u32>> {
        Ok(match f {
            Field::Blank => None,
            Field::Internal(n) => Some(n),
            Field::Regional(ch, n) => match self.lookup(ch, n) {
                Some(a) => Some(a),
                None => {
                    return Err(format!(
                        "{}{} is not defined yet, so cannot be an origin",
                        ch as char, n
                    ))
                }
            },
            Field::Local(_) => return Err("a local symbol cannot be an origin".to_string()),
        })
    }

    /// A Type-3 card (section 18.3).
    fn block_card(&mut self, card: &Card) -> Res<()> {
        // A print line's name may be in SYMB with NAME blank: Simon's
        // Heuristic Coder (listing of 16 July 1961) has it there, perhaps
        // as the first edition of the manual had it. That edition is not
        // available to check (sources/README.md), so this is accepted for
        // the decks' sake, not on the first edition's authority.
        let name = if card.name == Field::Blank && card.q == 1 {
            card.symb
        } else {
            card.name
        };
        let name = match name {
            Field::Regional(ch, n) => Some(self.regional(ch, n).map_err(|e| at(card, &e))?),
            Field::Blank => None,
            _ => return Err(at(card, "the name of a block must be a regional symbol")),
        };
        let size = card.count(COL_LINK).unwrap_or(0);
        match card.q {
            // Reserve regional symbols: SYMB is the first, LINK how many.
            0 => {
                let first = match card.symb {
                    Field::Regional(ch, n) => self.lookup(ch, n),
                    _ => None,
                };
                let Some(first) = first else {
                    return Err(at(
                        card,
                        "reserving regional symbols needs a regional symbol of a defined region in SYMB",
                    ));
                };
                self.reserve_regionals(first, size.max(1));
                if let Some(n) = name {
                    self.mem[n as usize] = Word {
                        p: 7,
                        q: 7,
                        symb: first,
                        link: size.max(1),
                    };
                }
            }
            1 => {
                let Some(a) = name else {
                    return Err(at(card, "a print line needs a regional name"));
                };
                let words = if size > 0 { size } else { (LINE_SIZE / 6) as u32 };
                // "These words are taken from available storage, not from
                // the region" (section 18.3): they are reserved as a block,
                // so that a line uses the space it would have used. The
                // characters themselves are kept beside the machine, in
                // `Machine::lines`, six to the word as on the 704 and 7090.
                let origin = self.reserve_block(None, None, words).map_err(|e| at(card, &e))?;
                let mut text = vec![b' '; words as usize * 6];
                if card.p != 0 {
                    if let Some(next) = self.cards.pop_front() {
                        let n = text.len().min(80);
                        text[..n].copy_from_slice(&next.cols[..n]);
                    }
                }
                self.mem[a as usize] = Word {
                    p: 7,
                    q: 7,
                    symb: origin,
                    link: words,
                };
                self.lines.insert(a, text);
            }
            2 => {
                if size == 0 {
                    return Err(at(card, "a block needs its size in LINK"));
                }
                let origin = self.origin_of(card.symb).map_err(|e| at(card, &e))?;
                self.reserve_block(name, origin, size).map_err(|e| at(card, &e))?;
            }
            // The buffer for routines in auxiliary storage: here they are
            // kept in main storage, so it needs no room.
            3 => {}
            // How much available space there is to be.
            4 => {
                if size > 0 {
                    self.space_limit = Some(size);
                }
                if let Some(n) = name {
                    self.mem[n as usize] = Word {
                        p: 7,
                        q: 7,
                        symb: 0,
                        link: size,
                    };
                }
            }
            q => {
                return Err(at(
                    card,
                    &format!("a Type-3 card with Q = {} reserves nothing", q),
                ))
            }
        }
        Ok(())
    }

    /// Assemble one set. Cells are taken from available space in the order
    /// the original loader took them, which the addresses in the manual's
    /// assembly listing show: a card's own cell, then the next cell if the
    /// card's LINK or SYMB is blank, then a cell for each local symbol met
    /// for the first time.
    fn load_set(&mut self, set: &[Card], h: Header) -> Res<Vec<u32>> {
        let mut loaded = Vec::new();
        if h.skip {
            return Ok(loaded);
        }
        let cards = set;
        let set: Vec<&Card> = cards.iter().filter(|c| c.typ == 0).collect();
        let mut comments = cards.iter().filter(|c| c.typ != 0).peekable();
        let mut locals = Locals::default();
        // The cell a blank SYMB or LINK has promised to the next card.
        let mut promised: Option<u32> = None;
        let mut title = String::from("?");
        let mut offset = 0;
        for (i, &card) in set.iter().enumerate() {
            while let Some(c) = comments.next_if(|c| c.line < card.line) {
                if let Some(l) = self.listing.as_mut() {
                    l.push_str(&format!("{:>25}{}\n", "", c.raw));
                }
            }
            let fail = |e: String| at(card, &e);
            // A regional or internal NAME begins a list structure, and the
            // local symbols start afresh (section 2.9).
            let addr = match card.name {
                Field::Regional(ch, n) => {
                    locals.close(self);
                    title = format!("{}{}", ch as char, n);
                    offset = 0;
                    self.regional(ch, n).map_err(fail)?
                }
                Field::Internal(n) => {
                    if !h.absolute {
                        locals.close(self);
                        title = n.to_string();
                        offset = 0;
                    }
                    self.internal(n, h.absolute).map_err(fail)?
                }
                Field::Local(n) => locals.define(self, n).map_err(fail)?,
                Field::Blank => match promised {
                    Some(a) => a,
                    None => self.load_alloc().map_err(fail)?,
                },
            };
            promised = None;
            loaded.push(addr);
            self.origins.insert(
                addr,
                Origin {
                    structure: title.clone(),
                    local: if let Field::Local(n) = card.name {
                        Some(n)
                    } else {
                        None
                    },
                    offset,
                    comment: card.comment(),
                    line: card.line,
                },
            );
            offset += 1;

            let as_data = h.data || card.sign == b'+' || card.sign == b'-';
            let word = if as_data && card.q == 1 {
                data_term(card).map_err(fail)?
            } else {
                // A blank SYMB or LINK means the next cell, unless the next
                // line begins another list structure (section 1.6).
                let mut next = None;
                if card.link == Field::Blank || card.symb == Field::Blank {
                    next = Some(match set.get(i + 1).map(|c| c.name) {
                        Some(Field::Blank) => {
                            let a = self.load_alloc().map_err(fail)?;
                            promised = Some(a);
                            Sym { a, q: Q_LOCAL }
                        }
                        Some(Field::Local(n)) => Sym {
                            a: locals.refer(self, n, card).map_err(fail)?,
                            q: Q_LOCAL,
                        },
                        _ => Sym { a: 0, q: Q_INTERNAL },
                    });
                }
                let mut resolve = |m: &mut Machine, f: Field| -> Res<Sym> {
                    Ok(match f {
                        Field::Blank => next.unwrap(),
                        Field::Regional(ch, n) => Sym {
                            a: m.regional(ch, n)?,
                            q: Q_REGIONAL,
                        },
                        Field::Internal(n) => Sym {
                            a: m.internal(n, h.absolute)?,
                            q: Q_INTERNAL,
                        },
                        Field::Local(n) => Sym {
                            a: locals.refer(m, n, card)?,
                            q: Q_LOCAL,
                        },
                    })
                };
                let s = resolve(self, card.symb).map_err(fail)?;
                let l = resolve(self, card.link).map_err(fail)?;
                if as_data {
                    Word {
                        p: 0,
                        q: s.q,
                        symb: s.a,
                        link: l.a,
                    }
                } else {
                    Word {
                        p: card.p,
                        q: card.q,
                        symb: s.a,
                        link: l.a,
                    }
                }
            };
            self.mem[addr as usize] = word;
            if let Some(l) = self.listing.as_mut() {
                let text = match word.datum() {
                    Some(d) if as_data => format!("{} {} {:<11}", word.p, word.q, datum_text(d)),
                    _ => format!("{} {} {:>5} {:>5}", word.p, word.q, word.symb, word.link),
                };
                l.push_str(&format!("{:>6}  {:<15}  {}\n", addr, text, card.raw));
            }
        }
        for c in comments {
            if let Some(l) = self.listing.as_mut() {
                l.push_str(&format!("{:>25}{}\n", "", c.raw));
            }
        }
        locals.close(self);
        Ok(loaded)
    }

    /// An internal symbol on a card. 0 to 9 are always themselves; the rest
    /// are pure symbolics given cells from available space, unless the
    /// header says they are machine addresses (section 18.5).
    fn internal(&mut self, n: u32, absolute: bool) -> Res<u32> {
        if n <= 9 {
            return Ok(n);
        }
        if absolute {
            if n as usize >= self.mem.len() {
                return Err(format!("the internal symbol {} is not an address in storage", n));
            }
            return Ok(n);
        }
        if let Some(&a) = self.internals.get(&n) {
            return Ok(a);
        }
        let a = self.load_alloc()?;
        self.internals.insert(n, a);
        Ok(a)
    }
}

/// The local symbols of the list structure being loaded (section 18.9).
#[derive(Default)]
struct Locals {
    cells: HashMap<u32, u32>,
    /// Those used in SYMB or LINK and not yet met as a NAME, with the card
    /// that used them first.
    undefined: Vec<(u32, String)>,
}

impl Locals {
    fn define(&mut self, m: &mut Machine, n: u32) -> Res<u32> {
        self.undefined.retain(|u| u.0 != n);
        match self.cells.get(&n) {
            Some(&a) => Ok(a),
            None => {
                let a = m.load_alloc()?;
                self.cells.insert(n, a);
                Ok(a)
            }
        }
    }

    fn refer(&mut self, m: &mut Machine, n: u32, card: &Card) -> Res<u32> {
        if let Some(&a) = self.cells.get(&n) {
            return Ok(a);
        }
        let a = m.load_alloc()?;
        self.cells.insert(n, a);
        self.undefined.push((
            n,
            at(
                card,
                &format!("the local symbol 9-{} is used but never defined", n),
            ),
        ));
        Ok(a)
    }

    fn close(&mut self, m: &mut Machine) {
        m.warnings.extend(self.undefined.drain(..).map(|u| u.1));
        self.cells.clear();
    }
}

fn at(card: &Card, msg: &str) -> String {
    format!("card {}: {}\n    {}", card.line, msg, card.raw)
}

/// The formats of Fig. 2.
fn data_term(card: &Card) -> Res<Word> {
    let negative = card.sign == b'-';
    Ok(Word::from_datum(match card.p {
        0 => {
            let d = card.digits();
            let v: i64 = if d.is_empty() {
                0
            } else {
                d.parse().map_err(|_| "the integer is too large".to_string())?
            };
            Datum::Int(match (negative, v) {
                // A minus sign on zero punches minus zero (J123).
                (true, 0) => MINUS_ZERO,
                (true, v) => -v,
                (false, v) => v,
            })
        }
        1 => {
            // .ddddd in SYMB, the exponent at the right of LINK, and any
            // character in column 59 for a negative exponent.
            let m: String = card
                .text(COL_SYMB, COL_SYMB + 5)
                .iter()
                .filter(|c| c.is_ascii_digit())
                .map(|&c| c as char)
                .collect();
            let e: String = card
                .text(60, 61)
                .iter()
                .filter(|c| c.is_ascii_digit())
                .map(|&c| c as char)
                .collect();
            let mut v: f64 = if m.is_empty() {
                0.0
            } else {
                format!("0.{}", m).parse().unwrap_or(0.0)
            };
            let mut e: i32 = e.parse().unwrap_or(0);
            if card.cols[58] != b' ' {
                e = -e;
            }
            v *= 10f64.powi(e);
            Datum::Float(if negative { -v } else { v })
        }
        2 => {
            let mut a = [b' '; 5];
            a.copy_from_slice(card.text(COL_SYMB, COL_SYMB + 4));
            Datum::Alpha(a)
        }
        3 => {
            // A sign in column 48, as for the other numbers: the 704 kept
            // a sign beside its 35 bits.
            let v = i64::from_str_radix(&card.digits(), 8).unwrap_or(0);
            Datum::Octal((if negative { -v } else { v }) as u64)
        }
        p => return Err(format!("data terms of type P = {} are not defined", p)),
    }))
}
