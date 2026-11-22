# Independent review, November 2026

## Who, and what was checked

On 22 November 2026 the repository was reviewed by OpenAI Codex CLI
(codex-cli 0.166.0), run as an independent second model. It is a
program, not a person. It was asked to check the implementation and the
claims made for it against the original documents: the *IPL-V Manual*,
second edition, 1964 (here **M**), and Stefferud's RAND RM-3731, 1963
(**S**).

It compared **20 passages** of the transcriptions in `manual/` and the
Logic Theorist test against the page images, field by field, allowing for
the declared renaming of J processes to U and V, shortened comments and
added drivers. It ran the probes behind each finding below, and the
builds and tests the README describes. It found no unexplained
difference in an executable field in the passages it sampled. Its
browser and editor checks could not run where it ran (no Chrome, no
network for npm); they have been run since, and pass.

Every finding was then checked again here against the code, by running
it, and against the page images. Of the thirteen findings (a fourteenth
reports the builds), twelve are accepted and one, finding 6, is accepted
in part and disputed in part; none is rejected outright. Six defects in the code were
put right, each with a test in `tests/review.rs` that fails before the
fix and passes after it.

The kinds used below are those of the README: **slip** (wrong under any
reading of the source), **conflict** (the sources say two things),
**interpretation** (our reading of what was meant), **ours** (a
consequence of how this implementation is built).

## The findings, and what was done

**1. Print lines cost no storage.** Accepted. M p.222, section 18.3: the
words of a print line "are taken from available storage, not from the
region". The loader reserved no cells, and a 1000-word line loaded into
936 cells of storage. Fixed in `79a0817`: the words are reserved as a
block. The characters are still kept outside the cells, and the README
now lists that as a third piece of run state outside storage. Test:
`a_print_line_takes_its_words_from_available_storage`.

**2. J180 reset 1W25 to 1.** Accepted. M p.236 defines J180 without
touching 1W25; R96 sets it to 0 so that J184, which scans from 1W25 + 1,
starts at column 1. With the reset, R95 read `HELLO` punched from column
1 as `ELLO`. The README said the Logic Theorist needs the reset; taking
it out shows that it does not (M89 calls J154 before every J180): all 24
theorems, 533 lines, 865,503 cycles and the 15,312-cell low-water mark
are unchanged. Fixed in `1683de6`; `examples/sum.iplv` and the tutorial's
copy relied on the reset and now call J154. Test:
`j180_leaves_the_entry_column_where_it_was`.

**3. No minus zero.** Accepted. M p.196: "Zero is signed; J123 takes zero
into minus zero." J114 found the negated zero equal to plus zero. Fixed in
`05f0c19` with a distinct bit pattern for integer minus zero; arithmetic
that comes to zero still gives plus zero, which the manual does not
settle, and the README says so. Tests:
`j123_takes_zero_into_minus_zero`, `minus_zero_can_be_punched`.

**4. The three "transcription slips" in the Logic Theorist output are in
the scan.** Accepted. S p.95 has `DFF.`; S pp.101 and 102 have no comma
before `SUBLEVEL REPLACEMENT` in the first subproblems of \*3.13 and
\*3.14, though every other such line has one. The transcription is right;
the README and `tests/historical.rs` blamed it. Documentation changed in
`c91086c`. `DFF.` is labelled a slip of the 1963 printing; the commas are
left unexplained. The test still allows these three lines to differ: this
run prints `DEF.` and the commas.

**5. A negative exponent lost its blank.** Accepted. M p.215: `140000 -16`
(10 columns). This implementation printed `140000-16`, and `manual/08`
called the page unclear; it is not. Fixed in `282fcf8`; the commentary is
corrected, and `examples/tape-list-structure.out` changes with it. Test:
`a_floating_point_number_keeps_the_blank_before_its_exponent`.

**6. List printing follows Part One rather than Part Two.** Accepted in
part. The format is a **conflict**: the format diagram of M p.214 has a PQ
column for every word, while the run printed in Fig. 2 (M p.23), made on
the real system, prints `148  0` and then `B1`, `C1`, `A1` with no PQ.
The run is kept, since it is the only evidence of what the system did,
and `tests/manual.rs` holds the implementation to it. What was wrong was
J151's local symbols, printed as `9-68` though M p.214 says their
addresses are printed with "the Q = 2" identifying them; Fig. 2 has no
local symbols, so nothing conflicts with following the text there. Fixed
in `b4b7adb` (now `02 68`; J152 likewise). Test:
`j151_prints_a_local_symbol_as_its_address_with_q_2`. The conflict is now
said in `src/print.rs` and the README.

**7. Auxiliary routines kept in main storage are not invisible.**
Accepted. A program that obeys the manual's rules for auxiliary routines
can still count H2 with J126 (M p.196 allows it) or run out of space, and
see the difference. Not changed in the code (it would need the shared
buffer of M pp.192 to 194); the claim of invisibility is withdrawn in the
README and in `src/auxiliary.rs`, `c91086c`.

**8. R95's five-column pieces.** Accepted. M p.91 asks for "a list of BCD
data terms, exactly like list L15", and L15 divides `SENTENCE.` between
terms. `manual/08` said the pieces were not what the manual intends;
that was an **interpretation**, and it is withdrawn in `6a3a79e`.

**9. Discrepancies in the transcriptions and the commentary.** Accepted,
all five, checked against the page images; fixed in `6a3a79e`:

- `manual/04`: step 48 of Fig. 3 is at level 4, not 3;
- `manual/04`: Fig. 3 lists 27 steps, not twenty-five (the test already
  had 27);
- `manual/07`: L5 is on page 29, not 23;
- `manual/08`: the Type-5 routine-header card before R96 on page 92 was
  left out of the deck; it is in now, and makes no difference to the run;
- `manual/02`: Fig. 2 also ends with `PROGRAM RAN TO COMPLETION.`, which
  is not printed here, so the trace differs in three ways, not "two ways
  only".

**10. IPL-V and the first AI programs.** Accepted. S p.1 says the Logic
Theorist was converted from IPL-II to IPL-V. The README's first sentence
now speaks of the IPL family, `c91086c`.

**11. Claims without retained evidence.** Accepted as evidence gaps, not
as errors. In `c91086c` the README now says that the cycle-fitting
experiments are reported, not reproduced (their tables were not kept),
and that the IBM 7094 figures are read from Moews's repository and
checked by nothing here. The loader's comment no longer attributes the
print-line-name-in-SYMB convention to section 25.3 of the first edition,
which is not available; it rests on Simon's Heuristic Coder of 1961,
which has it. The LARQL note now says no copy of the quoted files is
kept. The review found no claim to be the first modern IPL-V; the README
already credits Shrager and Moews. "Runs unmodified" for the Logic
Theorist is narrowed to "with no change to the deck", since the loader
accepts the transcription's conventions.

**12. The audit of corrections.** Accepted, with the kinds the review
gave, and recorded in two tables: the manual's own errors in
`manual/README.md` and all corrections and departures in the README's new
section "Corrections, and of what kind". In one place this goes further
than the review: the review called the J180 reset and the other loader
conveniences reconstruction decisions; the J180 reset turned out also to
be wrong (finding 2) and has been removed. The review's point that J173
and J176 reading 1W19 follows M pp.218 and 219, and is not an erratum, is
now said in the README.

**13. The spot check.** Its 20 passages are listed in the review. Seven
of them were looked at again here from the page images: Fig. 2 (M p.23),
Fig. 3 (M p.34), R95 to R97 (M pp.92 and 93), and the three Logic
Theorist lines (S pp.95, 101 and 102). So were the pages behind findings
1 to 8 and 10 (M pp.29, 91, 195, 196, 214, 215, 222, 236 and 237, and the
text of p.72). All agree with what the review says of them. The other
thirteen passages were not re-examined.

**14. Builds and checks.** Confirmed. After the fixes, here:

- `cargo build --release`, `cargo fmt --all --check` and
  `cargo clippy --all-targets -- -D warnings` pass;
- `cargo test --release`: 60 tests pass, with the historical decks
  present, so `tests/historical.rs` runs rather than skips;
- `node web/build.mjs && node web/test.mjs`: 201 checks, 0 failed;
- `node web/browser-test.mjs` in headless Chrome: 26 checks, 0 failed,
  after two changes to the test (`fe7feb3`): one step more before the
  error, since `sum.iplv` now begins with J154, and the light scheme set
  before the dark one, since the check failed on a machine already in
  dark mode, before any of these changes;
- in `editors/vscode`, `npm ci`, `npm run build`, `npm test` and
  `npm run test:vscode` pass.

## What this review does not establish

It sampled passages; it did not compare every card of every
transcription. It did not settle why the 1963 Logic Theorist's effort
figures are 1.3 to 1.7 per cent above this implementation's, nor why the
1963 printout lacks two commas. What the 704 and 7090 systems did with
the sign of a zero result of arithmetic is still not known here.
