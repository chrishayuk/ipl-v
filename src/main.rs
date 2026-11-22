use iplv::{Machine, Stop, DEFAULT_CELLS, DEFAULT_CYCLES};
use std::io::Write;
use std::process::ExitCode;

const USAGE: &str = "Usage: iplv run [OPTIONS] <DECK> [<CARDS>...]
       iplv check <DECK>       say what is wrong with a deck, without running it
       iplv format <DECK>      print the deck as a keypunch would have left it
       iplv lsp                serve the Language Server Protocol on stdin and stdout

run: the deck is loaded and run from its start card. Further files are put
in the card reader behind it, for the program to read. A DECK of - is
standard input.

Options:
  --trace          print every instruction as it is interpreted
  --no-trace       print none, whatever trace marks the program has
  --list           print the assembly listing
  --cells <N>      total available space in cells (default 22000)
  --cycles <N>     stop after N interpretation cycles (default 100000000)
  --stats          report cycles, storage and the processes used
  -h, --help       show this help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut files, mut trace, mut list, mut stats) = (Vec::new(), None, false, false);
    let (mut cells, mut cycles) = (DEFAULT_CELLS, DEFAULT_CYCLES);
    let mut it = args.iter();
    let mut command = None;
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{}", USAGE);
                return ExitCode::SUCCESS;
            }
            "--trace" => trace = Some(true),
            "--no-trace" => trace = Some(false),
            "--list" => list = true,
            "--stats" => stats = true,
            "--cells" | "--cycles" => {
                let Some(v) = it.next().and_then(|v| v.parse::<u64>().ok()) else {
                    eprintln!("iplv: {} needs a number", a);
                    return ExitCode::from(2);
                };
                if a == "--cells" {
                    cells = v as usize;
                } else {
                    cycles = v;
                }
            }
            "-" => files.push(a.clone()),
            _ if command.is_none() => command = Some(a.clone()),
            _ => files.push(a.clone()),
        }
    }
    match (command.as_deref(), files.first()) {
        (Some("lsp"), _) => {
            let stdin = std::io::stdin();
            return match iplv::lsp::serve(&mut stdin.lock(), &mut std::io::stdout()) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("iplv: {}", e);
                    ExitCode::FAILURE
                }
            };
        }
        (Some(c @ ("check" | "format")), Some(file)) => {
            let text = match slurp(file) {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("iplv: {}: {}", file, e);
                    return ExitCode::from(2);
                }
            };
            if c == "format" {
                print!("{}", iplv::check::format(&text));
                return ExitCode::SUCCESS;
            }
            let problems = iplv::check::check(&text);
            for d in &problems {
                println!(
                    "{}:{}:{}: {}: {}",
                    file,
                    d.line,
                    d.start + 1,
                    if d.error { "error" } else { "warning" },
                    d.message
                );
            }
            return if problems.iter().any(|d| d.error) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }
        _ => {}
    }
    if command.as_deref() != Some("run") || files.is_empty() {
        eprint!("{}", USAGE);
        return ExitCode::from(2);
    }
    let mut m = Machine::new(cells);
    m.trace_all = trace;
    if list {
        m.listing = Some(String::new());
    }
    for f in &files {
        match slurp(f) {
            Ok(text) => m.read_deck(&text),
            Err(e) => {
                eprintln!("iplv: {}: {}", f, e);
                return ExitCode::from(2);
            }
        }
    }
    let start = match m.load() {
        Ok(Some(s)) => s,
        Ok(None) => {
            flush(&mut m);
            eprintln!("iplv: the deck has no start card (TYPE 5 with a routine in SYMB)");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            flush(&mut m);
            eprintln!("iplv: {}", e);
            return ExitCode::FAILURE;
        }
    };
    m.start(start);
    let mut code = ExitCode::SUCCESS;
    loop {
        // Run in slices so that output appears as it is printed.
        let slice = (m.cycles + 100_000).min(cycles);
        let r = m.run(slice);
        flush(&mut m);
        match r {
            Ok(Stop::Halted) => break,
            Ok(Stop::Limit) if m.cycles >= cycles => {
                eprintln!("iplv: stopped after {} interpretation cycles", m.cycles);
                code = ExitCode::FAILURE;
                break;
            }
            Ok(Stop::Limit) => {}
            Err(e) => {
                eprintln!("iplv: {}", e);
                code = ExitCode::FAILURE;
                break;
            }
        }
    }
    if stats {
        eprintln!("cycles      {}", m.cycles);
        eprintln!(
            "storage     {} cells, {} available now, {} at the least",
            m.space(),
            m.free_cells,
            m.low_water
        );
        let mut used: Vec<(usize, u64)> = m
            .prim_counts
            .iter()
            .copied()
            .enumerate()
            .filter(|p| p.1 > 0)
            .collect();
        used.sort_by_key(|p| std::cmp::Reverse(p.1));
        let text: Vec<String> = used.iter().map(|(n, c)| format!("J{} {}", n, c)).collect();
        eprintln!("processes   {}", text.join(", "));
    }
    code
}

/// The text of a file, or of standard input for `-`.
fn slurp(file: &str) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    if file == "-" {
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut bytes)?;
    } else {
        bytes = std::fs::read(file)?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn flush(m: &mut Machine) {
    let (out, err) = (std::io::stdout(), std::io::stderr());
    let (mut out, mut err) = (out.lock(), err.lock());
    if let Some(l) = m.listing.as_mut() {
        let _ = out.write_all(l.as_bytes());
        l.clear();
    }
    for w in m.warnings.drain(..) {
        let _ = writeln!(err, "iplv: warning: {}", w);
    }
    // The trace and the program's printing are one stream, in order, as on
    // the printer.
    let _ = out.write_all(m.trace_out.as_bytes());
    m.trace_out.clear();
    let _ = out.write_all(m.out.as_bytes());
    m.out.clear();
    let _ = out.flush();
}
