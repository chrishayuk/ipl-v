# §8 and §9. Generators

*Part One, section 8.0, "Using Generators", pages 68 to 72 (PDF pages 103
to 107), and section 9.0, "Generator Construction", pages 74 to 81 (PDF
pages 109 to 116).*

## A generator with a bug in it

R77 (page 72) uses J102, the generator of the cells of a tree, to print
the algebraic expression L10 of Problem 33 in Polish prefix form: X+Y as
+XY. "The subprocess 9-10 contains a common error: It fails to set H5
properly."

L10 represents (A+B)-(A-B)=(A-B)+(A+B), as a tree of sublists, each with
its operator first (page 41). The code says `10 L5` where the text says
L10; L5 is a list of page 23 that has no operators in it, so we take the
text at its word.

```iplv
R77 AS PRINTED, PAGE 72                 9
                                        2 A                 1
                                        2 B                 1
                                        2 L                15
                                        2 R               100
                                        2 +                 1
                                        2 -                 1
                                        2 =                 1
                                        5       00
INPUT STRUCTURE NAME.                     R77   10L10
INPUT SUBPROCESS NAME, DO J102.                 109-10  J102
SKIP HEADS OF SUBLISTS.                   9-10  70      J8
OPERATOR OR OPERAND TO H0.                      52H0
SAVE IT IN CASE WE PRINT.                       40H0
TEST FOR LOCAL.                                   J132
PRINT IF NOT LOCAL; OTHERWISE POP H0.           70J152  J8
                                        5       01
                                          L10     0
                                                  =
                                                  9-1
                                                  9-2   0
                                          9-1     0
                                                  -
                                                  9-10
                                                  9-11  0
                                          9-10    0
                                                  +
                                                  A
                                                  B     0
                                          9-11    0
                                                  -
                                                  A
                                                  B     0
                                          9-2     0
                                                  +
                                                  9-20
                                                  9-21  0
                                          9-20    0
                                                  -
                                                  A
                                                  B     0
                                          9-21    0
                                                  +
                                                  A
                                                  B     0
                                        5         R77
```

```output
         =0
```

"R77 would terminate after printing the first operator, since J132 in the
subprocess 9-10 has set H5-, which the generator J102 interprets as a
signal to terminate. H5 should be set + after the negative branch is
selected and before 9-10 terminates." Done so:

```iplv
R77 WITH H5 SET + AFTER PRINTING        9
                                        2 A                 1
                                        2 B                 1
                                        2 L                15
                                        2 R               100
                                        2 +                 1
                                        2 -                 1
                                        2 =                 1
                                        5       00
INPUT STRUCTURE NAME.                     R77   10L10
INPUT SUBPROCESS NAME, DO J102.                 109-10  J102
SKIP HEADS OF SUBLISTS.                   9-10  70      J8
OPERATOR OR OPERAND TO H0.                      52H0
SAVE IT IN CASE WE PRINT.                       40H0
TEST FOR LOCAL.                                   J132
IF NOT LOCAL, PRINT; OTHERWISE POP H0.          709-11  J8
PRINT, AND SET H5 + TO GO ON.             9-11    J152  J4
                                        5       01
                                          L10     0
                                                  =
                                                  9-1
                                                  9-2   0
                                          9-1     0
                                                  -
                                                  9-10
                                                  9-11  0
                                          9-10    0
                                                  +
                                                  A
                                                  B     0
                                          9-11    0
                                                  -
                                                  A
                                                  B     0
                                          9-2     0
                                                  +
                                                  9-20
                                                  9-21  0
                                          9-20    0
                                                  -
                                                  A
                                                  B     0
                                          9-21    0
                                                  +
                                                  A
                                                  B     0
                                        5         R77
```

```output
         =0
         -0
         +0
         A0
         B0
         -0
         A0
         B0
         +0
         -0
         A0
         B0
         +0
         A0
         B0
```

`= - + A B - A B + - A B + A B`, one symbol to a line: the prefix form of
(A+B)-(A-B)=(A-B)+(A+B). J152 writes A as `A0`, the regional symbol it is.

## J100, wrongly and rightly

Section 9 begins by writing J100 the obvious way (page 74), a loop down
the list that executes the subprocess with `01 W0`, and uses it in the
version of J77 from page 68, whose subprocess compares each symbol with
the test symbol it expects to find in W0. The manual traces the first
few steps and stops at the line marked ERROR: the subprocess "finds 9-10
instead". The generator has put its own things in the working cells, and
the subprocess, working "in the same context as J77", finds them there
instead of J77's.

As with J77 in section 1, the manual's versions of J processes are named
here in region U, and the wrong J100 is V100.

```iplv
J77 WITH THE WRONG J100, PAGES 68 AND 749
                                        2 K                10
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        2 T                10
                                        2 U               100
                                        2 V               101
                                        5       00
V100: J100 AS FIRST WRITTEN ON PAGE 74. 1
SUBPROCESS IN W0, LIST IN W1.             V100    J51
INPUT LOCATION ON LIST.                   9-2   11W1
LOCATE NEXT.                                      J60
PUT BACK IN W1 PRIOR TO BRANCH.                 20W1
                                                709-1
SYMBOL IN CELL FOR SUBPROCESS.                  12W1
EXECUTE SUBPROCESS (P=0, Q=1).                  01W0
IF SUBPROCESS GIVES H5-, QUIT H5-.              70J31   9-2
IF WENT TO END OF LIST, QUIT H5+.         9-1     J4    J31
U77: J77 WRITTEN WITH A GENERATOR, PAGE 1
68, USING V100.                         1
PUT TEST SYMBOL IN W0.                    U77     J50
INPUT NAME OF SUBPROCESS.                       109-10
EXECUTE GENERATOR.                                V100
REVERSE SIGN.                                     J5    J30
SUBPROCESS: INPUT TEST SYMBOL.            9-10  11W0
TEST; REVERSE SIGN TO STOP.                       J2    J5
                                        5       00
                                          P0    10S2
                                                  P1
                                                10S4
                                                  P1
                                                10S9
                                                  P1    0
P1: SAY WHETHER U77 FINDS (0) ON L1.    1
                                          P1      J154
                                                40H0
                                                  J156
                                                10L1
                                                  J6
                                                  U77
                                                709-1
                                                10T1    9-2
                                          9-1   10T2
                                          9-2     J157  J155
                                        5       01
                                          L1      0
                                                  S2
                                                  S4    0
                                          T1    21 YES
                                          T2    21 NO
                                        5         P0
```

```output
S2 NO
S4 NO
S9 NO
```

Every answer is NO, because the symbol the subprocess finds in W0 is
always the subprocess's own name. Here is the J100 of page 75, and the
same J77 using it:

```iplv
J77 WITH THE J100 OF PAGE 75            9
                                        2 K                10
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        2 T                10
                                        2 U               101
                                        5       00
U100: J100 AS WRITTEN ON PAGE 75.       1
THE SYMBOL W0 FOR J17: ONE CELL.          U100  10W0
SET UP HOUSEKEEPING.                              J17
LOCATE NEXT SYMBOL.                       9-1     J60
PUT BACK IN W0 PRIOR TO BRANCH.                 20W0
END OF LIST: EXIT WITH J19.                     70J19
INPUT SYMBOL FOR SUBPROCESS.                    12W0
EXECUTE SUBPROCESS; CHANGES CONTEXTS.             J18
SUBPROCESS GIVES H5-: EXIT, J19.                70J19
INPUT CURRENT LOCATION AND CYCLE.               11W0    9-1
U77: J77 WRITTEN WITH A GENERATOR, PAGE 1
68, USING U100.                         1
PUT TEST SYMBOL IN W0.                    U77     J50
INPUT NAME OF SUBPROCESS.                       109-10
EXECUTE GENERATOR.                                U100
REVERSE SIGN.                                     J5    J30
SUBPROCESS: INPUT TEST SYMBOL.            9-10  11W0
TEST; REVERSE SIGN TO STOP.                       J2    J5
                                        5       00
                                          P0    10S2
                                                  P1
                                                10S4
                                                  P1
                                                10S9
                                                  P1    0
P1: SAY WHETHER U77 FINDS (0) ON L1.    1
                                          P1      J154
                                                40H0
                                                  J156
                                                10L1
                                                  J6
                                                  U77
                                                709-1
                                                10T1    9-2
                                          9-1   10T2
                                          9-2     J157  J155
                                        5       01
                                          L1      0
                                                  S2
                                                  S4    0
                                          T1    21 YES
                                          T2    21 NO
                                        5         P0
```

```output
S2 YES
S4 YES
S9 NO
```

J17 preserves W0 and puts the subprocess's name away; J18 takes the
generator's symbols out of the W cells while the subprocess runs and puts
them back after; J19 restores the W cells and sets H5 to what the last
subprocess said. The manual: "the problem is one of changing contexts of
information. 9-10 works in the same context as J77; J100 works in an
entirely different context."

## J101 and J72, as the manual writes them

J101 generates every cell of a list structure, in "print order": a list,
then its sublists in the order they were first met, each once (page 77).
To know which sublists it has met it marks each one processed, which
pushes its head down. So the head handed to the subprocess holds the
mark, and its own word comes next, as a cell of its own (page 78).

Fig. 11 (pages 80 and 81) is the code. It keeps two lists: a record list
of every sublist marked, so that the marks can be taken off at the end,
and a generation list whose head holds the next cell to be generated and
whose other cells are the sublists still to come. Two tricks from section
4 carry it: `61 W0` and `41 W0` put a name at the front of the record
list through its head, and `41 W2` with `21 W2` add one at the end of the
generation list. When the generation list runs out, `31 W1` restores its
last cell, which leaves a termination cell there, and J80 says so.

J72, ERASE LIST STRUCTURE, is then four instructions and a subprocess
(page 78). It works because J101 finds the next cell before the
subprocess sees the present one, so the subprocess may erase it.

The deck runs the built-in J101 and the manual's U101 on L1 of page 77 and
on L2, two lists that name each other, with one subprocess that prints `/`
for the head of a list and each regional or internal symbol in a list
cell. Then it erases L1 with the manual's J72, counting the cells of
available space before and after.

```iplv
J101 AND J72 AS CODED ON PAGES 78-81    9
                                        2 G                10
                                        2 K                10
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        2 T                10
                                        2 U               102
                                        5       00
                                          P0    10L1
                                                10J101
                                                  P1
                                                10L1
                                                10U101
                                                  P1
                                                10L2
                                                10J101
                                                  P1
                                                10L2
                                                10U101
                                                  P1
COUNT AVAILABLE SPACE,                            P2
                                                10L1
ERASE L1 WITH U72,                                U72
COUNT AGAIN,                                      P2
                                                10L1
AND PRINT WHAT IS LEFT OF L1.                     J150  0
P1: GENERATE LIST STRUCTURE (1) WITH THE1
GENERATOR (0), PRINTING WHAT IS         1
GENERATED ON ONE LINE.                  1
                                          P1      J154
THE GENERATOR.                                  20W9
                                                10G1
                                                11W9
                                                  J1
                                                  J155  0
G1: THE SUBPROCESS. A HEAD IS /, A LIST 1
CELL ITS SYMBOL UNLESS LOCAL.           1
                                          G1    709-1
A HEAD.                                         30H0
                                                10T0
                                                  J157  9-3
THE SYMBOL IN THE CELL.                   9-1   52H0
                                                40H0
                                                  J132
                                                709-2
LOCAL: SAY NOTHING.                             30H0    J4
                                          9-2     J156
                                          9-3   10K1
                                                  J161  J4
P2: PRINT THE NUMBER OF CELLS ON        1
AVAILABLE SPACE.                        1
                                          P2    10H2
                                                  J126
                                                  J154
                                                40H0
                                                  J157
                                                  J155
                                                  J9    0
U101: J101, FIG. 11. GENERATE THE CELLS 1
OF LIST STRUCTURE (1) FOR SUBPROCESS    1
(0).                                    1
SET UP:                                   U101  10W2
                                                  J17
BRING LIST IN FROM AUXILIARY.                     J105
CREATE GENERATION LIST                            J90
                                                40H0
AND RECORD LIST.                                  J90
W0 RECORD, W1 GENERATION, W2 INSERT.              J22
                                                  J4
SET H5+ AND SAVE FOR SUBPROCESS.                40H5
PUSH DOWN TO MATCH ENTRY.                       40H0    9-1
FIND NEXT CELL:                           9-4   40H0
                                                  J60
NEXT CELL TO HEAD OF GENERATION LIST.           21 W1
IS CURRENT CELL END OF SUBLIST.                 709-2   9-9
OBTAIN NEXT SUBLIST.                      9-2   31W1
EXECUTE SUBPROCESS: POP UP H5.            9-9   30H5
                                                  J18
DOES SUBPROCESS SAY QUIT (H5-).                 709-3
DIAGNOSE CURRENT CELL:                          11W1
FIND SYMBOL IN CELL.                              J80
                                                709-3
                                                40H0
TEST IF MARKED PROCESSED.                         J133
SAVE SIGN FOR SUBPROCESS.                       40H5
                                                70      9-4
                                                40H0
TEST IF A DATA TERM.                              J131
                                                70      9-2
                                                12H0
SYMBOL IN CELL ALREADY PROCESSED.                 J133
                                                70      9-4
                                                12H0
SYMBOL IN CELL LOCAL.                             J132
                                                709-4
PROCESS NEW SUBLIST:                            11W1
TEST IF ONLY ONE SUBLIST.                         J60
                                                70      9-10
RESET INSERT LOCATION.                          20W2
ADD TO END OF GENERATION LIST.            9-11  41W2
                                                11W2
ADVANCE INSERT POINTER TO NEW CELL.               J60
                                                20W2
ADD A CELL TO FRONT OF RECORD LIST.             41W0
                                                12H0
MARK CELL PROCESSED.                      9-1     J137
SUBLIST NAME IN RECORD LIST (HEAD).             61W0
SUBLIST NAME IN GENERATION LIST.                21 W2   9-4
                                          9-10  30H0    9-11
CLEAN UP:                                 9-3   11W1
ERASE REST OF GENERATION LIST.                    J71
CLEAN UP RECORD LIST:                     9-8   11W0
TEST IF END OF RECORD LIST.                       J80
                                                709-5
POP UP SUBLIST: REMOVE THE MARK.                31H0
WAS SUBLIST ERASED.                               J80
                                                709-6
DISCARD SYMBOL FROM J80.                        30H0    9-7
                                          9-6   12W0
ERASE CELL FOR NON-EXISTING SUBLIST.              J9
NEXT SUBLIST ON RECORD LIST.              9-7   31W0    9-8
                                          9-5   11W0
ERASE HEAD OF RECORD LIST; QUIT.                  J9    J19
U72: J72, PAGE 78. ERASE LIST STRUCTURE 1
(0), WITH U101.                         1
SAVE H5, SINCE H5 SAFE OVER J'S.          U72   40H5
INPUT SUBPROCESS; LIST IN H0.                   109-10
                                                  U101
RESTORE ORIGINAL H5 AND QUIT.                   30H5    0
TEST IF START OF SUBLIST:                 9-10  709-11
IF YES, DISENGAGE HEAD, ERASE CELL;               J75   J9
IF NO, ERASE CELL AND SET H5+.            9-11    J9    J4
                                        5       01
                                          K1    011
                                          T0    21/
                                          L1      0
                                                  S1
                                                  9-1
                                                  9-2
                                                  S2    0
                                          9-1     0
                                                  9-3
                                                  9-3
                                                  S3    0
                                          9-2     0
                                                  9-4   0
                                          9-3     0
                                                  S5    0
                                          9-4     0
                                                  S6    0
                                          L2      0
                                                  9-1   0
                                          9-1     0
                                                  9-1   0
                                        5         P0
```

```output
/ 0 S1 S2 / 0 S3 / 0 / 0 S5 / 0 S6
/ 0 S1 S2 / 0 S3 / 0 / 0 S5 / 0 S6
/ 0 / 0
/ 0 / 0
21650
21664
L1       0
```

The built-in J101 and the manual's give the same cells in the same order
with the same signs: each list's head, then its own word (the `0`, which
the processed mark pushed down), then its cells; L1, then 9-1 and 9-2 as
met on L1, then 9-3 (on 9-1, twice, generated once), then 9-4. L2 and its
sublist, which contains its own name, are each generated once.

L1 then comes to pieces cell by cell under J72 written in IPL, which
returns 14 cells to available space: all of L1's fifteen but its head,
which is a regional cell and stays, empty. U101's two working lists are
gone too: the counts differ by exactly the cells of L1.
