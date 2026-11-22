//! Programs of 1961 to 1963, run from other people's transcriptions of
//! their listings. Those files are not in this repository (see
//! `sources/README.md` for where they come from); each test passes without
//! running when they are absent.

use iplv::{Machine, Stop, DEFAULT_CELLS};
use std::fs;
use std::path::{Path, PathBuf};

fn archive() -> Option<PathBuf> {
    let p = Path::new("sources/shrager/ExecutableArchaeology");
    if p.is_dir() {
        Some(p.to_path_buf())
    } else {
        eprintln!("skipped: sources/shrager/ExecutableArchaeology is not here");
        None
    }
}

fn run(decks: &[PathBuf]) -> (Machine, String) {
    let mut m = Machine::new(DEFAULT_CELLS);
    m.trace_all = Some(false);
    for d in decks {
        m.read_deck(&String::from_utf8_lossy(&fs::read(d).unwrap()));
    }
    assert_eq!(iplv::run(&mut m, 10_000_000), Ok(Stop::Halted));
    let out = std::mem::take(&mut m.out);
    (m, out)
}

#[derive(Debug, PartialEq, Clone)]
struct Attempt {
    theorem: String,
    proved: bool,
    effort: u64,
    subproblems: u64,
    substitutions: u64,
}

/// What the Logic Theorist printed about each theorem it was given.
fn attempts(output: &str) -> Vec<Attempt> {
    let lines: Vec<&str> = output.lines().map(str::trim).collect();
    let mut all: Vec<Attempt> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let actual = |l: &str| l.rsplit(' ').next().unwrap().parse::<u64>().unwrap();
        if *l == "TO PROVE" {
            let theorem = lines[i + 1].split_whitespace().next().unwrap().to_string();
            all.push(Attempt {
                theorem,
                proved: false,
                effort: 0,
                subproblems: 0,
                substitutions: 0,
            });
        } else if let Some(a) = all.last_mut() {
            if l.starts_with("PROOF FOUND") {
                a.proved = true;
            } else if l.starts_with("EFFORT") {
                a.effort = actual(l);
            } else if l.starts_with("SUBPROBLEMS") {
                a.subproblems = actual(l);
            } else if l.starts_with("SUBSTITUTIONS") {
                a.substitutions = actual(l);
            }
        }
    }
    all
}

/// The printout with what cannot be the same taken out: page numbers,
/// spacing, the effort figure, and the core address that begins the line
/// for a rejected subproblem.
fn content(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .filter(|w| !w.is_empty())
        .filter(|w| {
            !(w.len() == 1
                && w[0].starts_with('-')
                && w[0].ends_with('-')
                && w[0].len() > 2
                && w[0][1..w[0].len() - 1].chars().all(|c| c.is_ascii_digit()))
        })
        .map(|mut w| {
            if w[0] == "EFFORT" {
                w.pop();
            }
            if w.ends_with(&["REJECTED", "PROBLEM"]) {
                w.remove(0);
            }
            w.join(" ")
        })
        .collect()
}

/// Einar Stefferud's Logic Theorist (RAND RM-3731, 1963), in David Moews's
/// transcription, against the output printed in that memorandum.
#[test]
fn the_logic_theorist_does_what_it_did_in_1963() {
    let Some(root) = archive() else { return };
    let lt = root.join("LogicTheorist");
    let (m, out) = run(&[
        lt.join("logic-theorist-1963-stefferud.iplv"),
        lt.join("logic-theorist-1963-stefferud-input.txt"),
    ]);
    let printed =
        String::from_utf8_lossy(&fs::read(lt.join("logic-theorist-1963-stefferud-output.txt")).unwrap())
            .into_owned();

    let (then, now) = (attempts(&printed), attempts(&out));
    assert_eq!(then.len(), 24);
    assert_eq!(now.len(), 24);
    assert_eq!(
        then.iter().filter(|a| a.proved).count(),
        23,
        "*2.15 ran out of effort in 1963"
    );
    for (a, b) in then.iter().zip(&now) {
        // The same result by the same search: as many subproblems
        // generated and substitutions tried.
        assert_eq!(
            (&a.theorem, a.proved, a.subproblems, a.substitutions),
            (&b.theorem, b.proved, b.subproblems, b.substitutions)
        );
        // Effort is the count of interpretation cycles. It falls short of
        // the IBM 7090's by between 1 and 2 per cent on every theorem.
        let ratio = b.effort as f64 / a.effort as f64;
        assert!(
            (0.98..0.99).contains(&ratio),
            "{}: effort {} then, {} now",
            a.theorem,
            a.effort,
            b.effort
        );
    }

    // Line for line the printout is the same, but for three slips in the
    // transcription of the 1963 pages.
    let (then, now) = (content(&printed), content(&out));
    assert_eq!(then.len(), now.len());
    let different: Vec<_> = then.iter().zip(&now).filter(|(a, b)| a != b).collect();
    assert_eq!(different.len(), 3, "{:#?}", different);
    for (a, b) in different {
        assert!(
            a.replace("DFF.", "DEF.") == *b || *a == b.replace(" , ", " "),
            "{} / {}",
            a,
            b
        );
    }

    // It fits: the run never has fewer than this many cells to spare.
    assert!(m.low_water > 15_000, "{}", m.low_water);
}

/// Herbert Simon's Heuristic Coder (listing of 16 July 1961) and Simon's
/// 1962 IPL-V version of Yngve's sentence generator, in Jeff Shrager's
/// transcriptions, against the output of his Common Lisp IPL-V. For the
/// sentence generator that output equals a run on W. T. Beyer's 1963 IPL-V
/// for the IBM 1620, random number for random number.
#[test]
fn two_more_programs_agree_with_the_lisp_ipl_v() {
    let Some(root) = archive() else { return };
    let squeeze = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");

    let (_, out) = run(&[root.join("HeuristicCoder/heuristic-run.card")]);
    let lisp = fs::read_to_string(root.join("HeuristicCoder/run/t1.txt")).unwrap();
    assert_eq!(squeeze(&out), squeeze(&lisp));

    let (_, out) = run(&[root.join("SimonYngve2/ysimon-clean.card")]);
    let lisp = fs::read_to_string(root.join("SimonYngve2/clean_sentences.txt")).unwrap();
    assert_eq!(squeeze(&out), squeeze(&lisp));
    assert!(out.contains(
        "WHEN HE IS PROUD OF BLACK WHISTLES, STEAM, SMALL AND BIG AND OILED WHEELS, HE IS HEATED."
    ));
}
