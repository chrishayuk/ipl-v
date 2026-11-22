//! The run the manual prints: *IPL-V Manual*, second edition (1964), Part
//! One, section 2, Fig. 1 (the assembly listing of F1) and Fig. 2 (its
//! trace). `examples/manual-f1.iplv` is that program. The figures were made
//! on the IPL-V system itself, so they are the one place where this
//! implementation can be held against the original instruction by
//! instruction: where each word was loaded, the order of interpretation,
//! the test cell, and the count of interpretation cycles in H3.

use iplv::machine::{j, w, H0, H1, H3, H5};
use iplv::{Machine, DEFAULT_CELLS};

const K1: u32 = 115;
const F1: u32 = 105;
const E1: u32 = 95;

/// One line of Fig. 2: LEVEL, CIA, H5, P, Q, SYMB, LINK, H3.
type Row = (u32, u32, char, u8, u8, u32, u32, i64);

fn figure_2() -> Vec<Row> {
    let mut rows: Vec<Row> = vec![
        (1, E1, '+', 1, 3, K1, 124, 5),
        (1, 124, '+', 0, 0, F1, 125, 6),
        (2, F1, '+', 0, 4, j(90), 126, 7),
        (2, 126, '+', 4, 0, w(0), 127, 9),
        (2, 127, '+', 2, 0, w(0), 128, 10),
    ];
    // Once round the loop for each of the six symbols on K1.
    let h3 = [
        [11, 13, 14, 15, 16, 18],
        [20, 22, 23, 24, 25, 27],
        [29, 31, 32, 33, 34, 36],
        [38, 40, 41, 42, 43, 45],
        [47, 49, 50, 51, 52, 54],
        [56, 58, 59, 60, 61, 63],
    ];
    for round in h3 {
        rows.extend([
            (2, 128, '+', 0, 0, j(60), 129, round[0]),
            (2, 129, '+', 7, 0, 131, 130, round[1]),
            (2, 130, '+', 1, 2, H0, 132, round[2]),
            (2, 132, '+', 1, 1, w(0), 133, round[3]),
            (2, 133, '+', 0, 0, j(6), 134, round[4]),
            (2, 134, '+', 0, 0, j(66), 128, round[5]),
        ]);
    }
    rows.extend([
        (2, 128, '+', 0, 0, j(60), 129, 65),
        (2, 129, '-', 7, 0, 131, 130, 67),
        (2, 131, '-', 5, 1, w(0), 135, 68),
        (2, 135, '-', 3, 0, w(0), 0, 69),
        (1, 125, '-', 0, 0, j(151), 0, 70),
    ]);
    rows
}

fn load() -> Machine {
    let mut m = Machine::new(DEFAULT_CELLS);
    m.trace_all = Some(false);
    m.read_deck(&std::fs::read_to_string("examples/manual-f1.iplv").unwrap());
    let start = m.load().unwrap().unwrap();
    assert_eq!(start, iplv::loader::Start::At(E1));
    let iplv::loader::Start::At(start) = start else {
        panic!("a restart")
    };
    m.start(start);
    m
}

#[test]
fn the_trace_is_figure_2() {
    let mut m = load();
    let mut seen = Vec::new();
    while !m.halted {
        let cia = m.mem[H1 as usize].symb;
        let word = m.mem[cia as usize];
        let h5 = if m.mem[H5 as usize].symb == j(4) { '+' } else { '-' };
        // The figure prints H3 with the instruction counted.
        let h3 = m.int(H3, "H3").unwrap() + 1;
        seen.push((m.level, cia, h5, word.p, word.q, word.symb, word.link, h3));
        m.step().unwrap();
    }
    let expected = figure_2();
    assert_eq!(seen.len(), expected.len());
    for (i, (got, want)) in seen.iter().zip(&expected).enumerate() {
        assert_eq!(got, want, "line {} of Fig. 2", i + 1);
    }
}

#[test]
fn the_words_are_where_figure_1_puts_them() {
    let m = load();
    // The regions, in the order of their Type-2 cards, from cell 64.
    for (ch, origin) in [
        (b'A', 64),
        (b'B', 74),
        (b'C', 84),
        (b'E', 94),
        (b'F', 104),
        (b'K', 114),
    ] {
        assert_eq!(m.region(ch).unwrap().origin, origin);
    }
    // The list K1: an empty head, then B1 C1 B1 A1 A1 B1 in cells 136-141.
    let (a1, b1, c1) = (65, 75, 85);
    let k1 = m.mem[K1 as usize];
    assert_eq!((k1.p, k1.q, k1.symb, k1.link), (0, 4, 0, 136));
    for (i, symbol) in [b1, c1, b1, a1, a1, b1].into_iter().enumerate() {
        let cell = m.mem[136 + i];
        let link = if i == 5 { 0 } else { 137 + i as u32 };
        assert_eq!(
            (cell.p, cell.q, cell.symb, cell.link),
            (0, 0, symbol, link),
            "cell {}",
            136 + i
        );
    }
    // "NUMBER OF CELLS ON AVAILABLE SPACE=21858": cells 142 to 21999.
    let mut m = m;
    m.alloc().unwrap();
    // Counted at the end of loading; the run has since withheld 1W32 cells
    // (ten) for the trap on H2.
    assert_eq!(m.reserve.len(), 10);
    assert_eq!(m.free_cells + 1 + m.reserve.len() as u32, 21858);
}

#[test]
fn the_output_list_is_b1_c1_a1() {
    let mut m = load();
    m.run(1000).unwrap();
    let lines: Vec<&str> = m
        .out
        .lines()
        .map(|l| l.split_whitespace().last().unwrap())
        .collect();
    assert_eq!(lines, ["0", "B1", "C1", "A1"]);
}

/// Fig. 3 (page 34): R3, the first version, finding S4 in L6, from
/// `manual/04-list-structures.md`. The figure gives 27 of the 151 steps,
/// with the level, H1, H5 before the instruction, H0 and W0, in the
/// addresses the manual loaded the deck at (page 33). The deck's cells are
/// given those names by card order.
#[test]
fn the_trace_of_r3_is_figure_3() {
    use std::collections::HashMap;
    let chapter = std::fs::read_to_string("manual/04-list-structures.md").unwrap();
    let deck = chapter
        .split("```iplv\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    let mut m = Machine::new(DEFAULT_CELLS);
    m.trace_all = Some(false);
    m.read_deck(deck);
    let start = m.load().unwrap().unwrap();
    let iplv::loader::Start::At(start) = start else {
        panic!("a restart")
    };
    m.start(start);

    // The manual's name for each cell of P0 (its cells 100 to 103), R3
    // (200 to 213) and L6 (500 to 503, then 600, 700 and 800 onwards).
    let mut cards: Vec<(usize, u32, String, Option<u32>)> = m
        .origins
        .iter()
        .map(|(&a, o)| (o.line, a, o.structure.clone(), o.local))
        .collect();
    cards.sort();
    let mut names: HashMap<u32, String> = HashMap::new();
    let (mut list_base, mut next) = (0u32, 0u32);
    let mut offset: HashMap<String, u32> = HashMap::new();
    for (_, a, structure, local) in cards {
        let k = offset.entry(structure.clone()).or_insert(0);
        let name = match structure.as_str() {
            "P0" => (100 + *k).to_string(),
            "R3" if *k == 0 => "R3".to_string(),
            "R3" => (199 + *k).to_string(),
            "L6" => match local {
                Some(n) => {
                    list_base = 500 + 100 * n;
                    next = list_base;
                    format!("9-{}", n)
                }
                None if *k == 0 => {
                    next = 500;
                    "L6".to_string()
                }
                None => {
                    next += 1;
                    (next - 1).to_string()
                }
            },
            _ => continue,
        };
        let _ = list_base;
        *k += 1;
        names.insert(a, name);
    }
    let name = |m: &Machine, a: u32| names.get(&a).cloned().unwrap_or_else(|| m.label(a));
    // A push-down list, top first, without the word it started with.
    let stack = |m: &Machine, cell: u32, bottom: bool| -> String {
        let mut items = Vec::new();
        let mut c = cell;
        loop {
            items.push(name(m, m.mem[c as usize].symb));
            c = m.mem[c as usize].link;
            if c == 0 {
                break;
            }
        }
        if !bottom {
            items.pop();
        }
        items.join(",")
    };

    // Step, level, H1, H5, H0, W0.
    let figure: [(u32, u32, &str, char, &str, &str); 27] = [
        (1, 1, "100", '+', "", ""),
        (2, 1, "101", '+', "L6", ""),
        (3, 1, "102", '+', "S4,L6", ""),
        (4, 2, "R3,102", '+', "S4,L6", ""),
        (5, 2, "200,102", '+', "L6", "S4"),
        (25, 2, "211,102", '+', "S4,9-1,501", "S4"),
        (26, 3, "R3,211,102", '+', "S4,9-1,501", "S4"),
        (27, 3, "200,211,102", '+', "9-1,501", "S4,S4"),
        (47, 3, "211,211,102", '+', "S4,9-3,601,501", "S4,S4"),
        (48, 4, "R3,211,211,102", '+', "S4,9-3,601,501", "S4,S4"),
        (49, 4, "200,211,211,102", '+', "9-3,601,501", "S4,S4,S4"),
        (85, 4, "200,211,211,102", '-', "803,601,501", "S4,S4,S4"),
        (86, 4, "201,211,211,102", '-', "803,601,501", "S4,S4,S4"),
        (87, 4, "213,211,211,102", '-', "803,601,501", "S4,S4,S4"),
        (88, 3, "212,211,102", '-', "601,501", "S4,S4"),
        (89, 3, "200,211,102", '-', "601,501", "S4,S4"),
        (100, 3, "211,211,102", '+', "S4,9-3,602,501", "S4,S4"),
        (142, 3, "200,211,102", '-', "602,501", "S4,S4"),
        (143, 3, "201,211,102", '+', "603,501", "S4,S4"),
        (144, 3, "202,211,102", '+', "603,501", "S4,S4"),
        (145, 3, "203,211,102", '+', "S4,603,501", "S4,S4"),
        (146, 3, "204,211,102", '+', "S4,S4,603,501", "S4,S4"),
        (147, 3, "205,211,102", '+', "603,501", "S4,S4"),
        (148, 3, "213,211,102", '+', "603,501", "S4,S4"),
        (149, 2, "212,102", '+', "501", "S4"),
        (150, 2, "213,102", '+', "501", "S4"),
        (151, 1, "103", '+', "", ""),
    ];
    let mut rows = figure.iter().peekable();
    let mut step = 0;
    while let Some(&&(at, level, h1, h5, h0, w0)) = rows.peek() {
        step += 1;
        if step == at {
            let got = (
                m.level,
                stack(&m, H1, true),
                if m.mem[H5 as usize].symb == j(4) { '+' } else { '-' },
                stack(&m, H0, false),
                stack(&m, w(0), false),
            );
            assert_eq!(
                got,
                (level, h1.to_string(), h5, h0.to_string(), w0.to_string()),
                "step {} of Fig. 3",
                at
            );
            rows.next();
        }
        m.step().unwrap();
    }
    assert_eq!(step, 151);
}
