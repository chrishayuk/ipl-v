//! The IPL word (manual section 1.4) and the data terms that share its cell
//! (section 1.5).
//!
//! A standard word is `P Q SYMB LINK`. P and Q are octal digits; SYMB and
//! LINK are symbols, which inside the computer are addresses. A data term is
//! a word with Q = 1 whose P gives the type and whose other two fields hold
//! the datum. On the IBM 704 that was thirty bits; here the two fields
//! together give sixty-four.

/// Twelve bytes, laid out as in C, so that a page can read storage as it
/// stands: P at 0, Q at 1, SYMB at 4, LINK at 8.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Word {
    pub p: u8,
    pub q: u8,
    pub symb: u32,
    pub link: u32,
}

/// Q in data words says what kind of symbol SYMB is (section 12.0).
pub const Q_REGIONAL: u8 = 0;
pub const Q_DATA: u8 = 1;
pub const Q_LOCAL: u8 = 2;
pub const Q_INTERNAL: u8 = 4;
/// Q in the cell that names a primitive: SYMB is the machine routine.
pub const Q_PRIM: u8 = 5;

/// A symbol together with the Q that travels with every copy of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sym {
    pub a: u32,
    pub q: u8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Datum {
    Int(i64),
    Float(f64),
    Alpha([u8; 5]),
    Octal(u64),
}

impl Word {
    pub const ZERO: Word = Word {
        p: 0,
        q: 0,
        symb: 0,
        link: 0,
    };

    pub fn is_data(&self) -> bool {
        self.q == Q_DATA
    }

    pub fn bits(&self) -> u64 {
        ((self.symb as u64) << 32) | self.link as u64
    }

    pub fn data(p: u8, bits: u64) -> Word {
        Word {
            p,
            q: Q_DATA,
            symb: (bits >> 32) as u32,
            link: bits as u32,
        }
    }

    pub fn datum(&self) -> Option<Datum> {
        if !self.is_data() {
            return None;
        }
        let b = self.bits();
        Some(match self.p {
            0 => Datum::Int(b as i64),
            1 => Datum::Float(f64::from_bits(b)),
            2 => {
                let mut a = [b' '; 5];
                for (i, c) in a.iter_mut().enumerate() {
                    *c = (b >> (8 * (4 - i))) as u8;
                }
                Datum::Alpha(a)
            }
            _ => Datum::Octal(b),
        })
    }

    pub fn from_datum(d: Datum) -> Word {
        match d {
            Datum::Int(i) => Word::data(0, i as u64),
            Datum::Float(f) => Word::data(1, f.to_bits()),
            Datum::Alpha(a) => {
                let mut b = 0u64;
                for c in a {
                    b = (b << 8) | c as u64;
                }
                Word::data(2, b)
            }
            Datum::Octal(o) => Word::data(3, o),
        }
    }

    pub fn sym(&self) -> Sym {
        Sym {
            a: self.symb,
            q: self.q,
        }
    }
}

/// How a data term is written on the coding sheet and in listings.
pub fn datum_text(d: Datum) -> String {
    match d {
        Datum::Int(i) => i.to_string(),
        Datum::Float(f) => float_text(f),
        Datum::Alpha(a) => String::from_utf8_lossy(&a).into_owned(),
        Datum::Octal(o) if (o as i64) < 0 => format!("-{:010o}", -(o as i64)),
        Datum::Octal(o) => format!("{:010o}", o),
    }
}

/// Section 16.2: "the entire number is entered, signed value followed by
/// signed exponent", so .505135 x 10^5 is `505135 05`.
pub fn float_text(f: f64) -> String {
    if f == 0.0 {
        return "000000 00".to_string();
    }
    let mut e = f.abs().log10().floor() as i32 + 1;
    let mut m = (f.abs() / 10f64.powi(e) * 1e6).round() as i64;
    if m >= 1_000_000 {
        m /= 10;
        e += 1;
    }
    format!(
        "{}{:06}{}{:02}",
        if f < 0.0 { "-" } else { "" },
        m,
        if e < 0 { "-" } else { " " },
        e.abs()
    )
}
