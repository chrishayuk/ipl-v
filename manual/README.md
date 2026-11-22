# The manual's own programs

Part One of the *IPL-V Manual* (second edition, 1964), "The Elements of IPL
Programming", teaches the language through programs. These chapters
transcribe the ones it prints in full, from the page images, and run
them. Each chapter gives the pages, the deck, and what the deck prints;
`cargo test` runs every deck and checks that it still prints that.

| chapter | section of Part One | pages | |
|---|---|---|---|
| [J77, written in IPL](01-j77.md) | 1.0 Lists and Routines | 11 | the basic process J77, coded in IPL |
| [A complete program](02-a-complete-program.md) | 2.0 A Complete Program | 17-23 | F1, with the assembly listing and trace of Figs. 1 and 2 |
| [R62, and the wrong Q](03-r62.md) | 3.0 Simple List Processing | 26 | a routine printed with a deliberate error |
| [R3: searching a list structure](04-list-structures.md) | 4.0 List Structures | 31-39 | three versions of a recursive search, and the trace of Fig. 3 |
| [Description lists](05-description-lists.md) | 6.0 Description Lists | 48-50 | the traces of Fig. 5 |
| [The bridge hand](06-the-bridge-hand.md) | 7.0 Description Lists, Continued; 8.0 | 53-64, 70 | sorting a hand and counting its points and quick tricks |
| [Generators](07-generators.md) | 8.0 Using Generators; 9.0 Generator Construction | 68-81 | a generator with a bug; J100, J101 and J72 coded in IPL |
| [Printing and reading a line](08-printing-and-reading.md) | 10.0 Line Printing; 11.0 Line Reading | 87-93 | R80 to R83, R95 to R97 |

Page numbers are the manual's; the PDF page is 35 more.

## How the transcriptions were made

The code is as printed, card for card, with these exceptions, each noted
where it happens:

- **Names in region J.** Where the manual codes a basic process in IPL
  (J77, J100, J101, J72), the transcription calls it U77, U100, U101 or
  U72, so as not to replace the real one.
- **Comments are shortened** to fit columns 1 to 40 of the card where the
  printed comment is longer. The manual's listings are typeset, not
  punched, and their comments run on.
- **What the manual leaves out.** The manual often shows a routine but not
  the program around it: a driver to execute it and print its answer, or
  data written as "...". What is added is said to be added, and is kept
  apart from the manual's code.

An earlier version of the R95 deck (chapter 8) left out the Type-5
routine-header card that page 92 has before R96; it is in now. The
independent review ([`docs/independent-review.md`](../docs/independent-review.md))
checked twenty passages against the page images and found that and three
slips in this commentary (a level and a count in chapter 4, a page number
in chapter 7), all now put right.

## What the manual gets wrong, and of what kind

Each correction is labelled by kind: a **slip** is wrong under any reading
of the source; a **conflict** is where the sources say two things and we
follow one; an **interpretation** is our reading of what was meant; and
**ours** is a consequence of how this implementation is built.

| where | correction | kind |
|---|---|---|
| R62, page 26 (chapter 3) | `10 W1` should be `11 W1` | slip, made on purpose: the manual prints the error and its answer |
| Fig. 3, step 2 (chapter 4) | the trace's `10L4` is `10 S4` in the listing, and H0 at step 3 agrees with the listing | slip |
| R77, page 72 (chapter 7) | the code's `10 L5` is run as `10 L10`, the list the text names | conflict |
| R77's subprocess (chapter 7) | H5 set + before 9-10 ends | slip, made on purpose: the manual explains it |
| J100 coded the obvious way, page 74 (chapter 7) | the manual's own context housekeeping, page 75 | slip, made on purpose: the manual's counterexample |

## What they checked

Running these programs found two things wrong with this implementation,
both now put right (both of kind **ours**):

- **J101** did not mark the lists it generates as processed, and so did
  not present each head's own word as the cell after the head, as Part
  Two says it does. The manual's J101, coded in IPL in Fig. 11, does; the
  built-in one now gives the same cells in the same order with the same
  signs (chapter 7).
- **A basic process reached by a LINK or a branch** was a step of its own.
  Fig. 3 counts `30HO J30` as one step and Fig. 2 traces no line for the
  J30, and so now does this machine. The count of cycles is unchanged.
