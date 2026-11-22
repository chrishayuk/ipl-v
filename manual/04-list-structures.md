# §4. R3: searching a list structure

*Part One, section 4.0, "List Structures", pages 31 to 39 (PDF pages 66
to 74).*

**R3: Test if (0) occurs anywhere in the list structure named (1).** The
manual writes R3 three times. The first version recurses on each sublist
it meets; the second keeps the test symbol in one place for every level
of the recursion; the third marks each sublist processed, so that a
sublist met twice is searched once, and a structure whose lists name
each other cannot send it round for ever.

All three search the same structure, L6, which the manual gives on page
33 with the addresses it was loaded at.

## The first R3

This is the version the manual traces, step by step, in Fig. 3 (page
34). The deck below is the manual's: R3 as printed on page 32, L6 as
listed on page 33, and in front of them the four instructions the manual
loads at cells 100 to 103 to use R3 (`10 L6`, `10 S4`, `R3`, `70 110`),
with a little more added after them to print the answer.

```iplv
R3, FIRST VERSION, PAGES 32 AND 33      9
                                        2 L                10
                                        2 P                10
                                        2 R                10
                                        2 S                12
                                        2 T                10
                                        5       00
CELL 100 IN FIG. 3.                       P0    10L6
101                                             10S4
102                                               R3
103                                             709-1
                                                10T1    9-2
                                          9-1   10T2
                                          9-2     J154
                                                10S4
                                                  J156
                                                  J157  J155
R3: TEST IF (0) OCCURS ANYWHERE IN THE  1
LIST STRUCTURE NAMED (1).               1
OUTPUT THE TEST SYMBOL TO W0.             R3      J50   9-2
LOCATE THE NEXT CELL.                     9-2     J60
IF NO MORE CELLS, EXIT H5-.                     709-1
                                                12H0
                                                11W0
TEST IF SYMBOL IS TEST SYMBOL.                    J2
IF SO (H5+), EXIT FROM R3 WITH H5+.             70      9-1
                                                12H0
TEST IF SYMBOL IN CELL IS LOCAL.                  J132
IF NOT (H5-), RECYCLE.                          709-2
                                                12H0
                                                11W0
IF SO, IS IT IN THE SUBSTRUCTURE.                 R3
IF R3 GIVES H5-, RECYCLE.                       709-2
COMMON CLEAN-UP FOR ALL EXITS.            9-1   30H0    J30
                                        5       01
                                          L6      0
                                                  S1
                                                  9-1
                                                  S2
                                                  9-2   0
                                          9-1     0
                                                  S3
                                                  9-3
                                                  9-3
                                                  S4    0
                                          9-2     0
                                                  S5
                                                  S6    0
                                          9-3     0
                                                  S7
                                                  S8
                                                  S9
                                                  S10   0
                                          T1    21 YES
                                          T2    21 NO
                                        5         P0
```

```output
S4 YES
```

The manual's Fig. 3 gives the level, H1, the instruction, H5, H0 and W0 at
twenty-five of the 151 steps it takes to find S4. `tests/manual.rs` runs
this deck and holds it to every one of them. A few, to read with the
code:

```text
Level  Step  H1                Instruction  H5  H0                  W0
  1      3   102               R3           +   S4,L6
  2      4   R3,102            J50          +   S4,L6
  2     25   211,102           R3           +   S4,9-1,501          S4
  3     48   R3,211,211,102    J50          +   S4,9-3,601,501      S4,S4
  4     87   213,211,211,102   30HO J30     -   803,601,501         S4,S4,S4
  3    100   211,211,102       R3           +   S4,9-3,602,501      S4,S4
  1    151   103               70110        +
```

Numbers are the manual's cell addresses: R3's instructions are at 200
to 213, L6's cells at 500 to 503 and its sublists' at 600, 700 and 800.
H1 is the push-down list of instruction addresses, one for each routine
in progress; at step 87 the search is four levels down. Under the inputs
in H0 lies the place each level has reached in its own list, which is how
R3 needs no working cell for it. Step 100 is the search of 9-3 a second
time, because 9-3 is on 9-1 twice. (The figure's step 2 says `10L4`; the
listing on page 33 says `10 S4`, and H0 at step 3 agrees with the
listing.)

One thing in the code is new: the card after `J2` has P = 7 and a blank
SYMB. A blank SYMB means the next cell, as a blank LINK does, so `70 [the
next card] 9-1` goes on to the next card if H5 is - and to 9-1 if it is +.

## The second R3

"We want a way to hold the test symbol in common for all levels of the
recursion." Page 36: the test symbol goes in W0 once, and the recursion is
a local subroutine, 9-10, that reads it there.

```iplv
R3, SECOND VERSION, PAGE 36             9
                                        2 L                10
                                        2 P                10
                                        2 R                10
                                        2 S                12
                                        2 T                10
                                        5       00
                                          P0    10L6
                                                10S4
                                                  P1
                                                10L6
                                                10S11
                                                  P1    0
P1: SAY WHETHER R3 FINDS (0) IN (1).    1
                                          P1    40H0
                                                  J154
                                                  J156
                                                  R3
                                                709-1
                                                10T1    9-2
                                          9-1   10T2
                                          9-2     J157  J155
R3: TEST IF (0) OCCURS ANYWHERE IN THE  1
LIST STRUCTURE NAMED (1).               1
SET UP (0) IN W0.                         R3      J50
EXECUTE SUBROUTINE, THEN CLEAN UP.                9-10  J30
IS THE SYMBOL IN W0 IN STRUCTURE (0).     9-10    J60
                                                70J8
                                                12H0
                                                11W0
                                                  J2
                                                70      J8
                                                12H0
                                                  J132
                                                709-10
                                                12H0
CAN BE EXECUTED WITHOUT INPUT FROM W0.            9-10
                                                709-10  J8
                                        5       01
                                          L6      0
                                                  S1
                                                  9-1
                                                  S2
                                                  9-2   0
                                          9-1     0
                                                  S3
                                                  9-3
                                                  9-3
                                                  S4    0
                                          9-2     0
                                                  S5
                                                  S6    0
                                          9-3     0
                                                  S7
                                                  S8
                                                  S9
                                                  S10   0
                                          T1    21 YES
                                          T2    21 NO
                                        5         P0
```

```output
S4 YES
S11 NO
```

The last two instructions show the difference the manual wants seen
between executing a routine and branching to it: `00 9-10` comes back,
and is a recursion; `70 9-10` does not, and is an iteration. Both start
9-10 again.

## The third R3

A list structure may name the same sublist twice, as L6 names 9-3, or
lists may name each other, and then the recursion never ends. The cure
(pages 37 to 39) is to mark each sublist **processed** before searching it,
skip those already marked, and keep a list of what was marked so that the
marks can be taken off at the end.

- **J137** marks a list processed: it preserves the head and puts P = 1
  in it, so the head's old word is one down in its push-down list.
- **J133** tests for the mark.
- Restoring the head takes the mark away.

This is the code of Fig. 4. Two lines in it repay study. `61 W0` copies
(0) into the cell *named in* W0, which is the head of the list of local
names; `41 W0` then preserves that cell, which pushes the name down into
the list. That is how a symbol is put at the front of a list without J64.
And at the end, `32 H0` restores the cell named in the cell whose name is
in H0: the sublist itself, so taking off its mark.

The structure here is L7, whose two sublists name each other. The first
two versions of R3, given L7, never stop.

```iplv
R3, THIRD VERSION: FIG. 4, PAGE 39      9
                                        2 L                10
                                        2 P                10
                                        2 R                10
                                        2 S                12
                                        2 T                10
                                        5       00
                                          P0    10L7
                                                10S4
                                                  P1
                                                10L7
                                                10S11
                                                  P1
                                                10L7
THE MARKS ARE ALL GONE.                           J150  0
P1: SAY WHETHER R3 FINDS (0) IN (1).    1
                                          P1    40H0
                                                  J154
                                                  J156
                                                  R3
                                                709-1
                                                10T1    9-2
                                          9-1   10T2
                                          9-2     J157  J155
R3: TEST IF (0) OCCURS ANYWHERE IN THE  1
LIST STRUCTURE NAMED (1).               1
CREATE LOCAL NAME LIST.                   R3      J90
NAME LIST IN W0, TEST SYMBOL IN W1.               J51
RECURSIVE SUBROUTINE: + OR -.                     9-10
PUSH DOWN H5 TO SAVE RESULT.                    40H5
                                                11W0
LOCATE NEXT SYMBOL ON LOCAL NAME LIST.    9-2     J60
                                                709-1
POP IT UP, THUS UNMARKING IT.                   32H0    9-2
INPUT LOCAL NAME LIST.                    9-1   51W0
ERASE LIST.                                       J71
POP UP H5; CLEAN UP W'S.                        30H5    J31
RECURSIVE SUBROUTINE; MARK PROCESSED.     9-10    J137
PUT IN TOP CELL OF LOCAL NAME LIST.             61W0
PUSH DOWN LOCAL NAME LIST.                      41W0
LOCATE NEXT SYMBOL ON LIST.               9-11    J60
IF AT END, EXIT.                                70J8
                                                12H0
                                                11W1
IS IT THE TEST SYMBOL (IN W1).                    J2
IF SO, EXIT +.                                  70      J8
                                                12H0
TEST IF LOCAL.                                    J132
IF NOT, PROCEED TO NEXT SYMBOL.                 709-11
                                                12H0
ALREADY MARKED PROCESSED.                         J133
IF SO, PROCEED TO NEXT SYMBOL.                  70      9-11
                                                12H0
RECURSE ON UNSEARCHED SUBLIST.                    9-10
IF H5+, FOUND SYMBOL, QUIT.                     709-11  J8
                                        5       01
                                          L7      0
                                                  S1
                                                  9-1   0
                                          9-1     0
                                                  S2
                                                  9-2
                                                  S3    0
                                          9-2     0
                                                  9-1
                                                  S4    0
                                          T1    21 YES
                                          T2    21 NO
                                        5         P0
```

```output
S4 YES
S11 NO
L7       0
         S1
         9-1
9-1      0
         S2
         9-2
         S3
9-2      0
         9-1
         S4
```

The structure printed at the end has no marks left in it.
