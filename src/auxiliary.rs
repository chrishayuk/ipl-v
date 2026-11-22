//! Auxiliary storage for data list structures (manual section 10.1).
//!
//! The IPL computer had two auxiliary stores, fast (a drum) and slow (tape or
//! disk). A data list structure filed in one leaves main storage, all but its
//! head, which becomes the control word that finds it again: Q = 6 for fast,
//! Q = 7 for slow, the store's number in SYMB and the structure's place in it
//! in LINK. The image is kept in relocatable form, so that moving it back in
//! may use any cells. Moving in, or erasing, leaves the image obsolete; W35
//! to W38 (slow) and W39 to W42 (fast) count the structures and say when to
//! compact.
//!
//! Routines in auxiliary storage (Type-6 and Type-7 headers on sets of
//! routines) are loaded into main storage here. The buffer they shared was a
//! way of saving main storage, and the manual's rules for them ("no auxiliary
//! routine shall modify itself") make the difference invisible to a program
//! that keeps them; H4, which tracked the buffer, is not used.

use crate::interp::{Cont, Outcome};
use crate::machine::*;
use crate::word::*;
use std::collections::HashMap;

/// One structure in an auxiliary store: its cells, head first, with each
/// SYMB and LINK that named one of its own cells replaced by that cell's
/// index.
#[derive(Clone, Debug, PartialEq)]
pub struct Filed {
    pub words: Vec<Word>,
    pub inner: Vec<(bool, bool)>,
    pub current: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Store {
    pub filed: Vec<Option<Filed>>,
}

/// SLOW is store 0 and FAST store 1, as the input to J109 numbers them.
pub const SLOW: usize = 0;
pub const FAST: usize = 1;

/// The W cells of each store: obsolete count, total count, the percentage
/// that sets off compacting, and the routine that decides.
fn cells(store: usize) -> (u32, u32, u32, u32) {
    if store == SLOW {
        (35, 36, 37, 38)
    } else {
        (39, 40, 41, 42)
    }
}

impl Machine {
    /// Is the cell the head of a structure in auxiliary storage?
    pub fn is_filed(&self, head: u32) -> bool {
        matches!(self.mem.get(head as usize).map(|w| w.q), Some(6 | 7)) && !self.mem[head as usize].is_data()
    }

    fn counts(&mut self, store: usize) -> Res<()> {
        let (obsolete, total, _, _) = cells(store);
        let s = &self.aux[store];
        let total_n = s.filed.iter().flatten().count() as i64;
        let obsolete_n = s.filed.iter().flatten().filter(|f| !f.current).count() as i64;
        self.set_wint(obsolete, obsolete_n);
        self.set_wint(total, total_n);
        Ok(())
    }

    /// J106 and J107: file list structure `head` in a store.
    pub fn file_structure(&mut self, head: u32, store: usize) -> Res<()> {
        if self.is_filed(head) {
            return Err(format!("{} is already in auxiliary storage", self.label(head)));
        }
        let mut cells = Vec::new();
        for l in self.structure(head)? {
            cells.push(l);
            if !self.mem[l as usize].is_data() {
                cells.extend(self.cells_after(l)?);
            }
        }
        let index: HashMap<u32, u32> = cells.iter().enumerate().map(|(i, &c)| (c, i as u32)).collect();
        let mut words = Vec::with_capacity(cells.len());
        let mut inner = Vec::with_capacity(cells.len());
        for &c in &cells {
            let mut w = self.mem[c as usize];
            let mut flags = (false, false);
            if !w.is_data() {
                if let Some(&i) = index.get(&w.symb) {
                    w.symb = i;
                    flags.0 = true;
                }
                if let Some(&i) = index.get(&w.link) {
                    w.link = i;
                    flags.1 = true;
                }
            }
            words.push(w);
            inner.push(flags);
        }
        for &c in &cells[1..] {
            self.free(c);
        }
        let place = self.aux[store].filed.len() as u32;
        self.aux[store].filed.push(Some(Filed {
            words,
            inner,
            current: true,
        }));
        self.mem[head as usize] = Word {
            p: 0,
            q: if store == FAST { 6 } else { 7 },
            symb: store as u32,
            link: place,
        };
        self.counts(store)
    }

    fn filed(&self, head: u32) -> Res<(usize, usize, Filed)> {
        let w = self.mem[head as usize];
        let store = if w.q == 6 { FAST } else { SLOW };
        match self.aux[store].filed.get(w.link as usize) {
            Some(Some(f)) => Ok((store, w.link as usize, f.clone())),
            _ => Err(format!(
                "the control word in {} names no structure in auxiliary storage",
                self.label(head)
            )),
        }
    }

    /// The image put into cells from available space, with `head` as its
    /// head cell.
    fn unpack(&mut self, f: &Filed, head: u32) -> Res<()> {
        let mut at = vec![head];
        for _ in 1..f.words.len() {
            at.push(self.alloc()?);
        }
        for (i, (w, (s, l))) in f.words.iter().zip(&f.inner).enumerate() {
            let mut w = *w;
            if *s {
                w.symb = at[w.symb as usize];
            }
            if *l {
                w.link = at[w.link as usize];
            }
            self.mem[at[i] as usize] = w;
        }
        Ok(())
    }

    /// Move a filed structure back into main storage; its image is left
    /// obsolete. Returns the store it was in.
    pub fn move_in(&mut self, head: u32) -> Res<usize> {
        let (store, place, f) = self.filed(head)?;
        self.unpack(&f, head)?;
        if let Some(Some(g)) = self.aux[store].filed.get_mut(place) {
            g.current = false;
        }
        self.counts(store)?;
        Ok(store)
    }

    /// J74 on a filed structure: a copy in main storage; the filed one stays.
    pub fn copy_filed(&mut self, head: u32) -> Res<u32> {
        let (_, _, f) = self.filed(head)?;
        let new = self.alloc()?;
        self.unpack(&f, new)?;
        Ok(new)
    }

    /// J72 on a filed structure: the head goes, and the image is obsolete.
    pub fn erase_filed(&mut self, head: u32) -> Res<()> {
        let (store, place, _) = self.filed(head)?;
        if let Some(Some(g)) = self.aux[store].filed.get_mut(place) {
            g.current = false;
        }
        self.mem[head as usize] = Word::ZERO;
        self.free(head);
        self.counts(store)
    }

    /// J109: purge the obsolete images. A place in the store is never used
    /// again, so the control words of current structures stay right.
    pub fn compact_store(&mut self, store: usize) -> Res<()> {
        for slot in self.aux[store].filed.iter_mut() {
            if slot.as_ref().is_some_and(|f| !f.current) {
                *slot = None;
            }
        }
        self.counts(store)
    }

    /// After J105: the routine in W38 (or W42) decides whether to compact.
    /// J0 there, as at the start, means the system's own test, against the
    /// percentage in W37 (or W41).
    fn after_move(&mut self, store: usize) -> Res<Outcome> {
        let (obsolete, total, percent, routine) = cells(store);
        let r = self.mem[w(routine) as usize].symb;
        if r != j(0) && r != 0 {
            return Ok(Outcome::Call(r, Cont::H5Plus));
        }
        let (o, t, p) = (self.wint(obsolete)?, self.wint(total)?, self.wint(percent)?);
        if o * 100 > t * p {
            self.compact_store(store)?;
        }
        self.set_h5(true);
        Ok(Outcome::Done)
    }

    /// J105 to J109.
    pub fn aux_prim(&mut self, n: u32) -> Res<Outcome> {
        match n {
            105 => {
                let head = self.arg(0).a;
                if !self.is_filed(head) {
                    self.set_h5(false);
                    return Ok(Outcome::Done);
                }
                let store = self.move_in(head)?;
                return self.after_move(store);
            }
            106 | 107 => {
                let head = self.pop().a;
                self.file_structure(head, if n == 106 { FAST } else { SLOW })?;
            }
            108 => {
                let head = self.pop().a;
                let filed = self.is_filed(head);
                self.set_h5(filed);
            }
            109 => {
                let s = self.pop().a;
                match self.int(s, "J109")? {
                    0 => self.compact_store(SLOW)?,
                    1 => self.compact_store(FAST)?,
                    k => return Err(format!("J109: {} is not 0 (slow) or 1 (fast)", k)),
                }
            }
            _ => return Err(format!("J{} is not implemented", n)),
        }
        Ok(Outcome::Done)
    }

    /// The processes that move a filed structure in before they work on it
    /// (sections 9.8, 16.2): the input that names it, if any.
    pub fn moves_in(&mut self, n: u32) -> Res<()> {
        let k = match n {
            100..=102 => 1,
            150 => 0,
            _ => return Ok(()),
        };
        let head = self.arg(k).a;
        if self.is_filed(head) {
            self.move_in(head)?;
        }
        Ok(())
    }
}
