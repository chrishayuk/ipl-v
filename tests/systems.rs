//! The parts of the IPL computer outside the basic list processes: tapes,
//! blocks, regions, auxiliary storage, restart and the loader's special
//! cards (manual, Part Two, sections 10, 13, 14, 17 to 20).

use iplv::card::Card;
use iplv::machine::Machine;
use iplv::tape::{Record, Tape};
use iplv::word::{Sym, Q_INTERNAL, Q_REGIONAL};
use iplv::{Stop, DEFAULT_CELLS};

/// A card from its fields, in their columns: TYPE in 41, NAME from 43,
/// SIGN in 48, PQ in 49 and 50, SYMB from 51, LINK from 57.
fn card(typ: &str, name: &str, pq: &str, symb: &str, link: &str) -> String {
    let mut c = [b' '; 61];
    let mut put = |col: usize, s: &str| c[col - 1..col - 1 + s.len()].copy_from_slice(s.as_bytes());
    put(41, typ);
    put(43, name);
    put(48, &format!("{:>3}", pq));
    put(51, symb);
    put(57, link);
    String::from_utf8(c.to_vec()).unwrap().trim_end().to_string()
}

fn word(name: &str, pq: &str, symb: &str, link: &str) -> String {
    card("", name, pq, symb, link)
}

fn machine(deck: &[String]) -> Machine {
    let mut m = Machine::new(DEFAULT_CELLS);
    m.read_deck(&deck.join("\n"));
    m
}

fn run(m: &mut Machine) -> String {
    match iplv::run(m, 1_000_000) {
        Ok(Stop::Halted) => {}
        other => panic!("{:?}\n{}", other, m.out),
    }
    m.out.clone()
}

fn regions(letters: &str) -> Vec<String> {
    letters
        .chars()
        .map(|ch| card("2", &ch.to_string(), "", "", "10"))
        .collect()
}

#[test]
fn a_tape_is_the_same_after_a_trip_through_its_file() {
    let mut m = machine(&[]);
    let t = m.tape(3);
    t.records
        .push(Record::Card(Card::parse(&word("L1", "", "A1", "0"), 1)));
    t.records.push(Record::EndOfFile);
    t.records.push(Record::Card(Card::parse("", 2)));
    let image = Box::new(m.snapshot());
    let t = m.tape(3);
    t.records.push(Record::Image(image));
    let text = t.to_text();
    let again = Tape::from_text(&text).unwrap();
    assert_eq!(again.records.len(), 4);
    assert_eq!(again.to_text(), text);
}

#[test]
fn a_storage_image_is_the_same_after_a_trip_through_its_text() {
    // Part way through the example of auxiliary storage: a structure is
    // filed, and routines are in progress.
    let mut m = Machine::new(DEFAULT_CELLS);
    m.read_deck(include_str!("../examples/auxiliary-storage.iplv"));
    let Some(iplv::loader::Start::At(start)) = m.load().unwrap() else {
        panic!()
    };
    m.start(start);
    m.run(40).unwrap();
    assert!(m.aux.iter().any(|s| !s.filed.is_empty()));
    let image = m.snapshot();
    let text = image.to_text();
    assert_eq!(iplv::restart::Image::from_text(&text).unwrap(), image);
}

#[test]
fn a_restart_tape_carries_a_run_into_the_next() {
    // The first run saves on tape 1 and goes on; the second is given only
    // the tape and a restart card, and goes on from the J166 with H5 minus.
    let deck = include_str!("../examples/save-for-restart.iplv");
    let mut first = Machine::new(DEFAULT_CELLS);
    first.read_deck(deck);
    assert_eq!(run(&mut first), "SAVED\nAGAIN\nP9\n");
    let tape = first.tapes.remove(&1).unwrap();
    let tape = Tape::from_text(&tape.to_text()).unwrap();

    let mut second = machine(&[card("5", "", "40", "1", ""), card("5", "", "", "P9", "")]);
    second.tapes.insert(1, tape);
    assert_eq!(run(&mut second), "AGAIN\nP9\n");
}

#[test]
fn j144_skips_to_the_next_file_and_j180_reads_a_tape() {
    let mut m = machine(&[]);
    let t = m.tape(2);
    for text in ["FIRST", "SECOND"] {
        t.records.push(Record::Card(Card::parse(text, 1)));
    }
    t.records.push(Record::EndOfFile);
    t.records.push(Record::Card(Card::parse("THIRD", 1)));
    m.set_wint(18, 2);
    m.prim(144).unwrap();
    m.prim(180).unwrap();
    assert!(m.h5());
    assert_eq!(&m.line()[..5], b"THIRD");
    m.prim(180).unwrap();
    assert!(!m.h5(), "the end of the tape");
}

#[test]
fn j176_spaces_a_tape_back_and_on() {
    let mut m = machine(&[]);
    m.set_wint(19, 4);
    for _ in 0..3 {
        m.prim(145).unwrap();
    }
    assert_eq!(m.tape(4).pos, 3);
    let back = m.alloc().unwrap();
    m.mem[back as usize] = iplv::word::Word::from_datum(iplv::word::Datum::Int(-2));
    m.push(Sym {
        a: back,
        q: Q_INTERNAL,
    })
    .unwrap();
    m.prim(176).unwrap();
    assert_eq!(m.tape(4).pos, 1);
    m.mem[back as usize] = iplv::word::Word::from_datum(iplv::word::Datum::Int(9));
    m.push(Sym {
        a: back,
        q: Q_INTERNAL,
    })
    .unwrap();
    m.prim(176).unwrap();
    assert_eq!(m.tape(4).pos, 3, "not past the end");
}

#[test]
fn j167_passes_over_a_list_structure_that_j140_would_read() {
    let mut deck = regions("LP");
    deck.extend([
        card("5", "", "", "", ""),
        word("P0", "", "J167", ""),
        word("", "70", "0", ""),
        word("", "", "J140", ""),
        word("", "70", "0", ""),
        word("", "", "J150", "0"),
        card("5", "", "", "P0", ""),
        word("L1", "", "0", ""),
        word("", "", "L1", "0"),
        word("L2", "", "0", ""),
        word("", "", "L2", "0"),
    ]);
    let mut m = machine(&deck);
    assert_eq!(run(&mut m), "L2       0\n         L2\n");
}

#[test]
fn j173_will_not_read_a_block_into_another() {
    let mut deck = regions("KPX");
    deck.extend([
        card("3", "X1", "02", "", "3"),
        card("3", "X2", "02", "", "3"),
        card("5", "", "", "", ""),
        word("P0", "10", "X1", ""),
        word("", "", "J174", ""),
        word("", "10", "K1", ""),
        word("", "", "J143", ""),
        word("", "10", "X2", ""),
        word("", "", "J173", "0"),
        card("5", "", "01", "", ""),
        word("K1", "01", "1", ""),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    m.set_wint(19, 1);
    run(&mut m);
    assert!(!m.h5());
    let x2 = m.lookup(b'X', 2).unwrap();
    assert_eq!(m.arg(0).a, x2, "(0) is left in H0 for the trap");
}

#[test]
fn j189_moves_a_field_to_another_line() {
    let mut deck = regions("PT");
    deck.extend([
        card("3", "T1", "01", "", "2"),
        card("5", "", "", "", ""),
        word("P0", "10", "T1", ""),
        word("", "", "J189", ""),
        word("", "10", "T1", ""),
        word("", "20", "W24", ""),
        word("", "", "J155", "0"),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    m.read_deck("HELLO WORLD");
    // Read the card into the line first, then take 5 columns from column 7.
    m.load().unwrap();
    m.prim(180).unwrap();
    m.set_wint(25, 7);
    m.set_wint(30, 5);
    m.set_wint(21, 3);
    let t1 = m.lookup(b'T', 1).unwrap();
    m.push(Sym { a: t1, q: Q_REGIONAL }).unwrap();
    m.prim(189).unwrap();
    assert!(m.h5());
    assert_eq!(&m.lines[&t1][..8], b"  WORLD ");
    assert_eq!(m.wint(25).unwrap(), 12);
}

#[test]
fn a_type_3_card_with_q_4_sets_the_size_of_available_space() {
    let mut deck = regions("P");
    deck.extend([
        card("3", "", "04", "", "500"),
        card("5", "", "", "", ""),
        word("P0", "", "J0", "0"),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    run(&mut m);
    // P0 is in its region, not on H2; ten cells are withheld for the trap.
    assert_eq!(m.free_cells + 10, 500);
}

#[test]
fn j171_gives_back_the_regional_cells_never_used() {
    let mut deck = regions("P");
    deck.extend([
        card("5", "", "", "", ""),
        word("P0", "", "J171", "0"),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    m.load().unwrap();
    m.withhold();
    let before = m.free_cells;
    let p0 = m.lookup(b'P', 0).unwrap();
    m.start(p0);
    m.run(100).unwrap();
    let region = m.regions.iter().find(|r| r.ch == b'P').unwrap();
    assert_eq!(region.size, 1, "P0 is the highest symbol used");
    assert_eq!(m.free_cells - before, 9, "the other nine cells of region P");
    let word = m.mem[m.region_word(b'P').unwrap() as usize];
    assert_eq!((word.p, word.q, word.link), (7, 7, 1));
}

#[test]
fn data_under_a_type_6_header_is_filed_in_fast_auxiliary() {
    let mut deck = regions("LP");
    deck.extend([
        card("5", "", "", "", ""),
        word("P0", "10", "L1", ""),
        word("", "", "J108", "0"),
        card("6", "", "01", "", ""),
        word("L1", "", "0", ""),
        word("", "", "L1", "0"),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    run(&mut m);
    assert!(m.h5(), "J108 finds L1 on auxiliary");
    let l1 = m.lookup(b'L', 1).unwrap();
    assert_eq!(m.mem[l1 as usize].q, 6);
    assert_eq!(m.wint(40).unwrap(), 1, "one structure in fast auxiliary");
}

#[test]
fn a_filed_structure_comes_back_with_j105_as_it_went() {
    let mut deck = regions("LP");
    deck.extend([
        card("5", "", "", "", ""),
        word("P0", "10", "L1", ""),
        word("", "", "J107", ""),
        word("", "10", "L1", ""),
        word("", "", "J105", "0"),
        card("5", "", "01", "", ""),
        word("L1", "", "0", ""),
        word("", "", "9-1", ""),
        word("", "", "L1", "0"),
        word("9-1", "", "0", ""),
        word("", "", "H2", "0"),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    let l1 = {
        m.load().unwrap();
        m.lookup(b'L', 1).unwrap()
    };
    let before = m.structure(l1).unwrap().len();
    let mut m = machine(&deck);
    run(&mut m);
    assert!(m.h5(), "J105 found it on auxiliary");
    assert_eq!(m.arg(0).a, l1, "the output (0) is the input (0)");
    assert_eq!(m.structure(l1).unwrap().len(), before);
    // One obsolete image of one is over the 25 per cent in W37, so the
    // system's test in W38 compacts at once.
    assert_eq!((m.wint(35).unwrap(), m.wint(36).unwrap()), (0, 0));
}

#[test]
fn a_data_term_taken_for_a_cell_stops_the_run_and_does_not_crash() {
    // J60 on an alphabetic data term takes its bits for a LINK; the step
    // that then uses that address ends the run with an error.
    let mut deck = regions("KP");
    deck.extend([
        card("5", "", "", "", ""),
        word("P0", "10", "K1", ""),
        word("", "", "J60", ""),
        word("", "11", "H0", ""),
        word("", "", "J60", "0"),
        card("5", "", "01", "", ""),
        word("K1", "21", "ABCDE", ""),
        card("5", "", "", "P0", ""),
    ]);
    let mut m = machine(&deck);
    let e = iplv::run(&mut m, 1000).unwrap_err();
    assert!(e.contains("is used as the address of a cell"), "{}", e);
}
