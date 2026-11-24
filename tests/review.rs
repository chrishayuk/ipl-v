//! Regressions for the findings of the independent review of November 2026
//! (`docs/independent-review.md`). Each test is named for what the manual
//! says, and the comment gives the finding and the page.

use iplv::machine::Machine;
use iplv::word::float_text;
use iplv::Stop;

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

fn run(cells: usize, deck: &[String]) -> Result<String, String> {
    let mut m = Machine::new(cells);
    m.read_deck(&deck.join("\n"));
    match iplv::run(&mut m, 1_000_000)? {
        Stop::Halted => Ok(m.out.clone()),
        other => Err(format!("{:?}", other)),
    }
}

/// The count of available space, printed by J126 on H2, with or without a
/// print line of `line` words reserved.
fn available(line: Option<u32>) -> Result<String, String> {
    let mut deck = vec![card("2", "P", "", "", "1"), card("2", "X", "", "", "2")];
    if let Some(n) = line {
        deck.push(card("3", "X1", "01", "", &n.to_string()));
    }
    deck.extend([
        card("5", "", "00", "", ""),
        word("P0", "10", "H2", ""),
        word("", "", "J126", ""),
        word("", "", "J153", "0"),
        card("5", "", "", "P0", ""),
    ]);
    run(1000, &deck)
}

/// Finding 1. Section 18.3, page 222: the words of a print line "are taken
/// from available storage, not from the region".
#[test]
fn a_print_line_takes_its_words_from_available_storage() {
    let without: i64 = available(None).unwrap().trim().parse().unwrap();
    let with: i64 = available(Some(20)).unwrap().trim().parse().unwrap();
    assert_eq!(without - with, 20);
    // A line bigger than storage cannot be reserved.
    let too_big = available(Some(1000)).unwrap_err();
    assert!(too_big.contains("do not fit"), "{}", too_big);
}

/// Finding 2. J180, page 236, reads the card and sets H5; it does not move
/// 1W25. With 1W25 = 0, as R96 leaves it, J184 scans from column 1.
#[test]
fn j180_leaves_the_entry_column_where_it_was() {
    let deck = [
        card("2", "M", "", "", "1"),
        card("2", "P", "", "", "1"),
        card("5", "", "01", "", ""),
        word("M1", "-01", "", "1"),
        card("5", "", "00", "", ""),
        // 1W25 = 1W21 - 1 = 0.
        word("P0", "", "J154", ""),
        word("", "10", "M1", ""),
        word("", "", "J161", ""),
        word("", "", "J180", ""),
        // Set 1W25 to the first non-blank column, as R95 does.
        word("", "11", "W25", ""),
        word("", "", "J184", ""),
        word("", "11", "W25", ""),
        word("", "", "J153", "0"),
        card("5", "", "", "P0", ""),
        "HELLO".to_string(),
    ];
    assert_eq!(run(2000, &deck).unwrap().trim(), "1");
}

/// Finding 3. J123, page 196: "Zero is signed; J123 takes zero into minus
/// zero." J114, page 195, tests identity, so minus zero is not plus zero;
/// J117 still finds it zero, and J123 takes it back to plus zero.
#[test]
fn j123_takes_zero_into_minus_zero() {
    let deck = [
        card("2", "N", "", "", "2"),
        card("2", "P", "", "", "1"),
        card("2", "S", "", "", "1"),
        card("2", "Y", "", "", "2"),
        card("5", "", "00", "", ""),
        // J123 on N0, then N0 = N1?
        word("P0", "10", "N0", ""),
        word("", "", "J123", ""),
        word("", "10", "N1", ""),
        word("", "", "J114", ""),
        word("", "", "S0", ""),
        // Minus zero prints as -0. Is it zero?
        word("", "10", "N0", ""),
        word("", "", "J153", ""),
        word("", "10", "N0", ""),
        word("", "", "J117", ""),
        word("", "", "S0", ""),
        // J123 again: is N0 now the same as N1?
        word("", "10", "N0", ""),
        word("", "", "J123", ""),
        word("", "10", "N1", ""),
        word("", "", "J114", ""),
        word("", "", "S0", "0"),
        // S0 prints the sign of H5.
        word("S0", "70", "9-1", ""),
        word("", "10", "Y0", ""),
        word("", "", "J153", "0"),
        word("9-1", "10", "Y1", ""),
        word("", "", "J153", "0"),
        card("5", "", "01", "", ""),
        word("N0", "+01", "", "0"),
        word("N1", "+01", "", "0"),
        word("Y0", "+21", "+", ""),
        word("Y1", "+21", "-", ""),
        card("5", "", "", "P0", ""),
    ];
    let out = run(2000, &deck).unwrap();
    let lines: Vec<&str> = out.lines().map(str::trim).collect();
    assert_eq!(lines, ["-", "-0", "+", "+"]);
}

/// A minus sign punched on an integer zero loads as minus zero.
#[test]
fn minus_zero_can_be_punched() {
    let deck = [
        card("2", "N", "", "", "1"),
        card("2", "P", "", "", "1"),
        card("5", "", "00", "", ""),
        word("P0", "10", "N0", ""),
        word("", "", "J153", "0"),
        card("5", "", "01", "", ""),
        word("N0", "-01", "", "0"),
        card("5", "", "", "P0", ""),
    ];
    assert_eq!(run(2000, &deck).unwrap().trim(), "-0");
}

/// Finding 5. Section 16.2, page 215: ".505135x10^5 entered as 505135 05
/// (9 cols.); .14x10^-16 entered as 140000 -16 (10 cols.)".
#[test]
fn a_floating_point_number_keeps_the_blank_before_its_exponent() {
    assert_eq!(float_text(0.505135e5), "505135 05");
    assert_eq!(float_text(0.14e-16), "140000 -16");
    assert_eq!(float_text(-0.33e-29), "-330000 -29");
}

/// Finding 6. J151, page 214: local symbols "are not translated to form
/// 9dddd; but instead, their addresses are printed, and the Q = 2
/// identifies them as locals." J152 handles locals as J151 does.
#[test]
fn j151_prints_a_local_symbol_as_its_address_with_q_2() {
    let deck = [
        card("2", "P", "", "", "1"),
        card("2", "L", "", "", "1"),
        card("5", "", "00", "", ""),
        word("P0", "10", "L0", ""),
        word("", "", "J151", ""),
        word("", "10", "L0", ""),
        word("", "", "J60", ""),
        word("", "12", "H0", ""),
        word("", "", "J152", "0"),
        card("5", "", "01", "", ""),
        word("L0", "", "0", ""),
        word("", "", "9-1", "0"),
        word("9-1", "", "0", "0"),
        card("5", "", "", "P0", ""),
    ];
    let out = run(1000, &deck).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3, "{}", out);
    assert!(!out.contains("9-"), "{}", out);
    let address = lines[1].trim().strip_prefix("02 ").expect(lines[1]);
    assert!(address.parse::<u32>().is_ok(), "{}", out);
    assert_eq!(lines[2].trim(), format!("2 {}", address));
}

/// Fig. 2, page 140: a floating point data term has its fraction in SYMB,
/// columns 51-55, and its sixth digit in column 57, the first of LINK; the
/// shaded column 56 is not used. The loader read 51-56, so .123456 punched
/// as the figure shows came in as .12345, and the writer of J142 put the
/// sixth digit in column 56. Found in writing TPK (examples/tpk.iplv).
#[test]
fn a_floating_point_data_term_has_its_sixth_digit_in_column_57() {
    let deck = [
        card("2", "K", "", "", "10"),
        card("2", "P", "", "", "10"),
        card("5", "", "00", "", ""),
        word("P0", "", "J0", "0"),
        card("5", "", "01", "", ""),
        word("K1", "11", "12345", "6  00"),
        word("K2", "11", "75", "  -02"),
        card("5", "", "", "P0", ""),
    ];
    let mut m = Machine::new(1000);
    m.read_deck(&deck.join("\n"));
    m.load().unwrap();
    let value = |m: &Machine, n| match m.mem[m.lookup(b'K', n).unwrap() as usize].datum() {
        Some(iplv::word::Datum::Float(f)) => f,
        other => panic!("K{} is {:?}", n, other),
    };
    assert_eq!(value(&m, 1), 0.123456);
    assert_eq!(value(&m, 2), 0.75e-2);
    // Punched again, the sixth digit goes back to column 57.
    let k1 = m.lookup(b'K', 1).unwrap();
    let cards = m.punch_structure(k1).unwrap();
    let c = cards.iter().find(|c| c.len() > 50 && &c[50..55] == "12345").expect("a card for K1");
    let col = |n: usize| c.as_bytes().get(n - 1).copied().unwrap_or(b' ') as char;
    assert_eq!((col(56), col(57), col(60), col(61)), (' ', '6', '0', '0'), "{}", c);
}

/// Fig. 2, page 140, the figure's own examples: a decimal integer has its
/// digits in 52-55 and 58-61 (-12,192 is `1` in 55 and `2192` in 58-61),
/// an octal number in 51-55 and 57-61. J142's writer ran both across 51-61.
#[test]
fn integers_and_octal_numbers_are_punched_in_the_columns_of_fig_2() {
    let deck = [
        card("2", "K", "", "", "10"),
        card("2", "P", "", "", "10"),
        card("5", "", "00", "", ""),
        word("P0", "", "J0", "0"),
        card("5", "", "01", "", ""),
        card("", "K1", "-01", "1", " 2192"),
        word("K2", "31", "77777", "77777"),
        card("5", "", "", "P0", ""),
    ];
    let mut m = Machine::new(1000);
    m.read_deck(&deck.join("\n"));
    m.load().unwrap();
    let punched = |n| {
        let k = m.lookup(b'K', n).unwrap();
        let cards = m.punch_structure(k).unwrap();
        let c = cards.into_iter().find(|c| c.len() > 50).expect("a data card");
        let mut f = c.into_bytes();
        f.resize(61, b' ');
        String::from_utf8(f[47..61].to_vec()).unwrap()
    };
    // Columns 48 to 61: SIGN, PQ, then SYMB (51-55), 56, LINK (57-61).
    assert_eq!(punched(1), "-01    1  2192");
    assert_eq!(punched(2), " 3177777 77777");
}
