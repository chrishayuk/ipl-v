//! Every deck in `examples/` prints what its `.out` file says. A deck with
//! an `.in` file has those cards put in the reader behind it.

use iplv::{Machine, Stop, DEFAULT_CELLS};
use std::fs;
use std::path::Path;

fn run(deck: &Path) -> String {
    let mut m = Machine::new(DEFAULT_CELLS);
    m.read_deck(&fs::read_to_string(deck).unwrap());
    if let Ok(cards) = fs::read_to_string(deck.with_extension("in")) {
        m.read_deck(&cards);
    }
    match iplv::run(&mut m, 10_000_000) {
        Ok(Stop::Halted) => {}
        other => panic!("{}: {:?}", deck.display(), other),
    }
    assert!(m.warnings.is_empty(), "{}: {:?}", deck.display(), m.warnings);
    format!("{}{}", m.trace_out, m.out)
}

#[test]
fn examples_print_their_expected_output() {
    let mut decks: Vec<_> = fs::read_dir("examples")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "iplv"))
        .collect();
    decks.sort();
    assert!(decks.len() >= 11);
    for deck in decks {
        let expected = fs::read_to_string(deck.with_extension("out"))
            .unwrap_or_else(|_| panic!("{} has no .out file", deck.display()));
        assert_eq!(run(&deck), expected, "{}", deck.display());
    }
}

#[test]
fn cards_are_80_columns_of_upper_case() {
    for entry in fs::read_dir("examples").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "iplv") {
            for (i, line) in fs::read_to_string(&path).unwrap().lines().enumerate() {
                assert!(
                    line.len() <= 80,
                    "{}:{} is {} columns",
                    path.display(),
                    i + 1,
                    line.len()
                );
                assert_eq!(line, line.to_uppercase(), "{}:{}", path.display(), i + 1);
            }
        }
    }
}

/// Storage is given back: after a run that builds and erases structures,
/// the cells not on the available space list are those the program's own
/// words occupy.
#[test]
fn erased_structures_return_to_available_space() {
    let out = run(Path::new("examples/structure.iplv"));
    let tail: Vec<&str> = out.lines().rev().take(2).collect();
    assert_eq!(tail, ["0", "14"]);
}
