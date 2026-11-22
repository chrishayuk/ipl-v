//! Reading a deck without running it: for `iplv check`, `iplv format` and
//! the language server.
//!
//! A deck is cards in fixed columns, so everything here is by column: what
//! is wrong with a card and where, what the thing under a position means,
//! where a symbol is defined and used, and how each stretch of a card
//! should be coloured. Lines are counted from 0 and columns from 0, as
//! editors count them, except in `Diag`, whose line is from 1.

use crate::card::*;
use crate::names;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Diag {
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub error: bool,
    pub message: String,
}

/// Line, first column, column after the last.
pub type Place = (usize, usize, usize);

/// A symbol, as far as a deck can tell symbols apart: a local symbol
/// belongs to the list structure it is written in.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Regional(u8, u32),
    Local(usize, u32),
    Internal(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Before the first header, and the cards that are not words.
    None,
    Routines,
    Data,
    /// Behind the last start card: for the program to read.
    Input,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    Name,
    Symb,
    Link,
}

#[derive(Clone, Debug)]
pub struct Occurrence {
    pub key: Key,
    pub place: Place,
    pub slot: Slot,
    pub defines: bool,
    /// The interpreter will be sent here: executed, branched to, or
    /// reached as a LINK.
    pub control: bool,
}

pub struct Line {
    pub card: Card,
    pub mode: Mode,
    /// A word that is data: in a data set, or marked with a sign.
    pub data: bool,
    /// A data term: Q = 1 in data.
    pub term: bool,
    pub structure: usize,
    /// How long the line is, in characters.
    pub width: usize,
}

pub struct Deck {
    pub lines: Vec<Line>,
    pub occurrences: Vec<Occurrence>,
    /// Region character to size and the line of its Type-2 card.
    pub regions: HashMap<u8, (u32, usize)>,
    pub start: Option<usize>,
}

pub fn show(key: Key) -> String {
    match key {
        Key::Regional(ch, n) => format!("{}{}", ch as char, n),
        Key::Local(_, n) => format!("9-{}", n),
        Key::Internal(n) => n.to_string(),
    }
}

/// The part of a five-column field that is not blank.
fn extent(card: &Card, col: usize) -> (usize, usize) {
    let f = card.text(col, col + 4);
    let first = f.iter().position(|&c| c != b' ').unwrap_or(0);
    let last = f.iter().rposition(|&c| c != b' ').map(|i| i + 1).unwrap_or(5);
    (col - 1 + first, col - 1 + last)
}

fn is_start(card: &Card) -> bool {
    (5..=8).contains(&card.typ) && matches!(card.symb, Field::Regional(..))
}

pub fn analyse(text: &str) -> Deck {
    let cards: Vec<(Card, usize)> = text
        .lines()
        .enumerate()
        .map(|(i, l)| (Card::parse(l, i + 1), l.chars().count()))
        .collect();
    let start = cards.iter().rposition(|(c, _)| is_start(c));
    let mut deck = Deck {
        lines: Vec::new(),
        occurrences: Vec::new(),
        regions: HashMap::new(),
        start,
    };
    let (mut mode, mut structure) = (Mode::None, 0);
    for (i, (card, width)) in cards.into_iter().enumerate() {
        let mut line = Line {
            card,
            mode: Mode::None,
            data: false,
            term: false,
            structure,
            width,
        };
        let card = &line.card;
        let mut note = |key: Option<Key>, slot: Slot, col: usize, defines: bool, control: bool| {
            if let Some(key) = key {
                let (a, b) = extent(card, col);
                deck.occurrences.push(Occurrence {
                    key,
                    place: (i, a, b),
                    slot,
                    defines,
                    control,
                });
            }
        };
        let key = |f: Field, structure: usize| match f {
            Field::Regional(ch, n) => Some(Key::Regional(ch, n)),
            Field::Local(n) => Some(Key::Local(structure, n)),
            Field::Internal(n) if n > 9 => Some(Key::Internal(n)),
            _ => None,
        };
        if start.is_some_and(|s| i > s) {
            line.mode = Mode::Input;
        } else {
            match card.typ {
                0 if !card.is_blank() && mode != Mode::None => {
                    if matches!(card.name, Field::Regional(..) | Field::Internal(_)) {
                        structure += 1;
                    }
                    line.structure = structure;
                    line.mode = mode;
                    line.data = mode == Mode::Data || card.sign == b'+' || card.sign == b'-';
                    line.term = line.data && card.q == 1;
                    note(key(card.name, structure), Slot::Name, COL_NAME, true, false);
                    if !line.term {
                        let routine = !line.data;
                        let executes = routine && matches!(card.p, 0 | 7) && matches!(card.q, 0 | 3 | 4);
                        note(key(card.symb, structure), Slot::Symb, COL_SYMB, false, executes);
                        note(key(card.link, structure), Slot::Link, COL_LINK, false, routine);
                    }
                }
                2 => {
                    if let Field::Regional(ch, _) = card.name {
                        deck.regions
                            .entry(ch)
                            .or_insert((card.count(COL_LINK).unwrap_or(1).max(1), i));
                    }
                }
                3 => {
                    let name = if card.name == Field::Blank {
                        card.symb
                    } else {
                        card.name
                    };
                    let col = if card.name == Field::Blank {
                        COL_SYMB
                    } else {
                        COL_NAME
                    };
                    note(key(name, 0), Slot::Name, col, true, false);
                }
                5..=8 => {
                    structure += 1;
                    if is_start(card) {
                        note(key(card.symb, 0), Slot::Symb, COL_SYMB, false, true);
                    } else {
                        mode = if card.q % 2 == 1 {
                            Mode::Data
                        } else {
                            Mode::Routines
                        };
                    }
                }
                _ => {}
            }
        }
        deck.lines.push(line);
    }
    deck
}

impl Deck {
    fn definitions(&self) -> HashMap<Key, Vec<Place>> {
        let mut map: HashMap<Key, Vec<Place>> = HashMap::new();
        for o in self.occurrences.iter().filter(|o| o.defines) {
            map.entry(o.key).or_default().push(o.place);
        }
        map
    }

    pub fn at(&self, line: usize, col: usize) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .find(|o| o.place.0 == line && col >= o.place.1 && col <= o.place.2)
    }

    /// The first line of each list structure that a regional symbol
    /// names, with the last line that belongs to it.
    pub fn structures(&self) -> Vec<(Key, usize, usize)> {
        let mut out: Vec<(Key, usize, usize)> = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            let word = line.card.typ == 0 && matches!(line.mode, Mode::Routines | Mode::Data);
            match line.card.name {
                Field::Regional(ch, n) if word => out.push((Key::Regional(ch, n), i, i)),
                _ if word || matches!(line.card.typ, 1 | 4) => {
                    if let Some(last) = out.last_mut() {
                        if word && self.lines[last.1].structure == line.structure {
                            last.2 = i;
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }
}

pub fn check(text: &str) -> Vec<Diag> {
    let deck = analyse(text);
    let defined = deck.definitions();
    let mut out = Vec::new();
    let mut say = |line: usize, start: usize, end: usize, error: bool, message: String| {
        out.push(Diag {
            line: line + 1,
            start,
            end: end.max(start + 1),
            error,
            message,
        });
    };
    let mut headers = 0;
    for (i, line) in deck.lines.iter().enumerate() {
        let card = &line.card;
        if line.mode == Mode::Input {
            continue;
        }
        if line.width > 80
            && text
                .lines()
                .nth(i)
                .is_some_and(|l| l.chars().skip(80).any(|c| c != ' '))
        {
            say(
                i,
                80,
                line.width,
                false,
                "a card has 80 columns: the rest is not read".to_string(),
            );
        }
        let t = card.cols[COL_TYPE - 1];
        if t != b' ' && !t.is_ascii_digit() {
            say(
                i,
                40,
                41,
                true,
                format!(
                    "column 41 is TYPE, and `{}` is not a type of card: comments end at column 40",
                    t as char
                ),
            );
            continue;
        }
        match card.typ {
            0 if card.is_blank() => {}
            0 if line.mode == Mode::None || headers == 0 => {
                say(
                    i,
                    0,
                    line.width,
                    true,
                    "a word before any header card (TYPE 5)".to_string(),
                );
            }
            0 => {
                for (col, what) in [(COL_P, "P"), (COL_Q, "Q")] {
                    let c = card.cols[col - 1];
                    if c != b' ' && !(b'0'..=b'7').contains(&c) {
                        say(
                            i,
                            col - 1,
                            col,
                            true,
                            format!("{} is a digit from 0 to 7, or blank", what),
                        );
                    }
                }
                if line.term && card.p > 3 {
                    say(
                        i,
                        COL_P - 1,
                        COL_P,
                        true,
                        format!("data terms are defined for P = 0, 1, 2 and 3, not {}", card.p),
                    );
                }
                if !line.term {
                    for col in [42, 56] {
                        if card.cols[col - 1] != b' ' {
                            say(
                                i,
                                col - 1,
                                col,
                                false,
                                format!("column {} belongs to no field, and is not read", col),
                            );
                        }
                    }
                }
                if line.data
                    && !line.term
                    && (card.cols[COL_P - 1] != b' ' || !matches!(card.cols[COL_Q - 1], b' ' | b'0'))
                {
                    say(
                        i,
                        COL_P - 1,
                        COL_Q,
                        false,
                        "P and Q mean nothing in a data word that is not a data term".to_string(),
                    );
                }
            }
            2 => {
                match card.name {
                    Field::Regional(ch, _) => {
                        if deck.regions.get(&ch).is_some_and(|r| r.1 != i) {
                            say(
                                i,
                                42,
                                47,
                                false,
                                format!("region {} is already defined; this card is ignored", ch as char),
                            );
                        } else if matches!(ch, b'H' | b'J' | b'W') {
                            say(
                                i,
                                42,
                                47,
                                true,
                                format!("region {} belongs to the system", ch as char),
                            );
                        }
                    }
                    _ => say(
                        i,
                        42,
                        47,
                        true,
                        "a region card needs the region's letter in NAME".to_string(),
                    ),
                }
                if headers > 0 {
                    say(
                        i,
                        40,
                        41,
                        true,
                        "region cards come before the first header card".to_string(),
                    );
                }
            }
            5..=8 => headers += 1,
            _ => {}
        }
    }

    // Symbols.
    let mut beyond = Vec::new();
    for o in &deck.occurrences {
        let (line, start, end) = o.place;
        match o.key {
            Key::Local(..) => {
                let places = defined.get(&o.key).map(Vec::as_slice).unwrap_or(&[]);
                if places.is_empty() && !o.defines {
                    say(
                        line,
                        start,
                        end,
                        false,
                        format!(
                            "the local symbol {} is used but never defined in this list structure",
                            show(o.key)
                        ),
                    );
                } else if o.defines && places.len() > 1 && places[0] != o.place {
                    say(
                        line,
                        start,
                        end,
                        true,
                        format!(
                            "{} already names a cell of this list structure, on line {}",
                            show(o.key),
                            places[0].0 + 1
                        ),
                    );
                }
            }
            Key::Regional(ch, n) => {
                let system = matches!(ch, b'H' | b'J' | b'W');
                if ch == b'J' && names::process(n).is_none() {
                    say(
                        line,
                        start,
                        end,
                        true,
                        format!("there is no basic process J{}", n),
                    );
                } else if (ch == b'H' && n > 5) || (ch == b'W' && n > 43) {
                    say(
                        line,
                        start,
                        end,
                        false,
                        format!("{} is not one of the system's cells", show(o.key)),
                    );
                } else if o.defines && ch == b'J' {
                    say(
                        line,
                        start,
                        end,
                        true,
                        format!("{} is a basic process and cannot be defined", show(o.key)),
                    );
                } else if o.defines && ch == b'H' {
                    // The Logic Theorist does it, to put symbols in H0.
                    say(
                        line,
                        start,
                        end,
                        false,
                        format!(
                            "{} is one of the system's cells: this card loads into it",
                            show(o.key)
                        ),
                    );
                }
                if !system {
                    if let Some(&(size, _)) = deck.regions.get(&ch) {
                        if n >= size && !beyond.contains(&o.key) {
                            beyond.push(o.key);
                            say(
                                line,
                                start,
                                end,
                                false,
                                format!(
                                    "{} is beyond the end of region {}, which has {} cells",
                                    show(o.key),
                                    ch as char,
                                    size
                                ),
                            );
                        }
                    }
                    if o.control && !defined.contains_key(&o.key) {
                        if Some(line) == deck.start {
                            say(
                                line,
                                start,
                                end,
                                true,
                                format!("the start card names {}, but no card defines it", show(o.key)),
                            );
                        } else {
                            say(
                                line,
                                start,
                                end,
                                false,
                                format!("{} is executed, but no card defines it", show(o.key)),
                            );
                        }
                    }
                }
            }
            Key::Internal(_) => {}
        }
    }
    if deck.start.is_none() && deck.lines.iter().any(|l| !l.card.is_blank()) {
        let last = deck.lines.len().saturating_sub(1);
        say(
            last,
            0,
            deck.lines[last].width,
            false,
            "the deck has no start card: TYPE 5 with a routine in SYMB".to_string(),
        );
    }
    out.sort_by_key(|d| (d.line, d.start));
    out
}

/// Markdown about what is at a position, and the stretch it is about.
pub fn hover(text: &str, line: usize, col: usize) -> Option<(String, Place)> {
    let deck = analyse(text);
    let l = deck.lines.get(line)?;
    let card = &l.card;
    if l.mode == Mode::Input {
        return Some((
            "A card for the program to read: it is behind the start card.".to_string(),
            (line, 0, l.width),
        ));
    }
    if col == COL_TYPE - 1 && card.cols[col] != b' ' {
        return Some((
            format!("**TYPE {}**: {}.", card.typ, names::CARD_TYPES[card.typ as usize]),
            (line, col, col + 1),
        ));
    }
    if card.typ != 0 && !is_start(card) {
        return (card.typ != 1 && col >= 40).then(|| {
            (
                format!("**TYPE {}**: {}.", card.typ, names::CARD_TYPES[card.typ as usize]),
                (line, 40, l.width.max(41)),
            )
        });
    }
    if card.is_blank() || col < 40 {
        return None;
    }
    let defined = deck.definitions();
    let mut parts = Vec::new();
    let field = |f: Field| match f {
        Field::Blank => String::new(),
        Field::Regional(ch, n) => format!("{}{}", ch as char, n),
        Field::Local(n) => format!("9-{}", n),
        Field::Internal(n) => n.to_string(),
    };
    // What the card as a whole says.
    if is_start(card) {
        parts.push(format!(
            "**The start card.** Loading ends, and {} is executed.",
            field(card.symb)
        ));
    } else if l.term {
        let what = match card.p {
            0 => format!(
                "the integer {}{}",
                if card.sign == b'-' { "-" } else { "" },
                card.digits().trim_start_matches('0')
            ),
            1 => "a floating point number".to_string(),
            2 => format!(
                "the characters `{}`",
                String::from_utf8_lossy(card.text(COL_SYMB, COL_SYMB + 4))
            ),
            _ => "an octal number".to_string(),
        };
        parts.push(format!(
            "**A data term**: {}.",
            if what == "the integer " {
                "the integer 0".to_string()
            } else {
                what
            }
        ));
    } else if l.data {
        let s = match card.symb {
            Field::Blank => "the next cell, or 0 at the end of a list structure".to_string(),
            f => format!("the symbol {}", field(f)),
        };
        parts.push(format!("**A data word** holding {}.", s));
    } else {
        let s = field(card.symb);
        let s = if s.is_empty() {
            "the next cell".to_string()
        } else {
            s
        };
        let designated = match card.q {
            1 => format!("S is the symbol in cell {}", s),
            2 => format!("S is the symbol in the cell named in cell {}", s),
            3 => format!("S is {}; a monitor point, so this routine is traced", s),
            4 => format!("S is {}; traced if the routine that executes it is", s),
            5..=7 => format!("Q = {} is for the system's own use", card.q),
            _ => format!("S is {}", s),
        };
        parts.push(format!(
            "**{}{} {}** {}. {}.",
            card.p,
            card.q,
            s,
            names::OPERATIONS[card.p as usize],
            designated
        ));
        if let (0, 0 | 3 | 4, Field::Regional(b'J', n)) = (card.p, card.q, card.symb) {
            if let Some(p) = names::process(n) {
                parts.push(format!(
                    "**J{}** {}.{}",
                    n,
                    p,
                    if names::sets_h5(n) { " Sets H5." } else { "" }
                ));
            }
        }
        match card.link {
            Field::Blank => {}
            Field::Internal(0) => parts.push("LINK 0: the list ends here.".to_string()),
            Field::Regional(b'J', n) => parts.push(format!(
                "LINK J{}: then {}, and the list ends.",
                n,
                names::process(n).unwrap_or_default().to_lowercase()
            )),
            f => parts.push(format!("LINK {}: that is the next instruction.", field(f))),
        }
    }
    // What the symbol under the pointer is.
    let mut place = (line, 40, l.width.max(41));
    if let Some(o) = deck.at(line, col) {
        place = o.place;
        let name = show(o.key);
        let about = match o.key {
            Key::Regional(b'J', n) => names::process(n).map(|p| format!("`J{}` {}.", n, p)),
            Key::Regional(ch, n) if names::cell(ch, n).is_some() => {
                names::cell(ch, n).map(|c| format!("`{}` is {}.", name, c))
            }
            _ => match defined.get(&o.key).and_then(|d| d.first()) {
                Some(d) if o.defines && d.0 == line => {
                    let uses = deck
                        .occurrences
                        .iter()
                        .filter(|u| u.key == o.key && !u.defines)
                        .count();
                    Some(format!(
                        "`{}` names this cell. It is used {} time{} in the deck.",
                        name,
                        uses,
                        if uses == 1 { "" } else { "s" }
                    ))
                }
                Some(d) => {
                    let comment = deck.lines[d.0].card.comment();
                    Some(format!(
                        "`{}` is defined on line {}{}",
                        name,
                        d.0 + 1,
                        if comment.is_empty() {
                            ".".to_string()
                        } else {
                            format!(": {}", comment)
                        }
                    ))
                }
                None if matches!(o.key, Key::Regional(..)) => Some(format!(
                    "`{}` is not defined by any card: an empty cell, or a symbol used for itself.",
                    name
                )),
                None => None,
            },
        };
        parts.extend(about);
    }
    Some((parts.join("\n\n"), place))
}

pub const TOKEN_TYPES: [&str; 8] = [
    "comment", "keyword", "function", "variable", "number", "operator", "string", "macro",
];

/// Line, column, length and an index into `TOKEN_TYPES`, in order.
pub fn semantic(text: &str) -> Vec<(usize, usize, usize, usize)> {
    let deck = analyse(text);
    let mut out = Vec::new();
    for (i, l) in deck.lines.iter().enumerate() {
        let card = &l.card;
        let mut push = |from: usize, to: usize, kind: usize| {
            // Columns from..to, from 0, trimmed of blanks.
            let s = &card.cols[from..to.min(80)];
            if let (Some(a), Some(b)) = (
                s.iter().position(|&c| c != b' '),
                s.iter().rposition(|&c| c != b' '),
            ) {
                out.push((i, from + a, b + 1 - a, kind));
            }
        };
        if l.mode == Mode::Input {
            push(0, 80, 6);
            continue;
        }
        push(0, 40, 0);
        push(40, 41, 1);
        let symbol = |f: Field, control: bool| match f {
            Field::Regional(b'J', _) => 7,
            Field::Regional(..) if control => 2,
            Field::Regional(..) | Field::Local(_) => 3,
            _ => 4,
        };
        match card.typ {
            0 if matches!(l.mode, Mode::Routines | Mode::Data) => {
                push(
                    42,
                    47,
                    if l.data || !matches!(card.name, Field::Regional(..)) {
                        3
                    } else {
                        2
                    },
                );
                push(47, 50, 5);
                if l.term {
                    push(50, 61, if card.p == 2 { 6 } else { 4 });
                } else {
                    push(50, 55, symbol(card.symb, !l.data && matches!(card.p, 0 | 7)));
                    push(56, 61, symbol(card.link, !l.data));
                }
                push(61, 80, 0);
            }
            2 | 3 | 5..=9 => {
                push(42, 47, 3);
                push(48, 50, 5);
                push(50, 55, if is_start(card) { 2 } else { 4 });
                push(56, 61, 4);
                push(61, 80, 0);
            }
            _ => push(41, 80, 0),
        }
    }
    out
}

pub fn definition(text: &str, line: usize, col: usize) -> Option<Place> {
    let deck = analyse(text);
    let key = deck.at(line, col)?.key;
    deck.definitions().get(&key)?.first().copied()
}

/// Every place the symbol at a position is written.
pub fn references(text: &str, line: usize, col: usize) -> Vec<Place> {
    let deck = analyse(text);
    match deck.at(line, col) {
        Some(o) => deck
            .occurrences
            .iter()
            .filter(|u| u.key == o.key)
            .map(|u| u.place)
            .collect(),
        None => Vec::new(),
    }
}

/// A new name as it must be written in a five-column field, if it is one:
/// a regional symbol where the old one was regional, local where local.
pub fn renamed(text: &str, line: usize, col: usize, new: &str) -> Option<Vec<(Place, String)>> {
    let deck = analyse(text);
    let old = deck.at(line, col)?.key;
    let new = new.trim().to_ascii_uppercase();
    let parsed = parse_field(format!("{:<5}", new).as_bytes().get(..5)?);
    let fits = new.len() <= 5
        && match (old, parsed) {
            (Key::Regional(ch, _), Field::Regional(c, _)) => {
                !matches!(ch, b'H' | b'J' | b'W')
                    && !matches!(c, b'H' | b'J' | b'W')
                    && new[1..].bytes().all(|b| b.is_ascii_digit())
            }
            (Key::Local(..), Field::Local(_)) => {
                new.starts_with("9-") && new[2..].bytes().all(|b| b.is_ascii_digit())
            }
            _ => false,
        };
    if !fits {
        return None;
    }
    // The whole field is rewritten, so that the columns after it stay put.
    Some(
        deck.occurrences
            .iter()
            .filter(|o| o.key == old)
            .map(|o| {
                let col = match (
                    o.slot,
                    deck.lines[o.place.0].card.typ,
                    deck.lines[o.place.0].card.name,
                ) {
                    (Slot::Name, 3, Field::Blank) => COL_SYMB,
                    (Slot::Name, ..) => COL_NAME,
                    (Slot::Symb, ..) => COL_SYMB,
                    (Slot::Link, ..) => COL_LINK,
                };
                ((o.place.0, col - 1, col + 4), format!("{:<5}", new))
            })
            .collect(),
    )
}

/// The outline: each list structure with a regional name, whether it is a
/// routine, its comment, and its first and last lines.
pub fn outline(text: &str) -> Vec<(String, bool, String, usize, usize)> {
    let deck = analyse(text);
    deck.structures()
        .into_iter()
        .map(|(key, first, last)| {
            (
                show(key),
                !deck.lines[first].data,
                deck.lines[first].card.comment(),
                first,
                last,
            )
        })
        .collect()
}

/// The deck as a keypunch would have left it: upper case, nothing after
/// the last punch, tabs as blanks, and each symbol at the left of its field
/// as the manual requires. Cards for the program to read are left alone.
pub fn format(text: &str) -> String {
    let deck = analyse(text);
    let mut out = String::with_capacity(text.len());
    for (l, raw) in deck.lines.iter().zip(text.lines()) {
        if l.mode == Mode::Input {
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        let mut cols: Vec<char> = raw.replace('\t', " ").to_uppercase().chars().collect();
        let word = l.card.typ == 0 && matches!(l.mode, Mode::Routines | Mode::Data);
        if word || is_start(&l.card) {
            let fields: &[usize] = if l.term {
                &[COL_NAME]
            } else {
                &[COL_NAME, COL_SYMB, COL_LINK]
            };
            for &col in fields {
                if cols.len() < col + 4 {
                    cols.resize(col + 4, ' ');
                }
                let f: String = cols[col - 1..col + 4].iter().collect();
                // An internal symbol may stand anywhere; the others go left.
                if !f.trim().starts_with(|c: char| c.is_ascii_digit()) || f.trim().starts_with("9-") {
                    for (k, c) in format!("{:<5}", f.trim()).chars().enumerate() {
                        cols[col - 1 + k] = c;
                    }
                }
            }
        }
        let line: String = cols.into_iter().collect();
        out.push_str(line.trim_end());
        out.push('\n');
    }
    if !text.ends_with('\n') && !text.is_empty() {
        out.pop();
    }
    out
}
