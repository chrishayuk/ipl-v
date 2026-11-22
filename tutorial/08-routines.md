# Routines

A routine is a list of instructions with a name. Executing its name, with
P = 0, makes the interpreter go down a level, interpret the list, and
come back when the list ends. Routines are how an IPL-V program is built:
the manual's advice, and the practice of every surviving program, is many
small routines with definitions written like those of the J's.

## Inputs and outputs

Write a routine's definition before the routine, in the manual's style:
what is in (0), (1), ... when it starts, and what it leaves. Then keep to
the rule: **remove your inputs, leave only your outputs**.

Here is one. *R1: (0) names a list; the output (0) is its last symbol.*

```iplv
THE LAST SYMBOL OF A LIST               9
                                        2 L                10
                                        2 P                10
                                        2 R                10
                                        2 S                10
                                        5       00
                                          P0    10L1
                                                  R1
                                                  J152  0
R1: THE LAST SYMBOL OF LIST (0).        1
LOCATE THE LAST CELL OF (0).              R1      J61
REPLACE (0) BY THE SYMBOL IN IT.                52H0    0
                                        5       01
                                          L1      0
                                                  S1
                                                  S2
                                                  S3    0
                                        5         P0
```

```output
         S3
```

`52 H0` is another idiom with Q = 2: replace (0), which names a cell, by
the symbol in that cell. `R1` now behaves just like a basic process, and
whoever uses it need not know it is not one.

## Working cells

A routine that needs to keep things uses the W cells, preserved on the
way in and restored on the way out (chapter 5). *R2: print the symbols
of list (0) on one line.*

```iplv
A LIST ON ONE LINE                      9
                                        2 K                10
                                        2 L                10
                                        2 P                10
                                        2 R                10
                                        2 S                10
                                        5       00
                                          P0    10L1
                                                  R2    0
R2: PRINT THE SYMBOLS OF LIST (0) ON ONE1
LINE.                                   1
W0 HOLDS A CELL OF THE LIST.              R2      J50
                                                  J154
                                          9-1   11W0
LOCATE THE NEXT CELL.                             J60
IT, OR THE LAST CELL, IS KEPT IN W0.            20W0
NONE: PRINT AND FINISH.                         709-2
INPUT THE SYMBOL IN THE CELL 1W0,               12W0
ENTER IT,                                         J156
                                                10K1
LEAVE A SPACE, AND GO ROUND.                      J161  9-1
                                          9-2     J155
                                                  J30   0
                                        5       01
                                          K1    011
                                          L1      0
                                                  S1
                                                  S2
                                                  S3    0
                                        5         P0
```

```output
S1 S2 S3
```

`20 W0` after `J60` does two jobs: it takes the cell's name out of H0,
and it does not disturb H5, so the branch after it still sees what `J60`
found.

## Local subroutines

A piece of a routine can be a routine of its own with a local name, and
be executed with `00 9-5`. It ends with a LINK of `0` like any other. It
belongs to the routine it is written in, and no other routine can reach
it.

## Recursion

Because H0 and the W cells are stacks, and the interpreter keeps its own
place in a stack too, a routine may execute itself. The Towers of Hanoi:
to move *n* discs from one peg to another, move *n* - 1 to the spare peg,
move one disc, and move the *n* - 1 on top of it.

```iplv
THE TOWERS OF HANOI                     9
                                        2 A                 1
                                        2 B                 1
                                        2 C                 1
                                        2 K                10
                                        2 N                10
                                        2 P                10
                                        2 R                10
                                        2 T                10
                                        5       00
BY WAY OF B,                              P0    10B0
TO C,                                           10C0
FROM A,                                         10A0
N0 DISCS.                                       10N0
                                                  R1    0
R1: MOVE (0) DISCS FROM (1) TO (2) BY   1
WAY OF (3).                             1
W0-W3 = DISCS, FROM, TO, VIA.             R1      J53
                                                11W0
NO DISCS.                                         J117
                                                709-1
THEN THERE IS NOTHING TO DO.                      J33   0
                                          9-1   11W0
COPY THE NUMBER OF DISCS,                         J120
                                                40W4
INTO W4,                                        20W4
                                                10K1
                                                11W4
                                                11W4
AND TAKE 1 FROM THE COPY.                         J111
                                                30H0
BY WAY OF THE PEG WE MOVE TO,                   11W2
TO THE SPARE PEG,                               11W3
FROM THE PEG WE MOVE FROM,                      11W1
ONE DISC FEWER.                                 11W4
                                                  R1
PRINT THE MOVE.                                   J154
                                                11W1
                                                  J156
                                                10T0
                                                  J157
                                                10K1
                                                  J161
                                                11W2
                                                  J156
                                                  J155
BY WAY OF THE PEG WE MOVED FROM,                11W1
TO THE PEG WE MOVE TO,                          11W2
FROM THE SPARE PEG,                             11W3
ONE DISC FEWER.                                 11W4
                                                  R1
                                                11W4
ERASE THE COPY.                                   J9
                                                30W4
                                                  J33   0
                                        5       01
                                          K1    011
                                          N0    013
                                          T0    21 TO
                                        5         P0
```

```output
A TO C
A TO B
C TO B
A TO C
B TO A
B TO C
A TO C
```

Three things in it are new.

`J120` copies a data term into a new cell. The routine must not count
`N0` itself down, since every level of the recursion needs its own
number, so each level makes a copy, takes 1 from it, and erases it with
`J9` before it ends.

`J156` enters a *symbol* in the print line, as `J157` enters a data term.
It enters `A0` as `A`: the zeroth symbol of a region is written as its
letter alone.

The regions `A`, `B` and `C` have one cell each. A region card with 1 in
LINK is how you make a symbol that is just a letter.

Run it with `--stats` and you will see how little it used:

```
target/release/iplv run examples/hanoi.iplv --stats
```
