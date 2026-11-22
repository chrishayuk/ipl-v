//! The 80-column card of the IPL-V coding sheet (manual section 1.6, Fig. 1).
//!
//! ```text
//! columns  1-40  comments
//!          41    TYPE   (blank or 0: a word; 1-9: see section 18)
//!          43-47 NAME
//!          48    SIGN
//!          49    P
//!          50    Q
//!          51-55 SYMB
//!          57-61 LINK
//!          63-80 comments and identification
//! ```

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Blank,
    /// Region character and number: `A12`, `*3`, `M` (which is `M0`).
    Regional(u8, u32),
    /// `9-12`: a 9 in the leftmost column.
    Local(u32),
    /// A decimal integer anywhere in the field.
    Internal(u32),
}

#[derive(Clone, Debug)]
pub struct Card {
    /// The card as punched, for listings.
    pub raw: String,
    /// Line number in the file it came from, from 1.
    pub line: usize,
    /// Upper-cased and padded to 80 columns.
    pub cols: [u8; 80],
    pub typ: u8,
    pub name: Field,
    pub sign: u8,
    pub p: u8,
    pub q: u8,
    pub symb: Field,
    pub link: Field,
}

/// Columns are numbered from 1, as on the coding sheet.
pub const COL_TYPE: usize = 41;
pub const COL_NAME: usize = 43;
pub const COL_SIGN: usize = 48;
pub const COL_P: usize = 49;
pub const COL_Q: usize = 50;
pub const COL_SYMB: usize = 51;
pub const COL_LINK: usize = 57;

/// The rules of section 1.6 for a five-column symbol field: a regional
/// symbol begins with its region character and a local symbol with 9;
/// after that everything that is not a digit is ignored, and the digits
/// are composed into a number wherever they stand.
///
/// Two things here are not in the manual. It wants the first character in
/// the leftmost column, and reads an indented ` 91` as the internal symbol
/// 91; but the surviving transcriptions were made from listings, which
/// indent, so a symbol is read from its first character wherever that is.
/// And a 9 by itself is the internal symbol 9, not the local symbol 9-0:
/// the Logic Theorist's table of characters lists the digits 0 to 9 so.
pub fn parse_field(f: &[u8]) -> Field {
    let Some(start) = f.iter().position(|&c| c != b' ') else {
        return Field::Blank;
    };
    let (first, rest) = (f[start], &f[start + 1..]);
    let number = |s: &[u8]| -> u32 {
        s.iter()
            .filter(|c| c.is_ascii_digit())
            .fold(0u32, |n, &c| n.wrapping_mul(10).wrapping_add((c - b'0') as u32))
    };
    if first == b'9' && rest.iter().any(|&c| c != b' ') {
        Field::Local(number(rest))
    } else if !first.is_ascii_digit() {
        Field::Regional(first, number(rest))
    } else {
        Field::Internal(number(&f[start..]))
    }
}

fn digit(c: u8) -> u8 {
    if c.is_ascii_digit() {
        c - b'0'
    } else {
        0
    }
}

impl Card {
    pub fn parse(raw: &str, line: usize) -> Card {
        let mut cols = [b' '; 80];
        for (i, c) in raw.trim_end_matches(['\r', '\n']).chars().take(80).enumerate() {
            cols[i] = if c.is_ascii() && !c.is_ascii_control() {
                c.to_ascii_uppercase() as u8
            } else {
                b' '
            };
        }
        let f = |col: usize| &cols[col - 1..col + 4];
        Card {
            raw: raw.trim_end().to_string(),
            line,
            typ: digit(cols[COL_TYPE - 1]),
            name: parse_field(f(COL_NAME)),
            sign: cols[COL_SIGN - 1],
            p: digit(cols[COL_P - 1]),
            q: digit(cols[COL_Q - 1]),
            symb: parse_field(f(COL_SYMB)),
            link: parse_field(f(COL_LINK)),
            cols,
        }
    }

    pub fn is_blank(&self) -> bool {
        self.cols.iter().all(|&c| c == b' ')
    }

    /// Columns `from..=to`, numbered from 1.
    pub fn text(&self, from: usize, to: usize) -> &[u8] {
        &self.cols[from - 1..to]
    }

    /// The digits of SYMB and LINK run together: the value of an integer or
    /// octal data term (Fig. 2).
    pub fn digits(&self) -> String {
        self.text(COL_SYMB, COL_LINK + 4)
            .iter()
            .filter(|c| c.is_ascii_digit())
            .map(|&c| c as char)
            .collect()
    }

    /// A count punched in a symbol field: the size on a region card.
    pub fn count(&self, col: usize) -> Option<u32> {
        let d: String = self
            .text(col, col + 4)
            .iter()
            .filter(|c| c.is_ascii_digit())
            .map(|&c| c as char)
            .collect();
        d.parse().ok()
    }

    pub fn comment(&self) -> String {
        String::from_utf8_lossy(self.text(1, 40)).trim().to_string()
    }
}
