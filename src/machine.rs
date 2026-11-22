//! The IPL computer: its storage, its symbols and its available space.
//!
//! Storage is one array of words. Every list in the system, including the
//! push-down list of every storage cell and the hierarchy of routines in
//! progress (H1), is made of cells taken from the available space list H2
//! and given back to it (manual sections 1.9, 1.15, 9.1).
//!
//! The map of storage follows the assembly listing in Fig. 1 of the
//! manual's Part One, made on a 32,768-word machine: the programmer's
//! regions begin at cell 64, the total available space is 22,000 cells, and
//! the system's own cells are above it, H0 at 24574 and W0 at 24587.
//!
//! ```text
//! 0-9          the internal symbols 0 to 9 (0 is the termination symbol)
//! 10-63        the region control words (not used here)
//! 64-21999     the programmer's regions, in the order of their Type-2
//!              cards, and then the available space
//! 24574-24579  H0-H5
//! 24587-24636  W0-W49
//! 24650-24859  J0-J209, each naming a primitive
//! 24860-24899  cells of the interpretive system itself
//! 25000-       further available space, if more than 24,574 cells are
//!              asked for
//! ```

use crate::card::Card;
use crate::word::*;
use std::collections::{HashMap, VecDeque};

pub const USER_ORG: u32 = 64;
pub const H0: u32 = 24574;
pub const H1: u32 = H0 + 1;
pub const H2: u32 = H0 + 2;
pub const H3: u32 = H0 + 3;
pub const H4: u32 = H0 + 4;
pub const H5: u32 = H0 + 5;
pub const W_ORG: u32 = 24587;
pub const W_SIZE: u32 = 50;
pub const J_ORG: u32 = 24650;
pub const J_SIZE: u32 = 210;
pub const SYS_ORG: u32 = 24860;
pub const SYS_SIZE: u32 = 40;
/// The system's part of storage: nothing in it is ever available space.
pub const SYSTEM: std::ops::Range<u32> = H0..SYS_ORG + SYS_SIZE;
pub const EXTRA_ORG: u32 = 25000;

pub const fn j(n: u32) -> u32 {
    J_ORG + n
}
pub const fn w(n: u32) -> u32 {
    W_ORG + n
}

/// The generator hideout of J17 to J19 (section 7.1).
pub const G_WN: u32 = SYS_ORG;
pub const G_SUB: u32 = SYS_ORG + 1;
pub const G_SIGN: u32 = SYS_ORG + 2;
pub const G_CONTEXT: u32 = SYS_ORG + 3;
pub const G_TRACE: u32 = SYS_ORG + 4;
/// The print line that W24 names until the program reserves its own.
pub const SYS_LINE: u32 = SYS_ORG + 5;
/// Integer data terms that the W cells name at the start of a run.
const SYS_INTS: u32 = SYS_ORG + 8;

pub const LINE_SIZE: usize = 120;

#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub ch: u8,
    pub origin: u32,
    pub size: u32,
}

/// Where a loaded word came from, for traces and listings.
#[derive(Clone, Debug)]
pub struct Origin {
    /// The regional or internal name that begins the list structure.
    pub structure: String,
    /// Its local name, if the card had one.
    pub local: Option<u32>,
    /// Cards from the start of the structure.
    pub offset: u32,
    pub comment: String,
    pub line: usize,
}

pub struct Machine {
    pub mem: Vec<Word>,
    pub regions: Vec<Region>,
    region_of: [u8; 128],
    /// Regional symbols outside any declared block (see `regional`).
    strays: HashMap<(u8, u32), u32>,
    stray_names: HashMap<u32, (u8, u32)>,
    /// First cell not yet given to a region or to available space.
    brk: u32,
    /// One past the last cell of the space below the system.
    top: u32,
    space_built: bool,
    /// Regional cells that a card or the program has mentioned.
    used: Vec<bool>,
    pub free_cells: u32,
    pub low_water: u32,

    /// The cards still to be read: the rest of the deck after the start card.
    pub cards: VecDeque<Card>,
    /// Internal symbols on cards, treated as pure symbolics (section 18.5).
    pub internals: HashMap<u32, u32>,
    pub origins: HashMap<u32, Origin>,
    pub lines: HashMap<u32, Vec<u8>>,
    pub out: String,
    pub warnings: Vec<String>,
    pub listing: Option<String>,

    pub conts: Vec<(crate::interp::Cont, crate::interp::Ctx)>,
    /// The cells still to come from each J101 or J102 in progress.
    pub gens: Vec<Vec<(u32, bool)>>,
    /// Each J101 in progress.
    pub structures: Vec<crate::prims::StructureGen>,
    pub level: u32,
    pub halted: bool,
    /// The external trace condition (section 15.5): Some(true) is full
    /// trace, Some(false) no trace, None whatever the program's marks say.
    pub trace_all: Option<bool>,
    pub trace_out: String,
    /// Count of executions of each primitive.
    pub prim_counts: Vec<u64>,
    pub cycles: u64,
}

pub type Res<T> = Result<T, String>;

impl Machine {
    /// An IPL computer with `cells` cells of total available space.
    pub fn new(cells: usize) -> Machine {
        let cells = cells.max(USER_ORG as usize + 64);
        let top = cells.min(H0 as usize);
        let mut m = Machine {
            mem: vec![Word::ZERO; EXTRA_ORG as usize + (cells - top)],
            regions: Vec::new(),
            region_of: [255; 128],
            strays: HashMap::new(),
            stray_names: HashMap::new(),
            brk: USER_ORG,
            top: top as u32,
            space_built: false,
            used: vec![false; EXTRA_ORG as usize],
            free_cells: 0,
            low_water: 0,
            cards: VecDeque::new(),
            internals: HashMap::new(),
            origins: HashMap::new(),
            lines: HashMap::new(),
            out: String::new(),
            warnings: Vec::new(),
            listing: None,
            conts: Vec::new(),
            gens: Vec::new(),
            structures: Vec::new(),
            level: 1,
            halted: false,
            trace_all: None,
            trace_out: String::new(),
            prim_counts: vec![0; J_SIZE as usize],
            cycles: 0,
        };
        m.add_region(b'H', H0, 6);
        m.add_region(b'J', J_ORG, J_SIZE);
        m.add_region(b'W', W_ORG, W_SIZE);
        for n in 0..J_SIZE {
            m.mem[j(n) as usize] = Word {
                p: 0,
                q: Q_PRIM,
                symb: n,
                link: 0,
            };
        }
        m.mem[H3 as usize] = Word::from_datum(Datum::Int(0));
        m.mem[H5 as usize].symb = j(4);
        m.mem[w(11) as usize] = Word::from_datum(Datum::Int(0));
        m.mem[w(33) as usize] = Word::from_datum(Datum::Int(0));
        // Cells that hold the name of an integer, with the integer they start with.
        let named: [(u32, i64); 13] = [
            (10, 1),  // random number control
            (16, 0),  // input mode
            (17, 0),  // output mode
            (18, 0),  // read unit
            (19, 0),  // write unit
            (20, 0),  // print unit
            (21, 1),  // print column: the left margin
            (22, 1),  // print spacing
            (25, 1),  // entry column
            (30, 5),  // field length
            (31, 2),  // trace mode: selective
            (32, 10), // reserved available space
            (43, 0),  // format
        ];
        for (i, (wn, v)) in named.iter().enumerate() {
            let cell = SYS_INTS + i as u32;
            m.mem[cell as usize] = Word::from_datum(Datum::Int(*v));
            m.mem[w(*wn) as usize] = Word {
                p: 0,
                q: Q_INTERNAL,
                symb: cell,
                link: 0,
            };
        }
        for wn in [12, 13, 14, 15] {
            m.mem[w(wn) as usize].symb = j(0);
        }
        m.mem[SYS_LINE as usize] = Word {
            p: 7,
            q: 7,
            symb: 0,
            link: (LINE_SIZE / 6) as u32,
        };
        m.lines.insert(SYS_LINE, vec![b' '; LINE_SIZE]);
        m.mem[w(24) as usize] = Word {
            p: 0,
            q: Q_INTERNAL,
            symb: SYS_LINE,
            link: 0,
        };
        m.mem[w(34) as usize].symb = H2;
        m
    }

    // ---- regions and the names of symbols -------------------------------

    fn add_region(&mut self, ch: u8, origin: u32, size: u32) {
        self.region_of[(ch & 127) as usize] = self.regions.len() as u8;
        self.regions.push(Region { ch, origin, size });
    }

    pub fn region(&self, ch: u8) -> Option<Region> {
        let i = self.region_of[(ch & 127) as usize];
        (i != 255).then(|| self.regions[i as usize])
    }

    /// A Type-2 card: the next block of `size` consecutive cells becomes
    /// the region (section 18.2).
    pub fn define_region(&mut self, ch: u8, size: u32) -> Res<()> {
        if self.region(ch).is_some() {
            return Err(format!("region {} is already defined", ch as char));
        }
        if self.space_built {
            return Err(format!(
                "region {} is defined after loading has begun",
                ch as char
            ));
        }
        let size = size.max(1);
        if self.brk + size + 16 > self.top {
            return Err(format!(
                "region {} of {} cells does not fit in storage",
                ch as char, size
            ));
        }
        self.add_region(ch, self.brk, size);
        self.brk += size;
        Ok(())
    }

    /// The address of a regional symbol. One mentioned before it is defined
    /// is "used but empty" (section 18.2): its cell holds the internal
    /// symbol 0, as B1 does in the manual's trace.
    ///
    /// A symbol beyond the end of its region, or in a region that has no
    /// Type-2 card, is given a cell of its own from available space; the
    /// loader of 1964 defined only the first symbol of an undeclared region
    /// and rejected the rest.
    pub fn regional(&mut self, ch: u8, n: u32) -> Res<u32> {
        if let Some(r) = self.region(ch) {
            if n < r.size {
                let a = r.origin + n;
                if !SYSTEM.contains(&a) && !self.used[a as usize] {
                    self.used[a as usize] = true;
                    self.mem[a as usize] = Word {
                        q: Q_INTERNAL,
                        ..Word::ZERO
                    };
                }
                return Ok(a);
            }
        }
        if let Some(&a) = self.strays.get(&(ch, n)) {
            return Ok(a);
        }
        let a = self.alloc()?;
        self.mem[a as usize] = Word {
            q: Q_INTERNAL,
            ..Word::ZERO
        };
        self.strays.insert((ch, n), a);
        self.stray_names.insert(a, (ch, n));
        if self.region(ch).is_some() {
            self.warnings.push(format!(
                "{}{} is beyond the end of region {}",
                ch as char, n, ch as char
            ));
        }
        Ok(a)
    }

    /// The address of a regional symbol, if it has one already.
    pub fn lookup(&self, ch: u8, n: u32) -> Option<u32> {
        match self.region(ch) {
            Some(r) if n < r.size => Some(r.origin + n),
            _ => self.strays.get(&(ch, n)).copied(),
        }
    }

    /// The first cell above the programmer's regions.
    pub fn brk(&self) -> u32 {
        self.brk
    }

    /// One past the last cell of the space below the system.
    pub fn top(&self) -> u32 {
        self.top
    }

    pub fn regional_name(&self, a: u32) -> Option<(u8, u32)> {
        if a >= self.brk && !SYSTEM.contains(&a) {
            return self.stray_names.get(&a).copied();
        }
        self.regions
            .iter()
            .find(|r| a >= r.origin && a < r.origin + r.size)
            .map(|r| (r.ch, a - r.origin))
    }

    pub fn is_regional(&self, a: u32) -> bool {
        self.regional_name(a).is_some()
    }

    /// The symbol for an address met where its Q has been lost: regional if
    /// a region covers it, internal otherwise (as J192 and J193 decide).
    pub fn sym(&self, a: u32) -> Sym {
        Sym {
            a,
            q: if self.is_regional(a) {
                Q_REGIONAL
            } else {
                Q_INTERNAL
            },
        }
    }

    /// A symbol as it is printed: `A12`, `9-1350`, `1350`.
    pub fn name(&self, s: Sym) -> String {
        match self.regional_name(s.a) {
            Some((ch, n)) => format!("{}{}", ch as char, n),
            None if s.q == Q_LOCAL => format!("9-{}", s.a),
            None => s.a.to_string(),
        }
    }

    /// A symbol as the programmer wrote it, where that is known.
    pub fn label(&self, a: u32) -> String {
        if let Some((ch, n)) = self.regional_name(a) {
            return format!("{}{}", ch as char, n);
        }
        match self.origins.get(&a) {
            Some(Origin { local: Some(n), .. }) => format!("9-{}", n),
            _ => a.to_string(),
        }
    }

    /// Where an instruction is: `M1`, `M1/9-2`, `M1+7`.
    pub fn place(&self, a: u32) -> String {
        if let Some((ch, n)) = self.regional_name(a) {
            return format!("{}{}", ch as char, n);
        }
        match self.origins.get(&a) {
            Some(o) => match o.local {
                Some(n) => format!("{}/9-{}", o.structure, n),
                None => format!("{}+{}", o.structure, o.offset),
            },
            None => a.to_string(),
        }
    }

    // ---- available space --------------------------------------------------

    /// All storage above the regions goes on H2, in ascending order.
    fn build_space(&mut self) {
        self.space_built = true;
        let cells: Vec<u32> = (self.brk..self.top)
            .chain(EXTRA_ORG..self.mem.len() as u32)
            .collect();
        let mut link = 0;
        for &a in cells.iter().rev() {
            self.mem[a as usize] = Word { link, ..Word::ZERO };
            link = a;
        }
        self.mem[H2 as usize].link = link;
        self.free_cells = cells.len() as u32;
        self.low_water = self.free_cells;
    }

    /// Cells in the programmer's regions, the available space and the lists
    /// made from it: everything but the system.
    pub fn space(&self) -> u32 {
        (self.top - USER_ORG) + (self.mem.len() as u32 - EXTRA_ORG)
    }

    /// The number of cells on the available space list.
    pub fn available(&mut self) -> u32 {
        if !self.space_built {
            self.build_space();
        }
        self.free_cells
    }

    pub fn alloc(&mut self) -> Res<u32> {
        if !self.space_built {
            self.build_space();
        }
        let a = self.mem[H2 as usize].link;
        if a == 0 {
            return Err("available space is exhausted".to_string());
        }
        self.mem[H2 as usize].link = self.mem[a as usize].link;
        self.mem[a as usize] = Word::ZERO;
        self.free_cells -= 1;
        self.low_water = self.low_water.min(self.free_cells);
        Ok(a)
    }

    /// Return a cell to the front of H2. A cell that belongs to a region is
    /// only set empty: its name is permanent, and on H2 it would be handed
    /// out again while the program could still refer to it.
    pub fn free(&mut self, a: u32) {
        if a == 0 {
            return;
        }
        if SYSTEM.contains(&a) || a < USER_ORG {
            // The W cells may be emptied; H, J and the system's cells may not.
            if (W_ORG..W_ORG + W_SIZE).contains(&a) {
                self.mem[a as usize] = Word::ZERO;
            }
            return;
        }
        if a < self.brk || self.stray_names.contains_key(&a) {
            self.mem[a as usize] = Word {
                q: Q_INTERNAL,
                ..Word::ZERO
            };
            return;
        }
        self.origins.remove(&a);
        self.mem[a as usize] = Word {
            link: self.mem[H2 as usize].link,
            ..Word::ZERO
        };
        self.mem[H2 as usize].link = a;
        self.free_cells += 1;
    }

    /// Preserve (section 1.15, 9.1): a new cell takes a copy of the word and
    /// is inserted after the cell.
    pub fn preserve(&mut self, c: u32) -> Res<()> {
        let n = self.alloc()?;
        self.mem[n as usize] = self.mem[c as usize];
        self.mem[c as usize].link = n;
        Ok(())
    }

    /// Restore: the next cell's word moves into the cell, and the next cell
    /// goes back to available space.
    pub fn restore(&mut self, c: u32) {
        let n = self.mem[c as usize].link;
        self.mem[c as usize] = self.mem[n as usize];
        if n != 0 {
            self.free(n);
        }
    }

    // ---- the communication cell and the test cell -------------------------

    /// The symbol n places down in H0: (0), (1), ...
    pub fn arg(&self, n: usize) -> Sym {
        let mut c = H0;
        for _ in 0..n {
            c = self.mem[c as usize].link;
        }
        self.mem[c as usize].sym()
    }

    pub fn push(&mut self, s: Sym) -> Res<()> {
        self.preserve(H0)?;
        let h = &mut self.mem[H0 as usize];
        h.p = 0;
        h.q = s.q;
        h.symb = s.a;
        Ok(())
    }

    pub fn pop(&mut self) -> Sym {
        let s = self.mem[H0 as usize].sym();
        self.restore(H0);
        s
    }

    pub fn drop_args(&mut self, n: usize) {
        for _ in 0..n {
            self.restore(H0);
        }
    }

    pub fn set_h5(&mut self, yes: bool) {
        self.mem[H5 as usize].symb = if yes { j(4) } else { j(3) };
    }

    pub fn h5(&self) -> bool {
        self.mem[H5 as usize].symb != j(3)
    }

    // ---- lists ------------------------------------------------------------

    /// A termination symbol: 0, or the name of a private termination cell
    /// (sections 1.13, 9.4). A cell holding the internal symbol 0 has Q = 4
    /// and is not one.
    pub fn is_term(&self, a: u32) -> bool {
        a == 0 || self.mem[a as usize] == Word::ZERO
    }

    /// The cell after `c` on its list.
    pub fn next(&self, c: u32) -> Option<u32> {
        let l = self.mem[c as usize].link;
        (!self.is_term(l)).then_some(l)
    }

    /// The list cells of a list, without its head.
    pub fn cells_after(&self, head: u32) -> Res<Vec<u32>> {
        let mut v = Vec::new();
        let mut c = head;
        while let Some(n) = self.next(c) {
            v.push(n);
            c = n;
            if v.len() > self.mem.len() {
                return Err(format!("the list {} is circular", self.label(head)));
            }
        }
        Ok(v)
    }

    pub fn last_cell(&self, head: u32) -> Res<u32> {
        Ok(self.cells_after(head)?.last().copied().unwrap_or(head))
    }

    /// The first cell after `head` that holds `s`.
    pub fn locate(&self, head: u32, s: u32) -> Res<Option<u32>> {
        Ok(self
            .cells_after(head)?
            .into_iter()
            .find(|&c| self.mem[c as usize].symb == s))
    }

    /// Put a new cell holding `s` after cell `c`.
    pub fn insert_after(&mut self, c: u32, s: Sym) -> Res<u32> {
        let n = self.alloc()?;
        let l = self.mem[c as usize].link;
        self.mem[n as usize] = Word {
            p: 0,
            q: s.q,
            symb: s.a,
            link: if self.is_term(l) { 0 } else { l },
        };
        if l != 0 && self.is_term(l) {
            self.free(l);
        }
        self.mem[c as usize].link = n;
        Ok(n)
    }

    /// Delete the symbol in cell `c` (J68). False if `c` was the last cell,
    /// which then becomes a private termination cell.
    pub fn delete(&mut self, c: u32) -> bool {
        let l = self.mem[c as usize].link;
        if self.is_term(l) {
            self.free(l);
            self.mem[c as usize] = Word::ZERO;
            false
        } else {
            self.mem[c as usize] = self.mem[l as usize];
            self.free(l);
            true
        }
    }

    /// The lists of a list structure, main list first, in the order of J101:
    /// a list, then the sublists whose local names occur on it.
    pub fn structure(&self, head: u32) -> Res<Vec<u32>> {
        let mut order = vec![head];
        let mut seen = std::collections::HashSet::new();
        seen.insert(head);
        let mut i = 0;
        while i < order.len() {
            let h = order[i];
            i += 1;
            let hw = self.mem[h as usize];
            if hw.is_data() {
                continue;
            }
            let mut cells = vec![h];
            cells.extend(self.cells_after(h)?);
            for c in cells {
                let cw = self.mem[c as usize];
                if cw.q == Q_LOCAL && cw.symb != 0 && seen.insert(cw.symb) {
                    order.push(cw.symb);
                }
            }
        }
        Ok(order)
    }

    /// J72.
    pub fn erase_structure(&mut self, head: u32) -> Res<()> {
        for h in self.structure(head)? {
            self.erase_list(h)?;
        }
        Ok(())
    }

    /// J71.
    pub fn erase_list(&mut self, head: u32) -> Res<()> {
        if self.mem[head as usize].is_data() {
            self.free(head);
            return Ok(());
        }
        let cells = self.cells_after(head)?;
        let tail = cells
            .last()
            .map(|&c| self.mem[c as usize].link)
            .unwrap_or(self.mem[head as usize].link);
        self.free(tail);
        for c in cells {
            self.free(c);
        }
        self.free(head);
        Ok(())
    }

    /// J73: a new list with the same words, from `head` to the end.
    pub fn copy_list(&mut self, head: u32) -> Res<u32> {
        let new = self.alloc()?;
        self.mem[new as usize] = self.mem[head as usize];
        if self.mem[head as usize].is_data() {
            return Ok(new);
        }
        let mut prev = new;
        for c in self.cells_after(head)? {
            let n = self.alloc()?;
            self.mem[n as usize] = self.mem[c as usize];
            self.mem[prev as usize].link = n;
            prev = n;
        }
        self.mem[prev as usize].link = 0;
        Ok(new)
    }

    /// J74.
    pub fn copy_structure(&mut self, head: u32) -> Res<u32> {
        let lists = self.structure(head)?;
        let mut map = HashMap::new();
        for &h in &lists {
            map.insert(h, self.copy_list(h)?);
        }
        for &h in &lists {
            let new = map[&h];
            if self.mem[new as usize].is_data() {
                continue;
            }
            let mut cells = vec![new];
            cells.extend(self.cells_after(new)?);
            for c in cells {
                let cw = self.mem[c as usize];
                if cw.q == Q_LOCAL && cw.symb != 0 {
                    if let Some(&n) = map.get(&cw.symb) {
                        self.mem[c as usize].symb = n;
                    }
                }
            }
        }
        Ok(map[&head])
    }

    // ---- description lists (section 2.6, 6.0) -----------------------------

    /// The cell holding the value of attribute `attr` of list `list`, and
    /// the cell before the attribute's cell.
    pub fn attribute(&self, list: u32, attr: u32) -> Res<Option<(u32, u32, u32)>> {
        let d = self.mem[list as usize].symb;
        if d == 0 || self.mem[list as usize].is_data() {
            return Ok(None);
        }
        let cells = self.cells_after(d)?;
        let mut prev = d;
        for pair in cells.chunks(2) {
            if pair.len() == 2 && self.mem[pair[0] as usize].symb == attr {
                return Ok(Some((prev, pair[0], pair[1])));
            }
            prev = *pair.last().unwrap();
        }
        Ok(None)
    }

    /// J11.
    pub fn assign(&mut self, list: u32, attr: Sym, value: Sym) -> Res<()> {
        if let Some((_, _, v)) = self.attribute(list, attr.a)? {
            let old = self.mem[v as usize];
            if old.q == Q_LOCAL && old.symb != value.a {
                self.erase_structure(old.symb)?;
            }
            self.mem[v as usize].symb = value.a;
            self.mem[v as usize].q = value.q;
            return Ok(());
        }
        let mut d = self.mem[list as usize].symb;
        if d == 0 {
            d = self.alloc()?;
            self.mem[list as usize].symb = d;
            self.mem[list as usize].q = Q_LOCAL;
        }
        let a = self.insert_after(d, attr)?;
        self.insert_after(a, value)?;
        Ok(())
    }

    // ---- data terms -------------------------------------------------------

    pub fn datum(&self, a: u32) -> Option<Datum> {
        self.mem[a as usize].datum()
    }

    pub fn int(&self, a: u32, what: &str) -> Res<i64> {
        match self.datum(a) {
            Some(Datum::Int(i)) => Ok(i),
            Some(Datum::Float(f)) => Ok(f as i64),
            Some(Datum::Octal(o)) => Ok(o as i64),
            _ => Err(format!("{}: {} is not a number", what, self.label(a))),
        }
    }

    pub fn set_int(&mut self, a: u32, v: i64) {
        self.mem[a as usize] = Word::from_datum(Datum::Int(v));
    }

    /// The integer named in a system cell: 1W25, 1W21 and so on.
    pub fn wint(&self, n: u32) -> Res<i64> {
        self.int(self.mem[w(n) as usize].symb, &format!("1W{}", n))
    }

    pub fn set_wint(&mut self, n: u32, v: i64) {
        let a = self.mem[w(n) as usize].symb;
        self.set_int(a, v);
    }

    pub fn new_int(&mut self, v: i64) -> Res<u32> {
        let a = self.alloc()?;
        self.set_int(a, v);
        Ok(a)
    }

    // ---- output -----------------------------------------------------------

    /// Print a line with the spacing named in W22 (section 16.0).
    pub fn print(&mut self, text: &str) {
        let spacing = self.wint(22).unwrap_or(1);
        if spacing >= 2 {
            self.out.push('\n');
        }
        self.out.push_str(text.trim_end());
        self.out.push('\n');
    }

    pub fn line(&mut self) -> &mut Vec<u8> {
        let a = self.mem[w(24) as usize].symb;
        self.lines.entry(a).or_insert_with(|| vec![b' '; LINE_SIZE])
    }
}
