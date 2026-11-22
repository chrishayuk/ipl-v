# Tracing, Reading Cards, and the Page

## Trace marks

When a routine does the wrong thing, you want to see every instruction it
interprets. IPL-V builds that into Q. In an instruction, Q = 3 and Q = 4
designate SYMB itself, exactly as Q = 0 does, and also:

- **Q = 3** turns tracing on for the routine it is in.
- **Q = 4** turns tracing on if the routine that executed this one is
  being traced.

The trace does not go down into routines by itself, so you choose what to
see: mark the routine you are worried about with a 3 in its first
instruction, and the routines under it that you also want with a 4.

This is the first complete program in the manual, traced as the manual
traces it. `F1` takes a list and makes a list of its symbols without the
repetitions.

```iplv
THE MANUAL'S FIRST PROGRAM              9
                                        2 A                10
                                        2 B                10
                                        2 C                10
                                        2 E                10
                                        2 F                10
                                        2 K                10
                                        5       00
Q = 3: TRACE THIS ROUTINE.                E1    13K1
                                                  F1
                                                  J151  0
                                        5       00
F1: A LIST OF THE DISTINCT SYMBOLS ON   1
LIST (0).                               1
Q = 4: TRACE IF E1 IS TRACED.             F1    04J90
                                                40W0
W0 HOLDS THE NAME OF THE OUTPUT LIST.           20W0
LOCATE THE NEXT CELL OF THE INPUT LIST.   9-1     J60
                                                709-2
INPUT THE SYMBOL IN THE CELL.                   12H0
                                                11W0
                                                  J6
INSERT IT AT THE END IF IT IS NOT THERE.          J66   9-1
PUT THE OUTPUT LIST'S NAME IN H0,         9-2   51W0
AND RESTORE W0.                                 30W0    0
                                        5       01
                                          K1      0
                                                  B1
                                                  C1
                                                  B1    0
                                        5         E1
```

```output
  1 E1           + 13 K1                   0       00 0     0       5
  1 E1+1         + 00 F1                   K1      04 0     136     6
  2 F1           + 04 J90                  K1      04 0     136     7
  2 F1+1         + 40 W0                   142     04 0     0       9
  2 F1+2         + 20 W0    9-1            142     04 0     0       10
  2 F1/9-1       + 00 J60                  K1      04 0     136     11
  2 F1+4         + 70 9-2                  136     00 B1    137     13
  2 F1+5         + 12 H0           B1      136     00 B1    137     14
  2 F1+6         + 11 W0           142     B1      04 0     0       15
  2 F1+7         + 00 J6                   142     04 0     0       16
  2 F1+8         + 00 J66   9-1            B1      04 0     0       18
  2 F1/9-1       + 00 J60                  136     00 B1    137     20
  2 F1+4         + 70 9-2                  137     00 C1    138     22
  2 F1+5         + 12 H0           C1      137     00 C1    138     23
  2 F1+6         + 11 W0           142     C1      04 0     0       24
  2 F1+7         + 00 J6                   142     04 0     143     25
  2 F1+8         + 00 J66   9-1            C1      04 0     0       27
  2 F1/9-1       + 00 J60                  137     00 C1    138     29
  2 F1+4         + 70 9-2                  138     00 B1    0       31
  2 F1+5         + 12 H0           B1      138     00 B1    0       32
  2 F1+6         + 11 W0           142     B1      04 0     0       33
  2 F1+7         + 00 J6                   142     04 0     143     34
  2 F1+8         + 00 J66   9-1            B1      04 0     0       36
  2 F1/9-1       + 00 J60                  138     00 B1    0       38
  2 F1+4         - 70 9-2                  138     00 B1    0       40
  2 F1/9-2       - 51 W0           142     138     00 B1    0       41
  2 F1+10        - 30 W0    0              142     04 0     143     42
  1 E1+2         - 00 J151  0              142     04 0     143     43
142      0
         B1
         C1
```

Each line of the trace is one instruction, *before* it is done:

| column | |
|---|---|
| 1 | the level: 1 for the routine the start card named, 2 for one it executes |
| 2 | where the instruction is: a name, a name and a local symbol, or a name and a count of cards |
| 3 | H5 |
| 4-6 | P Q, SYMB, and LINK if it is not just the next card |
| 7 | S, when Q made it different from SYMB |
| 8 | (0), the symbol in H0 |
| 9 | the word in the cell (0) names |
| 10 | H3, the count of interpretation cycles |

Follow (0) down the trace and you can see the routine work: `K1`, then
the new list `138`, then cell after cell of the input with `12 H0`
fetching `B1`, `C1`, `B1`.

H3 goes up by one for an instruction and by two for one that executes a
basic process. That is how the original system counted, and you can check
it: the manual prints this trace, for a longer list, in Fig. 2 of Part
One, and `tests/manual.rs` holds every line of it against this machine.

From the command line, `--trace` traces everything whatever the marks
say, and `--no-trace` nothing.

## Reading cards

The cards behind the start card are not part of the program. They stay in
the reader, and the program reads them when it likes.

- **J180** Read line: the next card goes into the print line, which is
  also the read line. H5 is `-` if there are no more.
- **J182** Input line data term (0): the field at the entry column is
  read into the data term (0), as a number if (0) is a number.
- **J181** Input line symbol, and **J186** Input line character.
- **J183** and **J184** find the next blank and the next non-blank.

The field is five columns wide unless you change the number that `W30`
names. Reading a card puts the entry column back at 1.

```iplv
ADDING UP THE NUMBERS ON THE CARDS      9
                                        2 P                10
                                        2 T                10
                                        2 X                10
                                        5       00
READ A CARD.                              P0      J180
NO MORE CARDS.                                  709-2
                                                10X0
COLUMNS 1-5, AS AN INTEGER, INTO X0.              J182
THE FIELD WAS BLANK.                            709-1
                                                10T0
                                                10T0
T0 + X0 INTO T0.                                  J110
                                          9-1   30H0    P0
                                          9-2     J154
                                                10T0
                                                  J157  J155
                                        5       01
                                          T0    010
                                          X0    010
                                        5         P0
```

```input
   12
  100

    7
 1961
```

```output
2080
```

From the command line the cards can be in the same file, after the start
card, or in a file of their own:

```
target/release/iplv run examples/sum.iplv examples/sum.in
```

The Logic Theorist reads its theorems this way, a formula to a card, and
takes them apart character by character with `J186`.

## The page

```
node web/build.mjs
python3 -m http.server -d web 8000
```

and open http://localhost:8000. It is this same machine, with nothing
hidden:

- the deck, with the card being interpreted marked, and the address each
  card was loaded at;
- the instruction in words, H5, H3, what is in H0 and the working cells,
  and the routines in progress, which is the push-down list of H1;
- storage, a square for each cell, coloured by what it is being used
  for, with the cells that have just changed in yellow;
- any list structure, drawn. Leave the box empty and it draws whatever
  (0) names, which follows the program's attention; or name a symbol.

Step through `manual-f1` with the drawing on `W0` and you will see the
output list grow a cell at a time, each cell appearing in storage as it
is taken from available space.

## Where to go from here

`examples/sentences.iplv` is a complete small program in the style of the
period: a grammar held as a list structure, and one recursive routine
that expands a symbol by choosing among alternatives at random, using a
generator on itself.

The real thing is the Logic Theorist, in Einar Stefferud's IPL-V version
of 1963: about 2,800 cards. `sources/README.md` says where to get the
deck, and with it

```
L=sources/shrager/ExecutableArchaeology/LogicTheorist
target/release/iplv run $L/logic-theorist-1963-stefferud.iplv $L/logic-theorist-1963-stefferud-input.txt
```

proves 23 theorems of *Principia Mathematica*, printing each proof as it
did in 1963. By now you can read it. Start at `M1`.
