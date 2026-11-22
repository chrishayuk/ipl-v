//! `check.rs`: what is said about a deck that is not run.

use iplv::check::{self, Diag};

const F1: &str = include_str!("../examples/manual-f1.iplv");

/// A card with these fields in their columns.
fn card(name: &str, pq: &str, symb: &str, link: &str) -> String {
    format!("{:40}  {:<5} {:>2}{:<5} {:<5}", "", name, pq, symb, link)
        .trim_end()
        .to_string()
}

fn deck(words: &[String]) -> String {
    let header = format!("{:40}5       00", "");
    let start = format!("{:40}5         P0", "");
    format!("{}\n{}\n{}\n", header, words.join("\n"), start)
}

fn messages(text: &str) -> Vec<String> {
    check::check(text)
        .into_iter()
        .map(|d| format!("{}:{} {}", d.line, d.start + 1, d.message))
        .collect()
}

#[test]
fn the_examples_have_nothing_wrong_with_them() {
    for entry in std::fs::read_dir("examples").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "iplv") {
            let text = std::fs::read_to_string(&path).unwrap();
            assert_eq!(check::check(&text), Vec::<Diag>::new(), "{}", path.display());
            assert_eq!(
                check::format(&text),
                text,
                "{} is already as a keypunch leaves it",
                path.display()
            );
        }
    }
}

#[test]
fn problems_are_found_by_column() {
    let text = deck(&[
        card("P0", "10", "L1", ""),
        card("", "", "J139", ""),
        card("", "", "J162", ""),
        card("", "80", "9-4", ""),
        card("", "", "Q7", "0"),
    ]);
    assert_eq!(
        messages(&text),
        [
            "3:51 there is no basic process J139",
            "4:51 J162 (enter (0) according to format 1w43) is not implemented here",
            "5:49 P is a digit from 0 to 7, or blank",
            "5:51 the local symbol 9-4 is used but never defined in this list structure",
            "6:51 Q7 is executed, but no card defines it",
        ]
    );
    // L1 is only input, never executed: an empty cell is a proper thing to name.
    assert!(!messages(&text).iter().any(|m| m.contains("L1")));
}

#[test]
fn a_comment_that_runs_into_column_41_is_caught() {
    let text = format!(
        "{}\n{}",
        "THIS COMMENT IS ONE CHARACTER TOO LONG, SEE",
        deck(&[card("P0", "", "J0", "0")])
    );
    let found = check::check(&text);
    assert_eq!(found.len(), 1);
    assert!(
        found[0].error && found[0].line == 1 && found[0].start == 40,
        "{:?}",
        found[0]
    );
    assert!(found[0].message.contains("comments end at column 40"));
}

#[test]
fn a_deck_needs_a_header_and_a_start_card() {
    let word = card("P0", "", "J0", "0");
    assert_eq!(
        messages(&format!("{}\n", word)),
        [
            "1:1 a word before any header card (TYPE 5)",
            "1:1 the deck has no start card: TYPE 5 with a routine in SYMB"
        ]
    );
    let text = deck(&[word]).replace("P0\n", "P1\n");
    assert_eq!(
        messages(&text),
        ["3:51 the start card names P1, but no card defines it"]
    );
}

#[test]
fn regions_have_sizes() {
    let region = format!("{:40}2 P                 5", "");
    let text = format!("{}\n{}", region, deck(&[card("P0", "10", "P5", "0")]));
    assert_eq!(
        messages(&text),
        ["3:51 P5 is beyond the end of region P, which has 5 cells"]
    );
}

#[test]
fn a_local_symbol_belongs_to_its_list_structure() {
    // 9-1 in P0 and 9-1 in P1 are different symbols.
    let text = deck(&[
        card("P0", "70", "9-1", ""),
        card("9-1", "", "J0", "0"),
        card("P1", "", "J0", "9-1"),
    ]);
    assert_eq!(
        messages(&text),
        ["4:57 the local symbol 9-1 is used but never defined in this list structure"]
    );
    assert_eq!(check::references(&text, 1, 51).len(), 2);
    assert_eq!(check::definition(&text, 1, 51), Some((2, 42, 45)));
    assert_eq!(check::definition(&text, 3, 57), None);
}

#[test]
fn cards_behind_the_start_card_are_the_programs_to_read() {
    let text = format!(
        "{}THIS IS NOT A CARD OF THE DECK, AND COLUMN 41 MAY HOLD ANYTHING\n",
        deck(&[card("P0", "", "J180", "0")])
    );
    assert!(check::check(&text).is_empty());
    assert!(check::hover(&text, 3, 5)
        .unwrap()
        .0
        .contains("for the program to read"));
}

#[test]
fn hover_says_what_a_card_does() {
    let at = |line, col| check::hover(F1, line, col).unwrap();
    let (text, place) = at(15, 51);
    assert!(text.contains("**13 K1** Input S"), "{}", text);
    assert!(text.contains("monitor point"));
    assert!(
        text.contains("`K1` is defined on line 41: THE LIST K1."),
        "{}",
        text
    );
    assert_eq!(place, (15, 50, 52));
    let (text, _) = at(28, 52);
    assert!(
        text.contains("**J60** LOCATE the next symbol after cell (0). Sets H5."),
        "{}",
        text
    );
    let (text, _) = at(29, 49);
    assert!(text.contains("Branch to S if H5 is -. S is 9-2"), "{}", text);
    let (text, _) = at(30, 51);
    assert!(text.contains("S is the symbol in the cell named in cell H0"));
    assert!(text.contains("`H0` is the communication cell"));
    let (text, _) = at(33, 58);
    assert!(
        text.contains("LINK 9-1: that is the next instruction.")
            && text.contains("`9-1` is defined on line 29"),
        "{}",
        text
    );
    let (text, _) = at(40, 43);
    assert!(
        text.contains("`K1` names this cell. It is used 1 time in the deck."),
        "{}",
        text
    );
    assert!(at(47, 51).0.contains("The start card"));
    assert!(at(9, 40).0.contains("**TYPE 5**"));
    assert!(
        check::hover(F1, 26, 5).is_none(),
        "nothing is said about a comment"
    );
}

#[test]
fn the_outline_is_the_named_list_structures() {
    let outline = check::outline(F1);
    let names: Vec<(&str, bool, usize, usize)> =
        outline.iter().map(|o| (o.0.as_str(), o.1, o.3, o.4)).collect();
    assert_eq!(
        names,
        [("E1", true, 15, 17), ("F1", true, 25, 37), ("K1", false, 40, 46)]
    );
    assert_eq!(outline[2].2, "THE LIST K1.");
}

#[test]
fn a_symbol_is_renamed_in_its_columns() {
    let edits = check::renamed(F1, 15, 51, "k7").unwrap();
    let places: Vec<_> = edits.iter().map(|e| e.0).collect();
    assert_eq!(places, [(15, 50, 55), (40, 42, 47)]);
    assert!(edits.iter().all(|e| e.1 == "K7   "));
    assert!(check::renamed(F1, 28, 43, "9-12").is_some());
    assert!(
        check::renamed(F1, 28, 43, "L1").is_none(),
        "a local symbol stays local"
    );
    assert!(check::renamed(F1, 15, 51, "LONGER").is_none());
    assert!(
        check::renamed(F1, 28, 51, "X1").is_none(),
        "J60 is not ours to rename"
    );
}

#[test]
fn format_is_what_a_keypunch_leaves() {
    let untidy = format!(
        "{:40}5       00\n{:40}  p0       j15   j155  \n{:40}5         p0\n   12\n",
        "a comment", "", ""
    );
    let tidy = check::format(&untidy);
    let lines: Vec<&str> = tidy.lines().collect();
    assert_eq!(lines[0], format!("{:40}5       00", "A COMMENT"));
    assert_eq!(lines[1], card("P0", "", "J15", "J155"));
    assert_eq!(lines[3], "   12", "a card for the program to read is left alone");
    assert!(check::check(&tidy).is_empty());
}

#[test]
fn colours_go_by_column() {
    let tokens = check::semantic(F1);
    let on = |line: usize| -> Vec<(usize, usize, &str)> {
        tokens
            .iter()
            .filter(|t| t.0 == line)
            .map(|t| (t.1, t.2, check::TOKEN_TYPES[t.3]))
            .collect()
    };
    assert_eq!(
        on(28),
        [(0, 31, "comment"), (42, 3, "variable"), (50, 3, "macro")]
    );
    assert_eq!(
        on(15),
        [(42, 2, "function"), (48, 2, "operator"), (50, 2, "variable")]
    );
    assert_eq!(on(16), [(0, 14, "comment"), (50, 2, "function")]);
    assert_eq!(on(47), [(40, 1, "keyword"), (50, 2, "function")]);
    // In order, as the protocol needs.
    assert!(tokens.windows(2).all(|w| (w[0].0, w[0].1) < (w[1].0, w[1].1)));
}
