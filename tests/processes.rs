//! The basic processes, one by one, against their definitions in Part Two
//! of the manual. Inputs are put in H0 and the process is executed
//! directly; what is checked is H0, H5, the words in storage and the count
//! of available space.

use iplv::machine::{j, w, Machine, H0, H2, H5};
use iplv::word::{Datum, Sym, Word, Q_INTERNAL, Q_LOCAL, Q_REGIONAL};

struct T {
    m: Machine,
}

impl T {
    fn new() -> T {
        let mut m = Machine::new(2000);
        m.define_region(b'S', 20, None).unwrap();
        m.define_region(b'A', 20, None).unwrap();
        T { m }
    }
    fn s(&mut self, n: u32) -> Sym {
        Sym {
            a: self.m.regional(b'S', n).unwrap(),
            q: Q_REGIONAL,
        }
    }
    fn a(&mut self, n: u32) -> Sym {
        Sym {
            a: self.m.regional(b'A', n).unwrap(),
            q: Q_REGIONAL,
        }
    }
    /// Execute Jn with these inputs, (0) first.
    fn j(&mut self, n: u32, inputs: &[Sym]) -> &mut T {
        for s in inputs.iter().rev() {
            self.m.push(*s).unwrap();
        }
        self.m.prim(n).unwrap();
        self
    }
    fn top(&self) -> Sym {
        self.m.arg(0)
    }
    fn pop(&mut self) -> Sym {
        self.m.pop()
    }
    fn plus(&self) -> bool {
        self.m.mem[H5 as usize].symb == j(4)
    }
    /// A new list of these symbols (J9n).
    fn list(&mut self, symbols: &[Sym]) -> Sym {
        let mut inputs: Vec<Sym> = symbols.to_vec();
        inputs.reverse();
        self.j(90 + symbols.len() as u32, &inputs);
        self.pop()
    }
    fn symbols(&self, list: Sym) -> Vec<u32> {
        self.m
            .cells_after(list.a)
            .unwrap()
            .iter()
            .map(|&c| self.m.mem[c as usize].symb)
            .collect()
    }
    fn int(&mut self, v: i64) -> Sym {
        Sym {
            a: self.m.new_int(v).unwrap(),
            q: Q_INTERNAL,
        }
    }
    fn stack_is_empty(&self) -> bool {
        self.m.mem[H0 as usize].link == 0
    }
}

#[test]
fn j9n_creates_a_list_in_the_order_of_its_inputs() {
    let mut t = T::new();
    let (s1, s2, s3) = (t.s(1), t.s(2), t.s(3));
    // (2) is first on the list and (0) last.
    t.j(93, &[s3, s2, s1]);
    let l = t.pop();
    assert_eq!(t.symbols(l), [s1.a, s2.a, s3.a]);
    assert_eq!(
        t.m.mem[l.a as usize].symb, 0,
        "the head is empty: the list is describable"
    );
    assert!(t.stack_is_empty());
}

#[test]
fn j60_locates_the_next_cell_and_says_when_there_is_none() {
    let mut t = T::new();
    let (s1, s2) = (t.s(1), t.s(2));
    let l = t.list(&[s1, s2]);
    t.j(60, &[l]);
    let first = t.top();
    assert!(t.plus());
    assert_eq!(t.m.mem[first.a as usize].symb, s1.a);
    t.j(60, &[]).j(60, &[]);
    // No next cell: the output is the last cell, and H5 is -.
    assert!(!t.plus());
    assert_eq!(t.m.mem[t.top().a as usize].symb, s2.a);
}

#[test]
fn j62_j77_search_from_the_cell_after_the_head() {
    let mut t = T::new();
    let (s1, s2, s9) = (t.s(1), t.s(2), t.s(9));
    let l = t.list(&[s1, s2]);
    t.j(62, &[s2, l]);
    assert!(t.plus());
    assert_eq!(
        {
            let c = t.pop().a;
            t.m.mem[c as usize].symb
        },
        s2.a
    );
    t.j(62, &[s9, l]);
    assert!(!t.plus(), "not found: H5 -, and the output is the last cell");
    assert_eq!(
        {
            let c = t.pop().a;
            t.m.mem[c as usize].symb
        },
        s2.a
    );
    t.j(77, &[s1, l]);
    assert!(t.plus());
    t.j(77, &[s9, l]);
    assert!(!t.plus());
    assert!(t.stack_is_empty());
}

#[test]
fn j63_j64_insert_before_and_after_a_symbol() {
    let mut t = T::new();
    let (s1, s2, s3, s4) = (t.s(1), t.s(2), t.s(3), t.s(4));
    let l = t.list(&[s1, s2]);
    let cell = Sym {
        a: t.m.next(l.a).unwrap(),
        q: Q_INTERNAL,
    };
    t.j(63, &[s3, cell]);
    assert_eq!(t.symbols(l), [s3.a, s1.a, s2.a]);
    // The cell now holds S3; inserting after it puts S4 between S3 and S1.
    t.j(64, &[s4, cell]);
    assert_eq!(t.symbols(l), [s3.a, s4.a, s1.a, s2.a]);
    t.j(65, &[s4, l]);
    assert_eq!(t.symbols(l).last(), Some(&s4.a));
    let before = t.symbols(l).len();
    t.j(66, &[s4, l]);
    assert_eq!(
        t.symbols(l).len(),
        before,
        "J66 does not insert a symbol that is on the list"
    );
}

#[test]
fn deleting_the_last_symbol_leaves_a_private_termination_cell() {
    let mut t = T::new();
    let (s1, s2) = (t.s(1), t.s(2));
    let l = t.list(&[s1, s2]);
    let free = t.m.free_cells;
    let cells = t.m.cells_after(l.a).unwrap();
    let last = Sym {
        a: cells[1],
        q: Q_INTERNAL,
    };
    // Section 9.4: the cell cannot be removed, since nothing names the
    // cell that links to it. It becomes a termination cell.
    t.j(68, &[last]);
    assert!(!t.plus());
    assert_eq!(t.m.mem[last.a as usize], Word::ZERO);
    assert_eq!(t.m.free_cells, free);
    assert_eq!(t.symbols(l), [s1.a]);
    // Section 9.5: J60 returns it to available space when it meets it.
    let first = Sym {
        a: cells[0],
        q: Q_INTERNAL,
    };
    t.j(60, &[first]);
    assert!(!t.plus());
    t.pop();
    assert_eq!(t.m.mem[first.a as usize].link, 0);
    assert_eq!(t.m.free_cells, free + 1);
    // Deleting any other symbol moves the next one up and frees its cell.
    t.j(65, &[s2, l]);
    let free = t.m.free_cells;
    t.j(68, &[first]);
    assert!(t.plus());
    assert_eq!(t.symbols(l), [s2.a]);
    assert_eq!(t.m.free_cells, free + 1);
}

#[test]
fn j69_j70_delete_by_symbol_and_from_the_end() {
    let mut t = T::new();
    let (s1, s2, s3) = (t.s(1), t.s(2), t.s(3));
    let l = t.list(&[s1, s2, s3]);
    t.j(69, &[s2, l]);
    assert!(t.plus());
    assert_eq!(t.symbols(l), [s1.a, s3.a]);
    t.j(69, &[s2, l]);
    assert!(!t.plus());
    t.j(70, &[l]);
    assert_eq!(t.symbols(l), [s1.a]);
    t.j(70, &[l]).j(70, &[l]);
    assert!(!t.plus(), "the list was empty");
    t.j(78, &[l]);
    assert!(!t.plus());
}

#[test]
fn description_lists_are_made_changed_and_erased() {
    let mut t = T::new();
    let (thing, colour, size, green, brown) = (t.s(1), t.a(1), t.a(2), t.s(5), t.s(6));
    t.j(10, &[colour, thing]);
    assert!(!t.plus(), "no description list yet");
    assert!(t.stack_is_empty());
    t.j(11, &[colour, green, thing]);
    // J11 made the description list, and gave it a local name.
    assert_eq!(t.m.mem[thing.a as usize].q, Q_LOCAL);
    t.j(10, &[colour, thing]);
    assert!(t.plus());
    assert_eq!(t.pop().a, green.a);
    t.j(11, &[colour, brown, thing]).j(11, &[size, green, thing]);
    t.j(10, &[colour, thing]);
    assert_eq!(t.pop().a, brown.a);
    // A new attribute goes at the front.
    let d = Sym {
        a: t.m.mem[thing.a as usize].symb,
        q: Q_LOCAL,
    };
    assert_eq!(t.symbols(d), [size.a, green.a, colour.a, brown.a]);
    t.j(14, &[size, thing]);
    assert_eq!(t.symbols(d), [colour.a, brown.a]);
    let free = t.m.free_cells;
    t.j(15, &[thing]);
    assert_eq!(t.m.mem[thing.a as usize].symb, 0);
    assert_eq!(t.m.free_cells, free + 3, "the head and both cells come back");
}

#[test]
fn j12_j13_keep_a_list_of_values() {
    let mut t = T::new();
    let (thing, parts, s1, s2, s3) = (t.s(1), t.a(1), t.s(2), t.s(3), t.s(4));
    t.j(13, &[parts, s1, thing])
        .j(13, &[parts, s2, thing])
        .j(12, &[parts, s3, thing]);
    t.j(10, &[parts, thing]);
    let values = t.pop();
    assert_eq!(values.q, Q_LOCAL);
    assert_eq!(t.symbols(values), [s3.a, s1.a, s2.a]);
    // The value list is local, so erasing the attribute erases it too.
    let free = t.m.free_cells;
    t.j(14, &[parts, thing]);
    assert_eq!(t.m.free_cells, free + 2 + 4);
}

#[test]
fn a_copied_list_structure_shares_nothing_local_with_the_original() {
    let mut t = T::new();
    let (s1, s2, s3) = (t.s(1), t.s(2), t.s(3));
    let inner = t.list(&[s2, s3]);
    let inner = Sym {
        a: inner.a,
        q: Q_LOCAL,
    };
    let outer = t.list(&[s1, inner, inner]);
    let free = t.m.free_cells;
    t.j(74, &[outer]);
    let copy = t.pop();
    // Three cells and a head, and the sublist once: two cells and a head.
    assert_eq!(free - t.m.free_cells, 7);
    let cells = t.symbols(copy);
    assert_eq!(cells[0], s1.a, "regional symbols are the same symbols");
    assert_ne!(cells[1], inner.a, "the sublist is a new one");
    assert_eq!(cells[1], cells[2], "and both occurrences name the same copy");
    assert_eq!(
        t.symbols(Sym {
            a: cells[1],
            q: Q_LOCAL
        }),
        [s2.a, s3.a]
    );
    t.j(72, &[copy]);
    assert_eq!(t.m.free_cells, free);
    assert_eq!(t.symbols(outer), [s1.a, inner.a, inner.a]);
}

#[test]
fn j75_j76_divide_a_list_and_join_it_again() {
    let mut t = T::new();
    let (s1, s2, s3) = (t.s(1), t.s(2), t.s(3));
    let l = t.list(&[s1, s2, s3]);
    let first = Sym {
        a: t.m.next(l.a).unwrap(),
        q: Q_INTERNAL,
    };
    t.j(75, &[first]);
    let rest = t.pop();
    assert_eq!(t.symbols(l), [s1.a]);
    assert_eq!(t.symbols(rest), [s2.a, s3.a]);
    let free = t.m.free_cells;
    t.j(76, &[rest, l]);
    assert!(t.plus());
    assert_eq!(
        {
            let c = t.pop().a;
            t.m.mem[c as usize].symb
        },
        s3.a,
        "the output is the last cell inserted"
    );
    assert_eq!(t.symbols(l), [s2.a, s3.a, s1.a]);
    assert_eq!(
        t.m.free_cells,
        free + 1,
        "the head of the inserted list is erased"
    );
}

#[test]
fn j8n_finds_the_nth_symbol() {
    let mut t = T::new();
    let (s1, s2) = (t.s(1), t.s(2));
    let l = t.list(&[s1, s2]);
    t.j(82, &[l]);
    assert!(t.plus());
    assert_eq!(t.pop().a, s2.a);
    t.j(83, &[l]);
    assert!(!t.plus());
    assert!(t.stack_is_empty());
    let two = t.int(2);
    t.j(200, &[two, l]);
    assert!(t.plus());
    assert_eq!(
        {
            let c = t.pop().a;
            t.m.mem[c as usize].symb
        },
        s2.a
    );
}

#[test]
fn arithmetic_puts_the_result_in_the_cell_named_first() {
    let mut t = T::new();
    let (r, seven, two) = (t.int(0), t.int(7), t.int(2));
    t.j(110, &[r, seven, two]);
    assert_eq!(t.top().a, r.a, "the result's name is left in H0");
    assert_eq!(t.m.datum(r.a), Some(Datum::Int(9)));
    t.pop();
    t.j(111, &[r, seven, two]).pop();
    assert_eq!(t.m.datum(r.a), Some(Datum::Int(5)));
    t.j(112, &[r, seven, two]).pop();
    assert_eq!(t.m.datum(r.a), Some(Datum::Int(14)));
    t.j(113, &[r, seven, two]).pop();
    assert_eq!(t.m.datum(r.a), Some(Datum::Int(3)));
    assert_eq!(t.m.datum(w(11)), Some(Datum::Int(1)), "the remainder is in W11");
    // If either factor is floating point, so is the result.
    let half = Sym {
        a: t.m.alloc().unwrap(),
        q: Q_INTERNAL,
    };
    t.m.mem[half.a as usize] = Word::from_datum(Datum::Float(0.5));
    t.j(112, &[r, seven, half]).pop();
    assert_eq!(t.m.datum(r.a), Some(Datum::Float(3.5)));
    t.j(115, &[seven, two]);
    assert!(t.plus());
    t.j(116, &[seven, two]);
    assert!(!t.plus());
    t.j(125, &[two]).pop();
    assert_eq!(t.m.datum(two.a), Some(Datum::Int(3)));
    t.j(123, &[two]).j(119, &[]);
    assert!(t.plus());
    assert!(t.stack_is_empty());
}

#[test]
fn a_list_marked_processed_is_restored_by_restoring_its_head() {
    let mut t = T::new();
    let (s1, s2) = (t.s(1), t.s(2));
    let l = t.list(&[s1, s2]);
    let head = t.m.mem[l.a as usize];
    t.j(133, &[l]);
    assert!(!t.plus());
    t.j(137, &[l]);
    assert_eq!(t.pop().a, l.a);
    t.j(133, &[l]);
    assert!(t.plus());
    assert_eq!(
        (
            t.m.mem[l.a as usize].p,
            t.m.mem[l.a as usize].q,
            t.m.mem[l.a as usize].symb
        ),
        (1, 4, 0)
    );
    t.m.restore(l.a);
    assert_eq!(t.m.mem[l.a as usize], head);
}

#[test]
fn working_cells_are_moved_preserved_and_restored_in_blocks() {
    let mut t = T::new();
    let (s1, s2, s3, s4) = (t.s(1), t.s(2), t.s(3), t.s(4));
    t.j(21, &[s1, s2]);
    assert_eq!(
        (t.m.mem[w(0) as usize].symb, t.m.mem[w(1) as usize].symb),
        (s1.a, s2.a)
    );
    t.j(51, &[s3, s4]);
    assert_eq!(
        (t.m.mem[w(0) as usize].symb, t.m.mem[w(1) as usize].symb),
        (s3.a, s4.a)
    );
    t.j(31, &[]);
    assert_eq!(
        (t.m.mem[w(0) as usize].symb, t.m.mem[w(1) as usize].symb),
        (s1.a, s2.a)
    );
    assert!(t.stack_is_empty());
}

#[test]
fn j126_counts_a_list_and_the_available_space() {
    let mut t = T::new();
    let (s1, s2) = (t.s(1), t.s(2));
    let l = t.list(&[s1, s2, s1]);
    t.j(126, &[l]);
    let n = t.pop();
    assert_eq!(t.m.datum(n.a), Some(Datum::Int(3)));
    let h2 = Sym { a: H2, q: Q_REGIONAL };
    t.j(126, &[h2]);
    let n = t.pop();
    // The count is taken before the cell that holds it.
    assert_eq!(t.m.datum(n.a), Some(Datum::Int(t.m.free_cells as i64 + 1)));
    let mut walked = 0;
    let mut c = t.m.mem[H2 as usize].link;
    while c != 0 {
        walked += 1;
        c = t.m.mem[c as usize].link;
    }
    assert_eq!(walked, t.m.free_cells, "H2 is a list of that many cells");
}
