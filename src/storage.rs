//! The cells of the IPL computer. An address outside storage is a mistake in
//! the program, usually a data term taken for the name of a cell, and the
//! machine must say so and not crash: such a cell reads as an empty word,
//! a write to it goes nowhere, and the address is kept so that the
//! interpreter can stop the run with an error at the end of the step.

use crate::word::Word;
use std::cell::Cell;
use std::ops::{Deref, Index, IndexMut};

#[derive(Clone, Debug)]
pub struct Storage {
    cells: Vec<Word>,
    sink: Word,
    wild: Cell<Option<usize>>,
}

impl Storage {
    pub fn new(cells: Vec<Word>) -> Storage {
        Storage {
            cells,
            sink: Word::ZERO,
            wild: Cell::new(None),
        }
    }

    /// The first address outside storage used since the last call.
    pub fn take_wild(&self) -> Option<usize> {
        self.wild.take()
    }

    pub fn to_vec(&self) -> Vec<Word> {
        self.cells.clone()
    }
}

impl Deref for Storage {
    type Target = [Word];
    fn deref(&self) -> &[Word] {
        &self.cells
    }
}

impl Index<usize> for Storage {
    type Output = Word;
    fn index(&self, a: usize) -> &Word {
        match self.cells.get(a) {
            Some(w) => w,
            None => {
                if self.wild.get().is_none() {
                    self.wild.set(Some(a));
                }
                &Word::ZERO
            }
        }
    }
}

impl IndexMut<usize> for Storage {
    fn index_mut(&mut self, a: usize) -> &mut Word {
        if a < self.cells.len() {
            return &mut self.cells[a];
        }
        if self.wild.get().is_none() {
            self.wild.set(Some(a));
        }
        self.sink = Word::ZERO;
        &mut self.sink
    }
}
