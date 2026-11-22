# Sources

The documents this implementation was written from, and the programs it
was checked against. None of them is in the repository: the scans are
large, and the transcribed decks are other people's work, published
without a licence. Everything in a subdirectory of `sources/` is ignored by
git.

## The documents

| | where |
|---|---|
| *Information Processing Language-V Manual*, ed. A. Newell, 2nd ed., Prentice-Hall, 1964 (scan, 300 pages, and its text) | `IPL-V/manual/` in Executable Archaeology, below |
| The same, 1961 edition: title page, preface and contents only | <https://iiif.library.cmu.edu/file/Newell_box00048_fld04166_doc0001/Newell_box00048_fld04166_doc0001.pdf> |
| A. Newell and F. M. Tonge, "An Introduction to Information Processing Language V", *CACM* 3(4), April 1960 | <https://iiif.library.cmu.edu/file/Newell_box00040_fld03188_doc0001/Newell_box00040_fld03188_doc0001.pdf> |
| E. Stefferud, *The Logic Theory Machine: A Model Heuristic Program*, RAND RM-3731, 1963 (scan) | `LogicTheorist/` in Executable Archaeology |

Part Two of the manual, the Programmers' Reference Manual, is the
specification. Part One has the one run printed from the original system
that `tests/manual.rs` reproduces: Fig. 1 and Fig. 2 of its section 2,
pages 21 and 23 (pages 56 and 58 of the scan).

`manual/` transcribes the programs of Part One from the page images of
the scan, sections 1 to 11 (PDF pages 46 to 128); each chapter gives its
pages.

The text of the scan confuses `l` with `1`, `O` with `0` and `S` with `5`
throughout (`HS` is H5, `JlO` is J10). The figures were read from the page
images.

## The programs

```bash
mkdir -p sources/shrager
git clone --depth 1 https://github.com/jeffshrager/ExecutableArchaeology sources/shrager/ExecutableArchaeology
```

That is Jeff Shrager's [Executable
Archaeology](https://github.com/jeffshrager/ExecutableArchaeology), which
holds his Common Lisp IPL-V and:

| | |
|---|---|
| `LogicTheorist/logic-theorist-1963-stefferud.iplv` | Stefferud's Logic Theorist, transcribed by David Moews (<https://github.com/dmoews/ipl-v-logic-theorist>) |
| `LogicTheorist/logic-theorist-1963-stefferud-input.txt` | the 24 theorems given to it in 1963 |
| `LogicTheorist/logic-theorist-1963-stefferud-output.txt` | the output printed in RM-3731, transcribed by Moews |
| `HeuristicCoder/heuristic-run.card` | Simon's Heuristic Coder, from a listing of 16 July 1961 |
| `SimonYngve2/ysimon-clean.card` | the IPL-V version of Yngve's sentence generator, run on 25 June 1962, with a new printing routine |

With that directory in place, `cargo test` runs `tests/historical.rs`
against them. To run the Logic Theorist by hand:

```bash
L=sources/shrager/ExecutableArchaeology/LogicTheorist
cargo run --release -- run $L/logic-theorist-1963-stefferud.iplv $L/logic-theorist-1963-stefferud-input.txt
```

The figures in the main README were taken with that repository at commit
`2ff0c51` (30 October 2026).
