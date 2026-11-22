//! Keeps the tutorial honest. In tutorial/*.md, an ```iplv block that is
//! followed by an ```output block (with an ```input block in between if the
//! program reads cards) is a complete deck: it is run, and must print
//! exactly what the output block says. A run that ends in an error has the
//! message as a last line beginning "! ". An ```iplv block followed by
//! anything else is a fragment and is not run, but every card in the
//! tutorial must fit in 80 columns.
//!
//! Run with IPLV_BLESS=1 to rewrite the output blocks from real runs.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

struct Block {
    lang: String,
    text: String,
    /// Byte range of the text within the file.
    start: usize,
    end: usize,
}

fn blocks(src: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut pos = 0;
    let mut open: Option<(String, usize)> = None;
    for line in src.split_inclusive('\n') {
        let fence = line.trim_end().strip_prefix("```");
        match (&open, fence) {
            (None, Some(lang)) => open = Some((lang.to_string(), pos + line.len())),
            (Some((lang, start)), Some("")) => {
                out.push(Block {
                    lang: lang.clone(),
                    text: src[*start..pos].to_string(),
                    start: *start,
                    end: pos,
                });
                open = None;
            }
            _ => {}
        }
        pos += line.len();
    }
    assert!(open.is_none(), "a code block is not closed");
    out
}

/// What the deck prints, the trace with it, as on the printer. A run that
/// stops on an error has the message as a last line beginning "! ". Tests
/// ask for input on separate lines, so the block is put in the reader as
/// the cards behind the start card.
fn transcript(deck: &str, input: &str) -> String {
    let mut m = iplv::Machine::new(iplv::DEFAULT_CELLS);
    m.read_deck(deck);
    if !input.is_empty() {
        m.read_deck(input);
    }
    let result = iplv::run(&mut m, 1_000_000);
    let mut out = format!("{}{}", m.trace_out, m.out);
    for w in &m.warnings {
        out.push_str(&format!("! warning: {}\n", w.lines().next().unwrap_or("")));
    }
    match result {
        Ok(iplv::Stop::Halted) => {}
        Ok(iplv::Stop::Limit) => out.push_str("! stopped after 1000000 cycles\n"),
        Err(e) => out.push_str(&format!("! {}\n", e.lines().next().unwrap_or(""))),
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[test]
fn every_program_in_the_tutorial_prints_what_the_tutorial_says() {
    check_chapters("tutorial", 13, 25);
}

/// The chapters of the manual's Part One, transcribed.
#[test]
fn every_program_from_the_manual_prints_what_its_chapter_says() {
    check_chapters("manual", 3, 8);
}

/// The decks in the notes in docs/.
#[test]
fn every_program_in_the_notes_prints_what_the_note_says() {
    check_chapters("docs", 1, 1);
}

fn check_chapters(folder: &str, chapters: usize, programs: usize) {
    let dir = root().join(folder);
    let bless = std::env::var("IPLV_BLESS").is_ok_and(|v| !v.is_empty());
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    files.sort();
    assert!(
        bless || files.len() >= chapters,
        "{} chapters are missing",
        folder
    );
    let (mut checked, mut failed) = (0, 0);
    for path in files {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let mut text = std::fs::read_to_string(&path).unwrap();
        let bs = blocks(&text);
        let mut edits = Vec::new();
        for (i, b) in bs.iter().enumerate() {
            if b.lang != "iplv" {
                continue;
            }
            let mut j = i + 1;
            let mut input = "";
            if bs.get(j).is_some_and(|n| n.lang == "input") {
                input = &bs[j].text;
                j += 1;
            }
            let Some(want) = bs.get(j).filter(|n| n.lang == "output") else {
                continue;
            };
            assert!(
                bless
                    || iplv::check::check(&b.text).iter().all(|d| !d.error)
                    || want.text.contains("\n! ")
                    || want.text.starts_with("! "),
                "{}: a deck with an error in it\n{}",
                name,
                b.text
            );
            let got = transcript(&b.text, input);
            checked += 1;
            if got != want.text {
                if bless {
                    edits.push((want.start, want.end, got));
                } else {
                    failed += 1;
                    eprintln!(
                        "---- {} ----\n{}--- the chapter says\n{}--- it prints\n{}",
                        name, b.text, want.text, got
                    );
                }
            }
        }
        if !edits.is_empty() {
            for (start, end, new) in edits.into_iter().rev() {
                text.replace_range(start..end, &new);
            }
            std::fs::write(&path, text).unwrap();
        }
    }
    assert!(
        bless || checked >= programs,
        "only {} {} programs were found",
        checked,
        folder
    );
    assert_eq!(failed, 0, "{} programs with the wrong output", folder);
}
