//! Save for restart and restarting (manual section 20).
//!
//! J166 writes the whole state of the IPL computer, storage and the
//! interpreter's own, to a tape as one record. A Type-5 card with P = 4 and
//! the unit in SYMB reloads it, and interpretation goes on with the
//! instruction after that J166, with H5 set minus. Tapes are not saved: as
//! the manual says, positioning them again is the program's business. The
//! cards in the reader are not part of the image either: what follows the
//! restart card is read by the program, normally by the J165 after J166.

use crate::interp::{Cont, Ctx};
use crate::machine::*;
use crate::prims::StructureGen;
use crate::word::*;
use std::collections::{HashMap, VecDeque};

/// A J101 in progress: its subprocess, the lists marked, the sublists
/// waiting and the next cell.
type J101Image = (u32, Vec<u32>, Vec<u32>, Option<u32>);

/// What J166 saves.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    mem: Vec<Word>,
    regions: Vec<Region>,
    strays: Vec<(u8, u32, u32)>,
    brk: u32,
    top: u32,
    space_built: bool,
    used: Vec<bool>,
    owned: Vec<bool>,
    reserved: Vec<bool>,
    free_cells: u32,
    low_water: u32,
    reserve: Vec<u32>,
    space_limit: Option<u32>,
    internals: Vec<(u32, u32)>,
    lines: Vec<(u32, Vec<u8>)>,
    conts: Vec<(ContImage, CtxImage)>,
    gens: Vec<Vec<(u32, bool)>>,
    structures: Vec<J101Image>,
    level: u32,
    cycles: u64,
    trace_all: Option<bool>,
    /// "The entire contents of main and auxiliary storage."
    aux: [crate::auxiliary::Store; 2],
    /// How J166 was reached, so that the restart takes the same next step.
    ctx: CtxImage,
}

// Cont and Ctx as plain data, so that an image can be compared and written.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ContImage {
    Subprocess,
    Symbols(u32, u32),
    Cells(u32),
    Structure,
    MonitorStart,
    MonitorEnd,
    Trap(bool),
    H5Plus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CtxImage {
    Instruction,
    Link,
    None,
    Resume,
}

fn cont_out(c: Cont) -> ContImage {
    match c {
        Cont::Subprocess => ContImage::Subprocess,
        Cont::Symbols { sub, next } => ContImage::Symbols(sub, next),
        Cont::Cells { sub } => ContImage::Cells(sub),
        Cont::Structure => ContImage::Structure,
        Cont::MonitorStart => ContImage::MonitorStart,
        Cont::MonitorEnd => ContImage::MonitorEnd,
        Cont::Trap { space } => ContImage::Trap(space),
        Cont::H5Plus => ContImage::H5Plus,
    }
}

fn cont_in(c: ContImage) -> Cont {
    match c {
        ContImage::Subprocess => Cont::Subprocess,
        ContImage::Symbols(sub, next) => Cont::Symbols { sub, next },
        ContImage::Cells(sub) => Cont::Cells { sub },
        ContImage::Structure => Cont::Structure,
        ContImage::MonitorStart => Cont::MonitorStart,
        ContImage::MonitorEnd => Cont::MonitorEnd,
        ContImage::Trap(space) => Cont::Trap { space },
        ContImage::H5Plus => Cont::H5Plus,
    }
}

fn ctx_out(c: Ctx) -> CtxImage {
    match c {
        Ctx::Instruction => CtxImage::Instruction,
        Ctx::Link => CtxImage::Link,
        Ctx::None => CtxImage::None,
        Ctx::Resume => CtxImage::Resume,
    }
}

fn ctx_in(c: CtxImage) -> Ctx {
    match c {
        CtxImage::Instruction => Ctx::Instruction,
        CtxImage::Link => Ctx::Link,
        CtxImage::None => Ctx::None,
        CtxImage::Resume => Ctx::Resume,
    }
}

impl Machine {
    /// The image J166 writes, as the machine stands while executing it.
    pub fn snapshot(&self) -> Image {
        let mut strays: Vec<_> = self.strays.iter().map(|(&(ch, n), &a)| (ch, n, a)).collect();
        strays.sort();
        let mut internals: Vec<_> = self.internals.iter().map(|(&k, &v)| (k, v)).collect();
        internals.sort();
        let mut lines: Vec<_> = self.lines.iter().map(|(&k, v)| (k, v.clone())).collect();
        lines.sort();
        Image {
            mem: self.mem.to_vec(),
            regions: self.regions.clone(),
            strays,
            brk: self.brk,
            top: self.top,
            space_built: self.space_built,
            used: self.used.clone(),
            owned: self.owned.clone(),
            reserved: self.reserved.clone(),
            free_cells: self.free_cells,
            low_water: self.low_water,
            reserve: self.reserve.clone(),
            space_limit: self.space_limit,
            internals,
            lines,
            conts: self
                .conts
                .iter()
                .map(|&(k, c)| (cont_out(k), ctx_out(c)))
                .collect(),
            gens: self.gens.clone(),
            structures: self
                .structures
                .iter()
                .map(|g| {
                    (
                        g.sub,
                        g.record.clone(),
                        g.waiting.iter().copied().collect(),
                        g.next,
                    )
                })
                .collect(),
            level: self.level,
            cycles: self.cycles,
            trace_all: self.trace_all,
            aux: self.aux.clone(),
            ctx: ctx_out(self.ctx_now),
        }
    }

    /// Put the machine back as `image` has it, then set H5 minus: the next
    /// step goes on after the J166 that saved it.
    pub fn restore_image(&mut self, image: &Image) {
        self.mem = crate::storage::Storage::new(image.mem.clone());
        self.regions = image.regions.clone();
        self.region_of = [255; 128];
        for (i, r) in self.regions.iter().enumerate() {
            self.region_of[(r.ch & 127) as usize] = i as u8;
        }
        self.strays = image.strays.iter().map(|&(ch, n, a)| ((ch, n), a)).collect();
        self.stray_names = image.strays.iter().map(|&(ch, n, a)| (a, (ch, n))).collect();
        self.brk = image.brk;
        self.top = image.top;
        self.space_built = image.space_built;
        self.used = image.used.clone();
        self.owned = image.owned.clone();
        self.reserved = image.reserved.clone();
        self.free_cells = image.free_cells;
        self.low_water = image.low_water;
        self.reserve = image.reserve.clone();
        self.space_limit = image.space_limit;
        self.internals = image.internals.iter().copied().collect();
        self.lines = image.lines.iter().cloned().collect::<HashMap<_, _>>();
        self.conts = image
            .conts
            .iter()
            .map(|&(k, c)| (cont_in(k), ctx_in(c)))
            .collect();
        self.gens = image.gens.clone();
        self.structures = image
            .structures
            .iter()
            .map(|(sub, record, waiting, next)| StructureGen {
                sub: *sub,
                record: record.clone(),
                waiting: waiting.iter().copied().collect::<VecDeque<_>>(),
                next: *next,
            })
            .collect();
        self.level = image.level;
        self.cycles = image.cycles;
        self.trace_all = image.trace_all;
        self.aux = image.aux.clone();
        self.pending_trap = None;
        self.w34_preserved = false;
        self.halted = false;
        self.set_h5(false);
        self.resume_ctx = Some(ctx_in(image.ctx));
    }
}

// ---- the image as text, for a tape kept in a file ----

fn bits(v: &[bool]) -> String {
    // Runs of set cells, as first-last.
    let mut out = Vec::new();
    let mut i = 0;
    while i < v.len() {
        if v[i] {
            let start = i;
            while i < v.len() && v[i] {
                i += 1;
            }
            out.push(format!("{}-{}", start, i - 1));
        } else {
            i += 1;
        }
    }
    out.join(" ")
}

fn unbits(s: &str, len: usize) -> Res<Vec<bool>> {
    let mut v = vec![false; len];
    for run in s.split_whitespace() {
        let (a, b) = run.split_once('-').ok_or("a bad run of cells")?;
        let (a, b): (usize, usize) = (num(a)?, num(b)?);
        for x in v.iter_mut().take(b.min(len.saturating_sub(1)) + 1).skip(a) {
            *x = true;
        }
    }
    Ok(v)
}

fn num<T: std::str::FromStr>(s: &str) -> Res<T> {
    s.parse().map_err(|_| format!("not a number: {}", s))
}

fn nums(s: &str) -> Res<Vec<u32>> {
    s.split_whitespace().map(num).collect()
}

fn opt(o: Option<u32>) -> String {
    o.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string())
}

fn unopt(s: &str) -> Res<Option<u32>> {
    if s == "-" {
        Ok(None)
    } else {
        num(s).map(Some)
    }
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|c| format!("{:02x}", c)).collect()
}

fn unhex(s: &str) -> Res<Vec<u8>> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).map_err(|_| "bad hex".to_string()))
        .collect()
}

impl Image {
    pub fn to_text(&self) -> String {
        let mut o = String::new();
        let mut put = |k: &str, v: String| {
            o.push_str(k);
            o.push(' ');
            o.push_str(&v);
            o.push('\n');
        };
        put("size", self.mem.len().to_string());
        put(
            "state",
            format!(
                "{} {} {} {} {} {} {} {} {:?} {:?}",
                self.brk,
                self.top,
                self.space_built as u8,
                self.free_cells,
                self.low_water,
                opt(self.space_limit),
                self.level,
                self.cycles,
                self.trace_all.map(|t| t as u8),
                self.ctx
            ),
        );
        for r in &self.regions {
            put("region", format!("{} {} {}", r.ch as char, r.origin, r.size));
        }
        for (ch, n, a) in &self.strays {
            put("stray", format!("{} {} {}", *ch as char, n, a));
        }
        put("used", bits(&self.used));
        put("owned", bits(&self.owned));
        put("reserved", bits(&self.reserved));
        put(
            "reserve",
            self.reserve
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
        for (k, v) in &self.internals {
            put("internal", format!("{} {}", k, v));
        }
        for (a, text) in &self.lines {
            put("line", format!("{} {}", a, hex(text)));
        }
        for (k, c) in &self.conts {
            let k = match k {
                ContImage::Subprocess => "subprocess".to_string(),
                ContImage::Symbols(s, n) => format!("symbols {} {}", s, n),
                ContImage::Cells(s) => format!("cells {}", s),
                ContImage::Structure => "structure".to_string(),
                ContImage::MonitorStart => "monitorstart".to_string(),
                ContImage::MonitorEnd => "monitorend".to_string(),
                ContImage::Trap(s) => format!("trap {}", *s as u8),
                ContImage::H5Plus => "h5plus".to_string(),
            };
            put("cont", format!("{:?} {}", c, k));
        }
        for g in &self.gens {
            put(
                "gen",
                g.iter()
                    .map(|(a, b)| format!("{}{}", a, if *b { "+" } else { "." }))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }
        for (sub, record, waiting, next) in &self.structures {
            let j = |v: &Vec<u32>| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ");
            put(
                "j101",
                format!("{} {} | {} | {}", sub, opt(*next), j(record), j(waiting)),
            );
        }
        for (store, s) in self.aux.iter().enumerate() {
            for (place, f) in s.filed.iter().enumerate() {
                if let Some(f) = f {
                    let cells: Vec<String> = f
                        .words
                        .iter()
                        .zip(&f.inner)
                        .map(|(w, (a, b))| {
                            format!("{},{},{},{},{},{}", w.p, w.q, w.symb, w.link, *a as u8, *b as u8)
                        })
                        .collect();
                    put(
                        "filed",
                        format!("{} {} {} {}", store, place, f.current as u8, cells.join(" ")),
                    );
                }
            }
        }
        for (a, w) in self.mem.iter().enumerate() {
            if *w != Word::ZERO {
                put("w", format!("{} {} {} {} {}", a, w.p, w.q, w.symb, w.link));
            }
        }
        o
    }

    pub fn from_text(text: &str) -> Res<Image> {
        let mut size = 0;
        let mut im = Image {
            mem: Vec::new(),
            regions: Vec::new(),
            strays: Vec::new(),
            brk: 0,
            top: 0,
            space_built: false,
            used: Vec::new(),
            owned: Vec::new(),
            reserved: Vec::new(),
            free_cells: 0,
            low_water: 0,
            reserve: Vec::new(),
            space_limit: None,
            internals: Vec::new(),
            lines: Vec::new(),
            conts: Vec::new(),
            gens: Vec::new(),
            structures: Vec::new(),
            level: 1,
            cycles: 0,
            trace_all: None,
            aux: Default::default(),
            ctx: CtxImage::Instruction,
        };
        let (mut used, mut owned, mut reserved) = ("", "", "");
        for line in text.lines() {
            let (k, v) = line.split_once(' ').unwrap_or((line, ""));
            let f: Vec<&str> = v.split_whitespace().collect();
            let ctx = |s: &str| -> Res<CtxImage> {
                Ok(match s {
                    "Instruction" => CtxImage::Instruction,
                    "Link" => CtxImage::Link,
                    "None" => CtxImage::None,
                    "Resume" => CtxImage::Resume,
                    _ => return Err(format!("unknown context {}", s)),
                })
            };
            match k {
                "size" => {
                    size = num(v)?;
                    im.mem = vec![Word::ZERO; size];
                }
                "state" => {
                    if f.len() != 10 {
                        return Err("a bad state line".to_string());
                    }
                    im.brk = num(f[0])?;
                    im.top = num(f[1])?;
                    im.space_built = f[2] == "1";
                    im.free_cells = num(f[3])?;
                    im.low_water = num(f[4])?;
                    im.space_limit = unopt(f[5])?;
                    im.level = num(f[6])?;
                    im.cycles = num(f[7])?;
                    im.trace_all = match f[8] {
                        "Some(1)" => Some(true),
                        "Some(0)" => Some(false),
                        _ => None,
                    };
                    im.ctx = ctx(f[9])?;
                }
                "region" if f.len() == 3 => im.regions.push(Region {
                    ch: f[0].as_bytes()[0],
                    origin: num(f[1])?,
                    size: num(f[2])?,
                }),
                "stray" if f.len() == 3 => im.strays.push((f[0].as_bytes()[0], num(f[1])?, num(f[2])?)),
                "used" => used = v,
                "owned" => owned = v,
                "reserved" => reserved = v,
                "reserve" => im.reserve = nums(v)?,
                "internal" if f.len() == 2 => im.internals.push((num(f[0])?, num(f[1])?)),
                "line" if f.len() == 2 => im.lines.push((num(f[0])?, unhex(f[1])?)),
                "line" if f.len() == 1 => im.lines.push((num(f[0])?, Vec::new())),
                "cont" if f.len() >= 2 => {
                    let c = ctx(f[0])?;
                    let k = match (f[1], &f[2..]) {
                        ("subprocess", _) => ContImage::Subprocess,
                        ("symbols", [s, n]) => ContImage::Symbols(num(s)?, num(n)?),
                        ("cells", [s]) => ContImage::Cells(num(s)?),
                        ("structure", _) => ContImage::Structure,
                        ("monitorstart", _) => ContImage::MonitorStart,
                        ("monitorend", _) => ContImage::MonitorEnd,
                        ("trap", [s]) => ContImage::Trap(*s == "1"),
                        ("h5plus", _) => ContImage::H5Plus,
                        _ => return Err(format!("a bad continuation: {}", v)),
                    };
                    im.conts.push((k, c));
                }
                "gen" => {
                    let mut g = Vec::new();
                    for t in &f {
                        let (a, b) = t.split_at(t.len() - 1);
                        g.push((num(a)?, b == "+"));
                    }
                    im.gens.push(g);
                }
                "j101" => {
                    let parts: Vec<&str> = v.split('|').collect();
                    if parts.len() != 3 {
                        return Err("a bad J101 line".to_string());
                    }
                    let head: Vec<&str> = parts[0].split_whitespace().collect();
                    if head.len() != 2 {
                        return Err("a bad J101 line".to_string());
                    }
                    im.structures
                        .push((num(head[0])?, nums(parts[1])?, nums(parts[2])?, unopt(head[1])?));
                }
                "filed" if f.len() >= 3 => {
                    let (store, place): (usize, usize) = (num(f[0])?, num(f[1])?);
                    if store > 1 {
                        return Err("a bad auxiliary store".to_string());
                    }
                    let mut filed = crate::auxiliary::Filed {
                        words: Vec::new(),
                        inner: Vec::new(),
                        current: f[2] == "1",
                    };
                    for c in &f[3..] {
                        let v: Vec<u32> = c.split(',').map(num).collect::<Res<_>>()?;
                        if v.len() != 6 {
                            return Err("a bad auxiliary cell".to_string());
                        }
                        filed.words.push(Word {
                            p: v[0] as u8,
                            q: v[1] as u8,
                            symb: v[2],
                            link: v[3],
                        });
                        filed.inner.push((v[4] == 1, v[5] == 1));
                    }
                    let slots = &mut im.aux[store].filed;
                    if slots.len() <= place {
                        slots.resize(place + 1, None);
                    }
                    slots[place] = Some(filed);
                }
                "w" if f.len() == 5 => {
                    let a: usize = num(f[0])?;
                    if a >= im.mem.len() {
                        return Err(format!("cell {} is outside the image", a));
                    }
                    im.mem[a] = Word {
                        p: num(f[1])?,
                        q: num(f[2])?,
                        symb: num(f[3])?,
                        link: num(f[4])?,
                    };
                }
                "" => {}
                _ => return Err(format!("a bad line in a storage image: {}", line)),
            }
        }
        if size == 0 {
            return Err("a storage image without its size".to_string());
        }
        let flags = im.used.len().max(EXTRA_ORG as usize);
        im.used = unbits(used, flags)?;
        im.owned = unbits(owned, flags)?;
        im.reserved = unbits(reserved, flags)?;
        Ok(im)
    }
}
