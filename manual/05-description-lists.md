# §6. Description lists

*Part One, section 6.0, "Description Lists", pages 48 to 50 (PDF pages 83
to 85).*

The manual introduces description lists with a bridge hand: X1 is a list
of the thirteen card symbols, and its description list says that its
point count (attribute Q0) is N2 and the length of its longest suit (Q1)
is N5. Page 49 writes it so:

```text
NAME  PQ SYMB  LINK
X1       9-1
         C13
         C12
         ...
         S3     0
9-1      0
         Q0
         N2
         Q1
         N5     0
```

Fig. 5 (page 50) then traces five uses of J10, J11 and J14, "each
independently using list X1 above", and shows the description list after
each. The deck below does the same five things, each to a fresh copy of
X1 made with J74 (which copies the description list too, since its name
is local), and prints the answer: the value J10 found, or `-`, or the
description list as it is afterwards, with J151, which names it by its
address. The manual's "..." is filled in
with the hand that section 7 uses.

```iplv
FIG. 5: DESCRIPTION LIST PROCESSES      9
                                        2 C                15
                                        2 D                15
                                        2 F                15
                                        2 N                10
                                        2 P                10
                                        2 Q                10
                                        2 S                15
                                        2 T                10
                                        2 X                10
                                        5       00
FIND THE VALUE OF Q0.                     P0    10X1
                                                  J74
                                                  J50
                                                11W0
                                                10Q0
                                                  J10
                                                  P1
                                                  P3
FIND THE VALUE OF Q3.                           10X1
                                                  J74
                                                  J50
                                                11W0
                                                10Q3
                                                  J10
                                                  P1
                                                  P3
ASSIGN N3 AS THE VALUE OF Q0.                   10X1
                                                  J74
                                                  J50
                                                11W0
                                                10N3
                                                10Q0
                                                  J11
                                                11W0
                                                  P2
                                                  P3
ASSIGN N6 AS THE VALUE OF Q4.                   10X1
                                                  J74
                                                  J50
                                                11W0
                                                10N6
                                                10Q4
                                                  J11
                                                11W0
                                                  P2
                                                  P3
ERASE ATTRIBUTE Q1.                             10X1
                                                  J74
                                                  J50
                                                11W0
                                                10Q1
                                                  J14
                                                11W0
                                                  P2
                                                  P3    0
P1: PRINT WHAT J10 FOUND, OR - IF IT    1
FOUND NOTHING.                          1
                                          P1      J154
                                                709-1
                                                  J156  J155
                                          9-1   10T0
                                                  J157  J155
P2: PRINT THE DESCRIPTION LIST OF (0).  1
THE NAME OF THE DESCRIPTION LIST.         P2    52H0
                                                  J151  0
P3: ERASE THE COPY IN W0.               1
                                          P3    11W0
                                                  J72
                                                  J30   0
                                        5       01
                                          T0    21-
                                          X1      9-1
                                                  C13
                                                  C12
                                                  C3
                                                  F11
                                                  F7
                                                  F6
                                                  D7
                                                  D4
                                                  S1
                                                  S11
                                                  S10
                                                  S5
                                                  S3    0
                                          9-1     0
                                                  Q0
                                                  N2
                                                  Q1
                                                  N5    0
                                        5         P0
```

```output
N2
-
243      0
         Q0
         N3
         Q1
         N5
264      0
         Q4
         N6
         Q0
         N2
         Q1
         N5
255      0
         Q0
         N2
```

Fig. 5 has the same five results: J10 finds N2 for Q0, and nothing for
Q3, with H5 -; J11 replaces the value of an attribute that is there, and
puts a new one "at the front of the description list"; J14 removes an
attribute and its value together.

P2 uses a habit of the manual's: `52 H0` replaces (0), the name of the
described list, by the symbol in its head, which is the name of its
description list. Nothing in IPL-V hides that a description list is an
ordinary list whose name is kept in the head.
