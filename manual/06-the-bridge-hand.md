# §7. The bridge hand

*Part One, section 7.0, "Description Lists, Continued", pages 53 to 64
(PDF pages 88 to 99), and section 8.0, page 70 (PDF page 105).*

"As we pose more complicated problems, we gradually move into the area of
real programming, where a representation must be chosen for the problem
and where this representation determines the efficiency and elegance of
the code."

The problem: a bridge hand arrives as a list of thirteen card symbols in
the order they were dealt. Rearrange it into four sublists, one for each
suit, each in descending order, and evaluate it: its point count, with
4, 3, 2 and 1 for an ace, king, queen and jack (attribute Q2), and its
quick tricks (attribute Q0).

The representation the manual chooses:

- Each card symbol (C3 is the three of clubs, S1 the ace of spades, C13
  the king) has a description list: its **type** under Q3 (T1 for an
  ace, T13 king, T12 queen, T11 jack, T10 ten; small cards have none),
  its **card value** under Q4 (N2 to N14), and its **suit** under Q5 (C0,
  D0, F0 or S0). The types and values are integer data terms, so a type
  *is* its point count.
- The arranged hand is itself a description list: suit symbols as
  attributes, the suit's sublist as each value.
- Q3, Q4 and Q5 are also the names of routines that find those
  attributes (page 58), so that `Q3` can be executed like a function:
  "Q4(X1) has the same form as sin(30°)".

## The routines

**R4** is the executive: arrange the hand with R5, count its points with
R6, and, with the extension on page 60, its quick tricks with R7. Each
count is made local and put on the hand's description list.

**R5** (Fig. 6, page 57) is the sort. It detaches the cards from the
hand's head with J75, so that the head can become the description list of
suits while the cards are taken off the detached remainder one by one.
Each card's suit is found with J10 on Q5, its sublist with J10 on the
suit, and the card is inserted before the first card of lower value.
Notice `10 9-10` before J10 and J11: the description list of the cell
9-10, which holds the hand's name, is the hand itself. The comment on the
card says so: "must be 9-10, not list name, for J10".

**R6** (page 59) adds up the point counts.

**R7** and **R8** (pages 60 to 63) count quick tricks. These depend on
combinations of cards in a suit (AK is two tricks, AQ one and a half),
and the manual turns the table of combinations into a tree of questions
(Fig. 9): what is the first card, then the second, and so on. The tree
is held as L14, a list structure of description lists: the attributes
are card types, the values either counts or further sublists, and the
head of each sublist holds the count for "anything else". R8 walks it
with one J10 per card. Counts are in quarter tricks, "to get integer
terms": N10 is two and a half.

R8 uses a device worth knowing. `10 W0` then J10 asks for an attribute of
the *cell W0*, whose description list is whatever list W0 names: the
current sublist of the table. `60 W0` then moves W0 down the tree. The
table's lists are never named directly at all.

The data is the hand of pages 54 and 55, dealt in an order of our own
(the manual does not give one) and chosen so that the suits come out in
the manual's order, C0, D0, F0, S0, since J11 puts each new suit at the
front.

```iplv
THE BRIDGE HAND, SECTION 7              9
                                        2 C                15
                                        2 D                15
                                        2 F                15
                                        2 L                15
                                        2 N                15
                                        2 P                10
                                        2 Q                10
                                        2 R                10
                                        2 S                15
                                        2 T                15
                                        2 X                10
                                        5       00
                                          P0    10X1
THE HAND AS DEALT.                                J150
                                                10X1
                                                  R4
                                                10X1
THE HAND ARRANGED AND EVALUATED.                  J150  0
R4: REARRANGE AND ANALYZE THE HAND LIST 1
(0). WITH THE QUICK TRICKS OF PAGE 60.  1
PUSH DOWN W0.                             R4    40W0
HAND IN W0, BUT LEAVE IN H0.                    60W0
REARRANGE.                                        R5
                                                11W0
                                                40H0
COMPUTE POINT COUNT.                              R6
MAKE LOCAL (TIE IT TO HAND LIST).                 J136
                                                10Q2
PUT ON DESCRIPTION LIST WITH Q2.                  J11
                                                11W0
                                                40H0
                                                  R7
                                                  J136
                                                10Q0
                                                  J11   J30
R5: REARRANGE THE HAND LIST (0) INTO    1
ORDERED FORM. FIG. 6.                   1
HAND LIST IN 9-10, LEAVE IN H0.           R5    609-10
DETACH LIST FROM HEAD, (0).                       J75
SAVE REMAINDER LIST FOR ERASING.                40H0
FIND NEXT CARD SYMBOL TO BE SORTED.       9-5     J60
PLACE IN CELL 9-20.                             209-20
NO MORE: ERASE REMAINDER AND QUIT.              70J71
                                                129-20
                                                10Q5
FIND SUIT SYMBOL, ASSUME IT EXISTS.               J10
PLACE IN CELL 9-30.                             209-30
INPUT HAND LIST (9-10, FOR J10).                109-10
INPUT SUIT SYMBOL.                              119-30
FIND SUBLIST.                                     J10
IF NOT EXIST, GO TO CREATE IT.                  709-1
INPUT CARD SYMBOL.                              129-20
                                                10Q4
FIND CARD VALUE, ASSUME IT EXISTS.                J10
PLACE IN CELL 9-40.                             209-40
LOCATE NEXT CARD SYMBOL ON SUBLIST.       9-3     J60
                                                709-2
                                                12H0
                                                10Q4
CARD VALUE OF SYMBOL ON SUBLIST.                  J10
CARD VALUE OF NEW SYMBOL.                       119-40
H5+ IF OLD GREATER THAN NEW.                      J116
IF H5 +, GO NEXT SYMBOL ON SUBLIST.             70      9-3
NEW GREATER THAN OLD.                           129-20
INSERT ON SUBLIST AHEAD OF OLD SYMBOL.            J63   9-4
END OF LIST.                              9-2   129-20
INSERT ON SUBLIST AT END.                         J64
REMAINDER LIST, GO TO NEXT SYMBOL.        9-4   119-20  9-5
NO SUBLIST, CREATE IT.                    9-1     J90
MARK IT LOCAL.                                    J136
PUSH DOWN TO SAVE IT FOR AFTER J11.             40H0
INPUT HAND LIST.                                109-10
INVERT ORDER IN H0 FOR J11.                       J6
INPUT SUIT SYMBOL.                              119-30
ASSIGN SUBLIST, PUT SYMBOL ON IT.                 J11   9-2
LOCAL CELL: HOLDS INPUT HAND LIST.        9-10    0     0
LOCAL CELL: LOCATION IN REMAINDER.        9-20    0     0
LOCAL CELL: HOLDS SUIT SYMBOL.            9-30    0     0
LOCAL CELL: CARD VALUE BEING SORTED.      9-40    0     0
R6: TOTAL POINT COUNT OF THE ARRANGED   1
HAND (0). OUTPUT A NEW INTEGER DATA     1
TERM. PAGE 59.                          1
PRESERVE W0 AND W1.                       R6      J41
PUT HAND LIST IN W0.                            20W0
CREATE CELL FOR OUT DATA TERM.                    J90
SET IT TO 0, LEAVE IN H0.                         J124
                                          9-1   11W0
LOCATE NEXT SYMBOL.                               J60
LOCATE NEXT SYMBOL (NAMES SUBLIST).               J60
PUT BACK IN W0 BEFORE BRANCHING.                20W0
NO MORE SUBLISTS: CLEAN UP, QUIT.               70J31
INPUT NAME OF SUBLIST.                          12W0
LOCATE NEXT CARD SYMBOL.                  9-2     J60
PUT BACK IN W1 BEFORE BRANCHING.                20W1
IF NO MORE, GET NEXT SUBLIST.                   709-1
INPUT CARD SYMBOL.                              12W1
FIND TYPE (Q3 = 10Q3 J10).                        Q3
IF NOT FIND, GO TO NEXT SUBLIST.                709-1
INVERT ORDER IN H0 FOR J110.                      J6
DOUBLE UP OUTPUT DATA TERM.                     40H0
ADD POINT COUNT.                                  J110
INPUT LOCATION IN SUBLIST, CONTINUE.            11W1    9-2
R7: QUICK TRICK COUNT OF THE ARRANGED   1
HAND (0): R6 WITH THE 9-2 LOOP REPLACED,1
PAGE 61.                                1
                                          R7      J41
                                                20W0
                                                  J90
                                                  J124
                                          9-1   11W0
                                                  J60
                                                  J60
                                                20W0
                                                70J31
INPUT NAME OF SUBLIST.                          12W0
FIND COUNT FOR SUIT.                              R8
INVERT FOR J110.                                  J6
DOUBLE UP OUTPUT DATA TERM.                     40H0
ADD IN COUNT TO TOTAL.                            J110  9-1
R8: QUICK TRICK COUNT OF THE LIST OF    1
CARDS (0). PAGE 63.                     1
INPUT THE NAME OF THE TABLE.              R8    10L14
TABLE IN W0, CARD LIST IN W1.                     J51
                                          9-3   11W1
LOCATE NEXT CARD.                                 J60
PUT BACK IN W1 PRIOR TO BRANCHING.              20W1
NO MORE CARDS: "ANYTHING ELSE".                 709-1
SUBLIST OF TABLE (SYMBOL W0).                   10W0
INPUT CARD.                                     12W1
FIND TYPE.                                        Q3
                                                70      9-2
IF NO TYPE FOUND, MUST BE TYPE T0.              10T0
FIND ENTRY IN SUBLIST OF TABLE.           9-2     J10
IF NOT FIND, "ANYTHING ELSE".                   709-1
REPLACE TABLE SUBLIST WITH NEW ONE.             60W0
TEST IF LOCAL.                                    J132
IF IT IS, ANOTHER SUBLIST;                      70      9-3
IF NOT, W0 HOLDS COUNT.                         11W0    J31
COUNT FROM HEAD: "ANYTHING ELSE".         9-1   12W0    J31
Q3, Q4 AND Q5 FIND THE VALUES OF        1
ATTRIBUTES Q3, Q4 AND Q5. PAGE 58.      1
                                          Q3    10Q3    J10
                                          Q4    10Q4    J10
                                          Q5    10Q5    J10
                                        5       01
L14: THE QUICK TRICK TABLE, FIG. 9.     1
                                          L14     N0
                                                  T1
                                                  9-1
                                                  T13
                                                  9-2
                                                  T12
                                                  9-3   0
                                          9-1     N4
                                                  T13
                                                  9-4
                                                  T12
                                                  N6
                                                  T11
                                                  N5    0
                                          9-2     N0
                                                  T12
                                                  9-5
                                                  T11
                                                  9-6
                                                  T10
                                                  N2
                                                  T0
                                                  N2    0
                                          9-3     N0
                                                  T11
                                                  9-7
                                                  T10
                                                  9-8
                                                  T0
                                                  9-9   0
                                          9-4     N8
                                                  T12
                                                  N10   0
                                          9-5     N4
                                                  T11
                                                  N6    0
                                          9-6     N2
                                                  T10
                                                  N4    0
                                          9-7     N0
                                                  T10
                                                  N2
                                                  T0
                                                  N2    0
                                          9-8     N0
                                                  T0
                                                  N1    0
                                          9-9     N0
                                                  T0
                                                  N1    0
THE HAND, AS DEALT.                       X1      0
                                                  S5
                                                  F7
                                                  D4
                                                  C3
                                                  S11
                                                  C13
                                                  F11
                                                  S1
                                                  D7
                                                  C12
                                                  S3
                                                  F6
                                                  S10   0
                                          C13     9-1   0
                                          9-1     0
                                                  Q3
                                                  T13
                                                  Q4
                                                  N13
                                                  Q5
                                                  C0    0
                                          C12     9-1   0
                                          9-1     0
                                                  Q3
                                                  T12
                                                  Q4
                                                  N12
                                                  Q5
                                                  C0    0
                                          C3      9-1   0
                                          9-1     0
                                                  Q4
                                                  N3
                                                  Q5
                                                  C0    0
                                          F11     9-1   0
                                          9-1     0
                                                  Q3
                                                  T11
                                                  Q4
                                                  N11
                                                  Q5
                                                  F0    0
                                          F7      9-1   0
                                          9-1     0
                                                  Q4
                                                  N7
                                                  Q5
                                                  F0    0
                                          F6      9-1   0
                                          9-1     0
                                                  Q4
                                                  N6
                                                  Q5
                                                  F0    0
                                          D7      9-1   0
                                          9-1     0
                                                  Q4
                                                  N7
                                                  Q5
                                                  D0    0
                                          D4      9-1   0
                                          9-1     0
                                                  Q4
                                                  N4
                                                  Q5
                                                  D0    0
                                          S1      9-1   0
                                          9-1     0
                                                  Q3
                                                  T1
                                                  Q4
                                                  N14
                                                  Q5
                                                  S0    0
                                          S11     9-1   0
                                          9-1     0
                                                  Q3
                                                  T11
                                                  Q4
                                                  N11
                                                  Q5
                                                  S0    0
                                          S10     9-1   0
                                          9-1     0
                                                  Q3
                                                  T10
                                                  Q4
                                                  N10
                                                  Q5
                                                  S0    0
                                          S5      9-1   0
                                          9-1     0
                                                  Q4
                                                  N5
                                                  Q5
                                                  S0    0
                                          S3      9-1   0
                                          9-1     0
                                                  Q4
                                                  N3
                                                  Q5
                                                  S0    0
                                          T1    014
                                          T13   013
                                          T12   012
                                          T11   011
                                          T10   010
                                          N0    010
                                          N1    011
                                          N2    012
                                          N3    013
                                          N4    014
                                          N5    015
                                          N6    016
                                          N7    017
                                          N8    018
                                          N9    019
                                          N10   0110
                                          N11   0111
                                          N12   0112
                                          N13   0113
                                          N14   0114
                                        5         P0
```

```output
X1       0
         S5
         F7
         D4
         C3
         S11
         C13
         F11
         S1
         D7
         C12
         S3
         F6
         S10
X1       9-1
         C0
         9-2
         D0
         9-3
         F0
         9-4
         S0
         9-5
9-1      0
         Q0
         9-6      01 9
         Q2
         9-7      01 11
9-2      0
         C13
         C12
         C3
9-3      0
         D7
         D4
9-4      0
         F11
         F7
         F6
9-5      0
         S1
         S11
         S10
         S5
         S3
9-6      01 9
9-7      01 11
```

The arranged hand is the list structure of page 54, C13 C12 C3, D7 D4, F11
F7 F6, S1 S11 S10 S5 S3, now with a description list of its own: Q2,
the point count, 11; and Q0, the quick tricks, 9 quarter tricks, which is
two and a quarter. By hand: K Q x in clubs is 1 trick (4), A J in spades
is "1 +" (5), and the jack-high and small suits are nothing.

J150 prints the counts beside the local names that the J136s gave them:
they are part of the structure, and erasing the hand would erase them.

## R6 again, with generators

Section 8 (page 70) rewrites R6 as two generators, one inside the other:
R9 generates the local symbols on a list, which in the arranged hand are
the four suit sublists; J100 generates the cards in each. The subprocess
9-20 adds a card's type into the total and stops the generation of its
suit, with `70 0`, at the first card that has no type.

R9 itself is from section 9 (page 76). It is a generator built from a
generator: J100 produces every symbol on the list, and R9's subprocess
passes on only the local ones, with J18.

```iplv
R6 WITH GENERATORS, PAGE 70; R9, PAGE 769
                                        2 C                15
                                        2 D                15
                                        2 F                15
                                        2 L                15
                                        2 N                15
                                        2 P                10
                                        2 Q                10
                                        2 R                10
                                        2 S                15
                                        2 T                15
                                        2 X                10
                                        5       00
                                          P0    10X1
ARRANGE THE HAND,                                 R5
                                                10X1
AND COUNT ITS POINTS.                             R6
                                                  J154
                                                  J157  J155
R6: POINT COUNT OF THE ARRANGED HAND    1
(1), WITH GENERATORS. PAGE 70.          1
CREATE OUTPUT DATA TERM.                  R6      J90
PRESET IT TO 0.                                   J124
INVERT TO PUT LIST ON TOP.                        J6
ITERATE OVER SUIT SUBLISTS.                     109-10  R9
ITERATE OVER CARD SYMBOLS IN SUBLIST.     9-10  109-20
                                                  J100  J4
SUBPROCESS FOR CARD: FIND TYPE.           9-20    Q3
NOT FOUND: EXIT H5-, STOP THIS SUIT.            700
INVERT FOR J110.                                  J6
ACCUMULATE COUNT, EXIT H5+.                     40H0    J110
R9: GENERATE THE LOCAL SYMBOLS ON LIST  1
(1) FOR SUBPROCESS (0). PAGE 76.        1
                                          R9    10W0
SET UP CONTEXT.                                   J17
INPUT SUBPROCESS FOR J100.                      109-10
EXECUTE J100, QUIT WITH J19.                      J100  J19
SAVE SYMBOL IN W0.                        9-10  60W0
TEST IF LOCAL.                                    J132
IF NOT LOCAL, SET H5+ FOR NEXT ONE.             70J4
EXECUTE SUBPROCESS TO R9.                       11W0    J18
R5: REARRANGE THE HAND LIST (0) INTO    1
ORDERED FORM. FIG. 6.                   1
HAND LIST IN 9-10, LEAVE IN H0.           R5    609-10
DETACH LIST FROM HEAD, (0).                       J75
SAVE REMAINDER LIST FOR ERASING.                40H0
FIND NEXT CARD SYMBOL TO BE SORTED.       9-5     J60
PLACE IN CELL 9-20.                             209-20
NO MORE: ERASE REMAINDER AND QUIT.              70J71
                                                129-20
                                                10Q5
FIND SUIT SYMBOL, ASSUME IT EXISTS.               J10
PLACE IN CELL 9-30.                             209-30
INPUT HAND LIST (9-10, FOR J10).                109-10
INPUT SUIT SYMBOL.                              119-30
FIND SUBLIST.                                     J10
IF NOT EXIST, GO TO CREATE IT.                  709-1
INPUT CARD SYMBOL.                              129-20
                                                10Q4
FIND CARD VALUE, ASSUME IT EXISTS.                J10
PLACE IN CELL 9-40.                             209-40
LOCATE NEXT CARD SYMBOL ON SUBLIST.       9-3     J60
                                                709-2
                                                12H0
                                                10Q4
CARD VALUE OF SYMBOL ON SUBLIST.                  J10
CARD VALUE OF NEW SYMBOL.                       119-40
H5+ IF OLD GREATER THAN NEW.                      J116
IF H5 +, GO NEXT SYMBOL ON SUBLIST.             70      9-3
NEW GREATER THAN OLD.                           129-20
INSERT ON SUBLIST AHEAD OF OLD SYMBOL.            J63   9-4
END OF LIST.                              9-2   129-20
INSERT ON SUBLIST AT END.                         J64
REMAINDER LIST, GO TO NEXT SYMBOL.        9-4   119-20  9-5
NO SUBLIST, CREATE IT.                    9-1     J90
MARK IT LOCAL.                                    J136
PUSH DOWN TO SAVE IT FOR AFTER J11.             40H0
INPUT HAND LIST.                                109-10
INVERT ORDER IN H0 FOR J11.                       J6
INPUT SUIT SYMBOL.                              119-30
ASSIGN SUBLIST, PUT SYMBOL ON IT.                 J11   9-2
LOCAL CELL: HOLDS INPUT HAND LIST.        9-10    0     0
LOCAL CELL: LOCATION IN REMAINDER.        9-20    0     0
LOCAL CELL: HOLDS SUIT SYMBOL.            9-30    0     0
LOCAL CELL: CARD VALUE BEING SORTED.      9-40    0     0
                                          Q3    10Q3    J10
                                          Q4    10Q4    J10
                                          Q5    10Q5    J10
                                        5       01
THE HAND, AS DEALT.                       X1      0
                                                  S5
                                                  F7
                                                  D4
                                                  C3
                                                  S11
                                                  C13
                                                  F11
                                                  S1
                                                  D7
                                                  C12
                                                  S3
                                                  F6
                                                  S10   0
                                          C13     9-1   0
                                          9-1     0
                                                  Q3
                                                  T13
                                                  Q4
                                                  N13
                                                  Q5
                                                  C0    0
                                          C12     9-1   0
                                          9-1     0
                                                  Q3
                                                  T12
                                                  Q4
                                                  N12
                                                  Q5
                                                  C0    0
                                          C3      9-1   0
                                          9-1     0
                                                  Q4
                                                  N3
                                                  Q5
                                                  C0    0
                                          F11     9-1   0
                                          9-1     0
                                                  Q3
                                                  T11
                                                  Q4
                                                  N11
                                                  Q5
                                                  F0    0
                                          F7      9-1   0
                                          9-1     0
                                                  Q4
                                                  N7
                                                  Q5
                                                  F0    0
                                          F6      9-1   0
                                          9-1     0
                                                  Q4
                                                  N6
                                                  Q5
                                                  F0    0
                                          D7      9-1   0
                                          9-1     0
                                                  Q4
                                                  N7
                                                  Q5
                                                  D0    0
                                          D4      9-1   0
                                          9-1     0
                                                  Q4
                                                  N4
                                                  Q5
                                                  D0    0
                                          S1      9-1   0
                                          9-1     0
                                                  Q3
                                                  T1
                                                  Q4
                                                  N14
                                                  Q5
                                                  S0    0
                                          S11     9-1   0
                                          9-1     0
                                                  Q3
                                                  T11
                                                  Q4
                                                  N11
                                                  Q5
                                                  S0    0
                                          S10     9-1   0
                                          9-1     0
                                                  Q3
                                                  T10
                                                  Q4
                                                  N10
                                                  Q5
                                                  S0    0
                                          S5      9-1   0
                                          9-1     0
                                                  Q4
                                                  N5
                                                  Q5
                                                  S0    0
                                          S3      9-1   0
                                          9-1     0
                                                  Q4
                                                  N3
                                                  Q5
                                                  S0    0
                                          T1    014
                                          T13   013
                                          T12   012
                                          T11   011
                                          T10   010
                                          N0    010
                                          N1    011
                                          N2    012
                                          N3    013
                                          N4    014
                                          N5    015
                                          N6    016
                                          N7    017
                                          N8    018
                                          N9    019
                                          N10   0110
                                          N11   0111
                                          N12   0112
                                          N13   0113
                                          N14   0114
                                        5         P0
```

```output
11
```

Eleven, as before. "Note that 9-10 must set H5+ (with a J4) as a signal
to R9 to continue with the next suit. If 9-10 does not do this, the H5-
by which 9-20 signals the end of a sublist will stop both 9-10 and R9,
and no further points will be counted." The J4 is the LINK of J100 in
9-10.
