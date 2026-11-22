# Introduction

IPL-V is Information Processing Language V. It was made between 1957 and
1964 at the RAND Corporation and the Carnegie Institute of Technology by
Allen Newell, J. C. Shaw, Herbert Simon, Fred Tonge and others, for one
purpose: writing programs that solve problems the way people seem to, by
search and rules of thumb. The Logic Theorist, the General Problem Solver
and EPAM were written in the IPL languages. They needed something no
language of the time had: data that grows and changes shape while the
program runs. IPL gave them the list.

This tutorial teaches the language from its manual, the *Information
Processing Language-V Manual* (second edition, 1964), and every program in
it is run by the tests of this repository, so what a program is shown
printing is what it prints.

## A computer, not a notation

IPL-V does not look like a programming language, because it is not one in
the modern sense. There are no expressions, no statements and no names for
variables. The manual describes a machine, the **IPL computer**, and you
write its storage out by hand:

- Storage is **cells**. Every cell holds one **word**, and a word has four
  parts: `P`, `Q`, `SYMB` and `LINK`.
- A **symbol** is the name of a cell. `SYMB` and `LINK` are symbols.
- `LINK` names the next cell, so cells chain into **lists**.
- A word can be data or an **instruction**. A routine is a list of
  instructions.
- Cells not in use are on one long list, the **available space**. A program
  takes cells from it when it builds something and gives them back when it
  erases it.

A program is punched on cards, one word to a card. The cards are a *deck*.

## Running a deck

```
cargo build --release
target/release/iplv run examples/hello.iplv
```

Here is that deck. Do not try to read it yet; the next chapter takes it
apart.

```iplv
HELLO, WORLD                            9
                                        2 P                10
                                        2 T                10
                                        5       00
CLEAR THE PRINT LINE.                     P0      J154
                                                10T0
ENTER HELLO IN IT.                                J157
                                                10T1
                                                  J157
                                                10T2
                                                  J157
PRINT THE LINE.                                   J155  0
                                        5       01
                                          T0    21HELLO
                                          T1    21 WORL
                                          T2    21D
                                        5         P0
```

```output
HELLO WORLD
```

## Seeing it work

Three things show you the machine at work, and you will want all of them.

`--trace` prints every instruction as it is interpreted:

```
target/release/iplv run examples/hello.iplv --trace
```

`--list` prints the assembly listing: where each card went in storage.

The page in `web/` shows the machine itself: the instruction being
interpreted, the cells with special functions, a picture of storage, and
any list drawn as boxes and arrows. Chapter 12 says how to start it.

## What you need to know first

Nothing about lists. It helps to have met an assembly language, because
IPL-V is nearer to one than to anything else, but it is not required.
