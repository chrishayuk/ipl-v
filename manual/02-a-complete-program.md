# §2. A complete program

*Part One, section 2.0, "A Complete Program", pages 17 to 23 (PDF pages 52
to 58).*

F1 makes a list of the distinct symbols on a list: B1 C1 B1 A1 A1 B1
becomes B1 C1 A1. It is the first program the manual carries "all the way
through to assembly and execution on the computer", and it prints both:
the assembly listing as Fig. 1 and the trace as Fig. 2. Both were made on
the IPL-V system itself, which makes this the one place where the
original's behaviour is printed instruction by instruction, with the
addresses it loaded the program at and the count of interpretation
cycles.

The deck is the program of Fig. 1, card for card, with the manual's
comments. E1 has Q = 3 in its first instruction, to trace it; F1 has Q =
4, to trace it when E1 is traced.

```iplv
EXAMPLE RUN OF F1, FROM THE MANUAL      9
IPL-V MANUAL, PART ONE, SECTION 2:      1
THE PROGRAM OF FIG. 1 AND FIG. 2.       1
                                        2 A                10
                                        2 B                10
                                        2 C                10
                                        2 E                10
                                        2 F                10
                                        2 K                10
                                        5       00
E1---EXECUTIVE ROUTINE.                 1
EXECUTE F1 WITH THE LIST K1             1
AS INPUT AND PRINT THE                  1
RESULTING OUTPUT LIST.                  1
Q=3 SAYS TRACE THIS ROUTINE.            1
                                          E1    13K1
EXECUTE F1 AND                                    F1
PRINT THE OUTPUT LIST.                            J151  0
                                        5       00
F1---PRODUCE AN OUTPUT LIST (0)         1
CONTAINING ONLY ONE OCCURRENCE OF       1
EACH DISTINCT SYMBOL ON THE INPUT       1
LIST (0). THE ROUTINE F1 IS             1
MARKED TO TRACE CONDITIONALLY WITH      1
Q=4 IN ITS FIRST INSTRUCTION.           1
CREATE AN EMPTY OUTPUT LIST.              F1    04J90
PRESERVE W0.                                    40W0
W0 HOLDS NAME OF OUTPUT LIST.                   20W0
LOCATE NEXT CELL OF INPUT LIST.           9-1     J60
GO TO 9-2 IF NO NEXT CELL.                      709-2
INPUT THE SYMBOL IN THE CELL.                   12H0
INPUT THE NAME OF THE OUTPUT LIST,              11W0
REVERSE THEIR POSITION IN H0, AND                 J6
ADD THE SYMBOL TO THE OUTPUT LIST                 J66   9-1
IF IT IS NOT ALREADY ON IT---           1
RETURN TO 9-1 IN EITHER CASE.           1
PUT THE NAME OF THE OUTPUT LIST           9-2   51W0
IN H0 AND RESTORE W0 BEFORE                     30W0    0
QUITTING.                               1
                                        5       01
THE LIST K1.                              K1      0
                                                  B1
                                                  C1
                                                  B1
                                                  A1
                                                  A1
                                                  B1    0
                                        5         E1
```

```output
  1 E1           + 13 K1                   0       00 0     0       5
  1 E1+1         + 00 F1                   K1      04 0     136     6
  2 F1           + 04 J90                  K1      04 0     136     7
  2 F1+1         + 40 W0                   145     04 0     0       9
  2 F1+2         + 20 W0    9-1            145     04 0     0       10
  2 F1/9-1       + 00 J60                  K1      04 0     136     11
  2 F1+4         + 70 9-2                  136     00 B1    137     13
  2 F1+5         + 12 H0           B1      136     00 B1    137     14
  2 F1+6         + 11 W0           145     B1      04 0     0       15
  2 F1+7         + 00 J6                   145     04 0     0       16
  2 F1+8         + 00 J66   9-1            B1      04 0     0       18
  2 F1/9-1       + 00 J60                  136     00 B1    137     20
  2 F1+4         + 70 9-2                  137     00 C1    138     22
  2 F1+5         + 12 H0           C1      137     00 C1    138     23
  2 F1+6         + 11 W0           145     C1      04 0     0       24
  2 F1+7         + 00 J6                   145     04 0     146     25
  2 F1+8         + 00 J66   9-1            C1      04 0     0       27
  2 F1/9-1       + 00 J60                  137     00 C1    138     29
  2 F1+4         + 70 9-2                  138     00 B1    139     31
  2 F1+5         + 12 H0           B1      138     00 B1    139     32
  2 F1+6         + 11 W0           145     B1      04 0     0       33
  2 F1+7         + 00 J6                   145     04 0     146     34
  2 F1+8         + 00 J66   9-1            B1      04 0     0       36
  2 F1/9-1       + 00 J60                  138     00 B1    139     38
  2 F1+4         + 70 9-2                  139     00 A1    140     40
  2 F1+5         + 12 H0           A1      139     00 A1    140     41
  2 F1+6         + 11 W0           145     A1      04 0     0       42
  2 F1+7         + 00 J6                   145     04 0     146     43
  2 F1+8         + 00 J66   9-1            A1      04 0     0       45
  2 F1/9-1       + 00 J60                  139     00 A1    140     47
  2 F1+4         + 70 9-2                  140     00 A1    141     49
  2 F1+5         + 12 H0           A1      140     00 A1    141     50
  2 F1+6         + 11 W0           145     A1      04 0     0       51
  2 F1+7         + 00 J6                   145     04 0     146     52
  2 F1+8         + 00 J66   9-1            A1      04 0     0       54
  2 F1/9-1       + 00 J60                  140     00 A1    141     56
  2 F1+4         + 70 9-2                  141     00 B1    0       58
  2 F1+5         + 12 H0           B1      141     00 B1    0       59
  2 F1+6         + 11 W0           145     B1      04 0     0       60
  2 F1+7         + 00 J6                   145     04 0     146     61
  2 F1+8         + 00 J66   9-1            B1      04 0     0       63
  2 F1/9-1       + 00 J60                  141     00 B1    0       65
  2 F1+4         - 70 9-2                  141     00 B1    0       67
  2 F1/9-2       - 51 W0           145     141     00 B1    0       68
  2 F1+10        - 30 W0    0              145     04 0     146     69
  1 E1+2         - 00 J151  0              145     04 0     146     70
145      0
         B1
         C1
         A1
```

`tests/manual.rs` holds this run to the manual's figures:

- every word is loaded at the address Fig. 1 gives it: the regions from
  cell 64, E1's instructions at 95, 124 and 125, F1's at 105 and 126 to
  135, the list K1 at 115 and 136 to 141; and the listing's last line,
  `NUMBER OF CELLS ON AVAILABLE SPACE=21858`, is the same;
- all 46 lines of Fig. 2 are the same in level, instruction address, H5,
  P, Q, SYMB, LINK and H3.

The trace above differs from Fig. 2 in three ways: the list the program
makes is numbered 145 here where Fig. 2 has 148 (the system of 1964 had
used three more cells of available space by then; for what, the manual
does not say); the layout of the line is this implementation's; and Fig.
2 ends with the system's message `PROGRAM RAN TO COMPLETION.`, which this
implementation does not print. (An earlier version of this page said
"two ways only" and missed the message.) `iplv run examples/manual-f1.iplv --list` prints the
assembly listing.
