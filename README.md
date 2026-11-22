# iplv

An implementation of **IPL-V**, Information Processing Language V: the
list-processing language in which the first artificial intelligence
programs were written. It was designed at RAND and Carnegie Tech by Allen
Newell, J. C. Shaw, Herbert Simon, Fred Tonge and others, ran on the IBM
650, 704 and 709 by 1960, and is described in the *Information Processing
Language-V Manual* (Prentice-Hall, 1961; second edition 1964).

IPL-V is less a language than a computer. Its programs are lists of
instruction words, `P Q SYMB LINK`, punched one to a card, and its manual
defines the machine that interprets them: cells, an available space list,
push-down storage cells, an interpretation cycle and some 190 basic
processes. This is that machine, written from the manual, in Rust with no
dependencies. Storage is a real array of cells and a real free list, so a
program here uses space, gives it back and runs out of it as it did then.

```
THE LIST K1.                              K1      0
                                                  B1
                                                  C1
                                                  B1
                                                  A1
                                                  A1
                                                  B1    0
```

```
$ iplv run examples/manual-f1.iplv --no-trace
145      0
         B1
         C1
         A1
```

That is the manual's own first complete program, which removes the
duplicates from a list.

## What has been checked

**Against the manual's own run.** Part One of the manual prints the
assembly listing and the full trace of that program, made on the IPL-V
system (Figs. 1 and 2). `cargo test` holds this implementation to both:

- every word is loaded at the address the listing gives (regions from cell
  64, `E1`'s second instruction in cell 124, the list `K1` in 136 to 141),
  with `H0` at 24574 and `W0` at 24587, and the listing ends with the same
  line, `NUMBER OF CELLS ON AVAILABLE SPACE=21858`;
- all 46 lines of the trace agree in level, instruction address, the sign
  of H5, P, Q, SYMB, LINK and the count of interpretation cycles in H3.

The H3 column is what settled how cycles are counted: one for each
instruction interpreted, and one more for each primitive it executes.

**Against a program of 1963.** Einar Stefferud's IPL-V version of the Logic
Theorist of Newell, Shaw and Simon (RAND RM-3731, 1963) runs unmodified, in
David Moews's transcription of the listing, and its output is compared with
the output printed in that memorandum:

- all 24 theorems of *Principia Mathematica* get the 1963 result (23
  proved; \*2.15 gives up at its effort limit), each by the same search:
  the same number of subproblems and of substitutions for every theorem,
  \*2.15 included;
- the printout is the same line for line, 533 lines of it, apart from
  three slips in the transcription of the 1963 pages (`DFF.` for `DEF.`
  and two missing commas), the effort figures, and the core addresses it
  prints for rejected subproblems;
- "effort", which is the cycle count, comes to between 98.3 and 98.7 per
  cent of the 1963 figure on every theorem.

| theorem | 1963 effort | here | | theorem | 1963 effort | here |
|---|---|---|---|---|---|---|
| \*2.01 | 8062 | 7941 | | \*2.20 | 38111 | 37494 |
| \*2.02 | 6041 | 5946 | | \*2.21 | 8021 | 7888 |
| \*2.03 | 11904 | 11729 | | \*2.24 | 21521 | 21181 |
| \*2.04 | 32262 | 31776 | | \*3.13 | 13293 | 13089 |
| \*2.05 | 13440 | 13225 | | \*3.14 | 12753 | 12563 |
| \*2.06 | 46665 | 45954 | | \*3.24 | 14541 | 14323 |
| \*2.07 | 2623 | 2579 | | \*4.13 | 15965 | 15728 |
| \*2.08 | 8998 | 8848 | | \*4.20 | 12404 | 12204 |
| \*2.10 | 5150 | 5073 | | \*4.24 | 92331 | 90771 |
| \*2.11 | 6190 | 6101 | | \*4.25 | 20788 | 20438 |
| \*2.12 | 8215 | 8105 | | | | |
| \*2.13 | 18263 | 18022 | | | | |
| \*2.14 | 181261 | 178532 | | | | |
| \*2.15 | 206728 | 203429 | | | | |

The whole run takes 865,503 cycles and never has fewer than 15,322 cells
of available space left, out of 22,000. The remaining one and a half per
cent of effort is not explained. Measured over exactly the cycles the
program counts (from `50 H3` in M3 to `10 H3` in M77), it is not:

- a cycle more for each routine executed, each return, each branch
  taken, or each basic process reached through a LINK: none of these is
  in proportion to it;
- a fixed number of extra cycles for any one or two basic processes: the
  best such fit is still 40 cycles out on some theorems;
- J100 written in IPL, as the manual codes it on page 75: with that in
  place of the built-in J100, effort is 105 to 110 per cent of 1963's.

It may be several basic processes that were IPL routines in the 1963
system, each costing a little; these counts cannot say which.

**Against another implementation.** Herbert Simon's Heuristic Coder (a
listing of 16 July 1961) and the 1962 IPL-V version of Yngve's sentence
generator from Simon's papers, in Jeff Shrager's transcriptions, print what Shrager's Common
Lisp IPL-V prints for them. For the sentence generator that is also what
W. T. Beyer's 1963 IPL-V for the IBM 1620 prints, random number for random
number.

None of those decks is in this repository: see
[`sources/README.md`](sources/README.md). The tests that use them pass
without running when the files are absent.

This is not the first modern IPL-V, nor the first to run the Logic
Theorist. Jeff Shrager's
[Executable Archaeology](https://github.com/jeffshrager/ExecutableArchaeology)
and David Moews's
[ipl-v-logic-theorist](https://github.com/dmoews/ipl-v-logic-theorist) did
both, and the transcriptions are theirs. What is different here is the
storage model and the cycle count. Shrager reports effort about a third
below the 1963 figures on his interpreter, with \*2.15 searching further
than it did in 1963 before the limit stops it (48 subproblems for 31), and
says that Moews's differs in the same way.

## The manual's own programs

[`manual/`](manual/README.md) transcribes the programs Part One of the
manual prints in full, from the page images, and runs them: J77 and J100,
J101 and J72 coded in IPL by the manual itself, a recursive search with
the step-by-step trace of Fig. 3, the description-list traces of Fig. 5, a
bridge hand sorted and scored, a generator printed with a bug in it, and
routines that print and read a line. `cargo test` runs them all, and holds
the R3 run to all 27 steps that Fig. 3 lists.

## Tutorial

[`tutorial/`](tutorial/01-introduction.md) teaches the language in
thirteen chapters, from the card and its columns through push-down cells,
list processes, description lists, list structures and generators, to
tracing and a reference. Every deck in it is run by `cargo test`, and
what it is shown printing is what it prints.

## Quick start

```bash
cargo build --release
target/release/iplv run examples/hello.iplv
target/release/iplv run examples/hanoi.iplv
target/release/iplv run examples/sum.iplv examples/sum.in
target/release/iplv run examples/manual-f1.iplv --list
```

```text
Usage: iplv run [OPTIONS] <DECK> [<CARDS>...]

The deck is loaded and run from its start card. Further files are put in
the card reader behind it, for the program to read.

Options:
  --trace          print every instruction as it is interpreted
  --no-trace       print none, whatever trace marks the program has
  --list           print the assembly listing
  --cells <N>      total available space in cells (default 22000)
  --cycles <N>     stop after N interpretation cycles (default 100000000)
  --stats          report cycles, storage and the processes used

       iplv check <DECK>       say what is wrong with a deck, without running it
       iplv format <DECK>      print the deck as a keypunch would have left it
       iplv lsp                serve the Language Server Protocol on stdin and stdout
```

## The language in one page

A card is 80 columns. Columns 1 to 40 are comment.

| columns | field | |
|---|---|---|
| 41 | TYPE | blank for a word; 1 comment, 2 region, 3 block, 5 header, 9 first card |
| 43-47 | NAME | the symbol that names this cell, if it needs a name |
| 48 | SIGN | `+` or `-`: this word in a routine is data |
| 49 | P | operation |
| 50 | Q | designation |
| 51-55 | SYMB | |
| 57-61 | LINK | the next cell; blank means the next card |

A symbol is a letter or punctuation mark and a number (`A12`, `*3`), a
regional symbol; or `9-` and a number, a local symbol, which means
something only inside the list structure it occurs in.

Q designates a symbol S from SYMB: 0, SYMB itself; 1, the symbol in the
cell SYMB names; 2, the symbol in the cell that that symbol names. Q = 3
and 4 mean 0 and mark the routine to be traced. P says what to do with S:

| P | | |
|---|---|---|
| 0 | execute S | a routine or a basic process |
| 1 | input S | push H0 down and put S in it |
| 2 | output to S | put (0) in cell S and pop H0 up |
| 3 | restore S | pop cell S up |
| 4 | preserve S | push cell S down |
| 5 | replace (0) by S | |
| 6 | copy (0) in S | |
| 7 | branch to S if H5 is - | |

`H0` is the communication cell: a routine takes its inputs from it and
leaves its outputs there, and (0), (1), ... are the symbols in it and under
it. `H5` is the test cell. `W0` to `W9` are working cells. The basic
processes are `J0` to `J202`; the manual ends with a two-page list of
them, the system cells and the card types.

So, from `examples/factorial.iplv`:

```
                                          P0      J154
                                          9-1   10N0
                                                10F0
                                                10F0
F0 TIMES N0 INTO F0.                              J112
REMOVE THE NAME F0.                             30H0
                                                10K1
                                                10N0
                                                10N0
N0 MINUS 1 INTO N0.                               J111
IS N0 ZERO.                                       J117
IF NOT, MULTIPLY AGAIN.                         709-1
                                                10F0
ENTER THE RESULT                                  J157
AND PRINT IT.                                     J155  0
```

## Examples

`examples/` holds eleven decks, each with the output it must print in a
`.out` file:

| deck | |
|---|---|
| `hello` | a print line, three data terms |
| `pushdown` | preserve and restore, step by step, as in section 1.15 |
| `factorial` | arithmetic on data terms; a loop |
| `hanoi` | a routine that calls itself, with its working cells saved |
| `lists` | building, reversing and erasing a list; the generator J100 |
| `describe` | description lists: attributes and values, asked and changed |
| `generator` | a generator written in IPL with J17, J18 and J19 |
| `structure` | a list structure printed, copied and erased; counting H2 |
| `sentences` | random sentences from a grammar held as a list structure |
| `sum` | reading cards from the reader |
| `manual-f1` | the manual's first complete program, with its trace |

## In the browser

`web/` is the same machine compiled to WebAssembly, with a page that shows
it working: the deck as a listing with the instruction being interpreted,
H0, H5, the working cells and the routines in progress, a picture of
storage in which every cell is a square, and any list structure drawn as
cells and links.

```bash
node web/build.mjs                  # builds web/iplv.wasm and web/examples.json
node web/test.mjs                   # 77 checks of the WebAssembly build
python3 -m http.server -d web 8000  # then open http://localhost:8000
```

- **Step**, **Over** (to this routine's next instruction), **Out** (until
  it has finished) and **Run** at eleven speeds, from one instruction a
  second to as fast as it goes. A click beside a card sets a stop there.
- **Storage** is coloured by what each cell is for: available space, a
  region, a word loaded from a card, a cell made while running, a data
  term, the push-down lists of H0, H1 and the working cells. Cells that
  changed since the last picture are yellow. It is read from the module's
  memory, a cell at a time, and the available space is found by following
  H2.
- **A list structure** follows the symbol in H0 unless a symbol is named
  (`K1`, `W0`, `1350`). A click on a cell, or on a symbol in H0 or a
  working cell, shows that one.
- **Open a deck** takes a deck from a file, and with it files of cards for
  the program to read. The Logic Theorist loads and runs this way, in
  about a tenth of a second at full speed.
- `?deck=hanoi&steps=120&show=W0` in the address opens a deck part of the
  way through its run.

`src/debug.rs` is what the page talks to: a run with breakpoints that
describes itself as JSON. `src/web.rs` exports it, and `web/iplv.js` wraps
those exports for the page and for the tests.

## In an editor

`iplv lsp` is a language server for decks, and `editors/vscode` is a
Visual Studio Code extension that carries it: rulers at the card's fields,
Tab from field to field, what a card says when the pointer rests on it,
problems as you type, definitions, references, rename, an outline, and
commands to run the deck. See [`editors/README.md`](editors/README.md).

```bash
target/release/iplv check examples/hanoi.iplv    # says nothing: nothing is wrong
target/release/iplv format deck.iplv             # as a keypunch would have left it
```

## The machine

`src/machine.rs` is the storage: the map of cells, regions, the available
space list H2, preserve and restore, lists and description lists.
`src/loader.rs` is the one-pass assembly of section 18. `src/interp.rs` is
the interpretation cycle of section 3.15. `src/prims.rs` and `src/print.rs`
are the basic processes.

The state of a run is in the cells and nowhere else, with one exception:
the processes that execute a routine and have work to do when it returns
(J18, J100 to J102, and the monitor and trap actions) keep a note of that
work outside storage. The hierarchy of routines in progress is the
push-down list of H1, made of cells from H2, as the manual has it.

## What is different from the manual, and what is missing

Where the manual leaves something to the "machine system write-ups", or a
deck needs something the manual does not say, the choice is marked in the
source. The ones that matter:

- **Symbols may be indented in their fields.** The manual wants the first
  character in the leftmost column. The surviving transcriptions follow
  listings, which indent. And a `9` by itself is the internal symbol 9,
  which is how the Logic Theorist's table of characters uses it.
- **A regional cell is never put on available space.** Erasing it sets it
  empty. In 1964 it went on H2 with everything else.
- **A regional symbol beyond its region gets a cell of its own**, with a
  warning. The original loader rejected it.
- **J129's multiplier** is the one in Beyer's IBM 1620 system. The 704 and
  7090 systems' is not known to me.
- **The first instruction is cycle 5**, as in the manual's trace. What the
  system did in the first four is not known.
- **Cells made while a program runs** come three addresses lower than in
  the manual's trace (145 for its 148). Those made by loading agree.
- **J180 sets the entry column 1W25 to 1.** The manual does not say so;
  the Logic Theorist needs it.

Not implemented: auxiliary storage (J106, J107 and J109; Q = 6 and 7;
header types 6 and 7 are loaded to main storage; J105 and J108 report
that nothing is on auxiliary storage, which is true); reading and writing list
structures and tapes (J140 to J146); blocks (J172 to J176) and block cards
other than print lines; J162, J167, J189; the compressed and binary card
forms; origins on region cards; the traps on H2 and H3 (J170 itself
works). J171 and J201 do nothing, and J166 saves nothing.

## Tests

```bash
cargo test
```

| file | |
|---|---|
| `tests/manual.rs` | Figs. 1, 2 and 3 of the manual's Part One |
| `tests/processes.rs` | the basic processes against their definitions in Part Two |
| `tests/examples.rs` | every deck in `examples/` against its `.out` |
| `tests/historical.rs` | the Logic Theorist, the Heuristic Coder and the sentence generator, if their decks are present |
| `tests/tutorial.rs` | every deck in the tutorial and in `manual/` against the output printed under it |
| `tests/check.rs`, `tests/lsp.rs` | the deck checker, and a conversation with the language server |
| `web/browser-test.mjs` | the page, driven in headless Chrome |
| `web/test.mjs` | the WebAssembly build: every example stepped and run, stops, the JSON the page draws from |

## Sources

- *Information Processing Language-V Manual*, edited by Allen Newell,
  second edition, Prentice-Hall, 1964. Part Two, the Programmers' Reference
  Manual, is the specification followed here.
- A. Newell and F. M. Tonge, "An Introduction to Information Processing
  Language V", *Communications of the ACM* 3(4), April 1960.
- E. Stefferud, *The Logic Theory Machine: A Model Heuristic Program*,
  RAND RM-3731, 1963.

[`sources/README.md`](sources/README.md) says where to find them.

## Licence

Apache License 2.0 - see [LICENSE](LICENSE). That covers the code, the
examples, the tutorial and the commentary in `manual/`. The programs that
`manual/` transcribes are quoted from the *Information Processing
Language-V Manual* (Prentice-Hall, 1961 and 1964) for study and
reference, and remain the copyright of its authors and publisher. The
historical decks in `sources/` are not part of this repository; see
[`sources/README.md`](sources/README.md).
