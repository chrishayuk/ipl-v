# §10 and §11. Printing and reading a line

*Part One, section 10.0, "Line Printing", pages 87 to 89 (PDF pages 122
to 124), and section 11.0, "Line Reading", pages 91 to 93 (PDF pages 126
to 128).*

## R80 to R83: printing

Section 10 builds a generator and two subprocesses for printing lists of
data terms. R80 clears the print line, generates the list for whatever
subprocess it is given, and prints the line. R81 enters each data term as
it is; R82 enters it after five blanks. Both print the line and start a
new one if the term will not fit, and both keep the term's name in a cell
of their own, 9-30, because J157 removes it from H0 whether there was
room or not.

The data shows the four kinds of data term: L15 is a sentence in
alphanumeric terms, and L16 an integer (-33), a floating point number
(-.33 x 10^-29) and an octal one (-7777777777). As the manual notes, the
blank after THIS in 9-1 is lost on entry, which is why each later word
begins with its blank.

```iplv
R80 TO R83, PAGES 88 AND 89             9
                                        2 L                20
                                        2 R               100
                                        5       00
R80: GENERATE LIST (1) FOR SUBPROCESS   1
(0).                                    1
CLEAR THE PRINT LINE 1W24.                R80     J154
GENERATE LIST (1) FOR SUBPROCESS (0).             J100
PRINT THE LINE 1W24.                              J155  0
R81: SUBPROCESS FOR ENTERING A DATA TERM1
INTO LINE 1W24 WITHOUT ANY FORMATTING.  1
SAVE NAME, IN CASE LINE IS FULL.          R81   609-30
ENTER DATA TERM LEFT-JUSTIFIED.           9-10    J157
QUIT, WITH H5+, IF THERE WAS ROOM.              70      0
OTHERWISE, PRINT FULL LINE                        J155
AND THEN CLEAR IT, RESETTING 1W25.                J154
NOW ENTER DATA TERM AT LEFT MARGIN.             119-30  J157
9-30 HOLDS NAME OF DATA TERM.             9-30 +  0     0
R82: SUBPROCESS FOR ENTERING DATA TERM  1
(0) + 5 BLANKS.                         1
SAVE NAME, IN CASE LINE IS FULL.          R82   609-30
TAB 5 COLUMNS (SAME AS BLANKS).           9-10  109-40
                                                  J161
ENTER DATA TERM LEFT-JUSTIFIED.                   J157
QUIT WITH H5+ IF THERE WAS ROOM.                70      0
OTHERWISE, PRINT FULL LINE                        J155
AND CLEAR IT, RESETTING 1W25.                     J154
GET DATA TERM, TAB 5, ENTER IT.                 119-30  9-10
9-30 HOLDS NAME OF DATA TERM.             9-30 +  0     0
9-40 IS NUMBER OF COLUMNS TO TAB.         9-40 +015
R83 (EXECUTIVE): PRINT LINE L15 AS A    1
SENTENCE, THEN PRINT LIST L16 AS A ROW  1
OF NUMBERS.                             1
INPUT DATA,                               R83   10L15
INPUT FORMAT ROUTINE,                           10R81
AND FIRE GENERATOR.                               R80
INPUT DATA,                                     10L16
INPUT FORMAT ROUTINE, AND FIRE.                 10R82   R80
                                        5       01
L15 REPRESENTS A SENTENCE AS A LIST OF  1
BCD DATA TERMS.                         1
                                          L15     0
                                                  9-1
                                                  9-2
                                                  9-3
                                                  9-4   0
                                          9-1  +21THIS
                                          9-2  +21 IS A
                                          9-3  +21 SENT
                                          9-4  +21ENCE.
L16 IS A LIST OF NUMERIC DATA TERMS.    1
                                          L16     0
                                                  9-1
                                                  9-2
                                                  9-3   0
INTEGER DATA TERM = -33.                  9-1  -0133
                                          9-2  -1133      -29
OCTAL -7777777777.                        9-3  -3177777 77777
                                        5         R83
```

```output
THIS IS A SENTENCE.
     -33     -330000-29     -7777777777
```

The floating point number is entered as the manual's section 16.2 says,
"signed value followed by signed exponent": six digits of fraction, then
the exponent. The manual is not clear about where a negative exponent's
sign goes, and the scan of that page is hard to read; here it goes
straight before the exponent.

## R95 to R97: reading

R95 (section 11) reads a card of English into a list of five-character
data terms, like L15, so that it can be printed again with J157. It
reads the card into a print line of its own, X1, reserved by a Type-3
card, and sets up the line read processes with R96: W24 names the line,
W21 and W25 name new integers set to 0, and W30 a copy of N5, the width
of the field. R97 puts all four back.

The trick is at the end of the loop. J157 drops trailing blanks, so a
word must carry its blank at the front. After each five columns R95 steps
back one (`10 M1`, `J161`) and looks at the last character with J186: if
it is a blank, the next field begins with it.

The manual's program ends with the list in H0. To see it, the deck here
starts at P0 instead of R95, which executes R95 and then prints the list
with R80 and R81 from section 10.

```iplv
R95 TO R97, PAGES 91 TO 93; R80, R81    9
                                        2 B               100
                                        2 M               100
                                        2 N               100
                                        2 P                10
                                        2 R               100
                                        2 X               100
                                        3 X1    01         80
                                        5       01
M1 = MINUS ONE.                           M1   -011
N1 = PLUS ONE.                            N1   +011
N5 = PLUS FIVE.                           N5   +015
N80 = PLUS EIGHTY.                        N80  +0180
B1 = BCD BLANK.                           B1    21
                                        5       00
R95: READ A CARD INTO LINE X1 AND CREATE1
A LIST OF LOCAL BCD DATA TERMS FROM THE 1
LINE, STARTING FROM THE LEFTMOST NON-   1
BLANK COLUMN. OUTPUT (0) NAMES THE LIST,1
AND H5 IS SET +. IF NO CARD IS PRESENT, 1
OR IF THE CARD IS BLANK, THERE IS NO    1
OUTPUT AND H5 IS SET -.                 1
                                          R95   10X1
SET UP TO READ INTO LINE X1.                      R96
READ A CARD FROM UNIT 1W18.                       J180
R97 CLEANS UP, QUITS H5- IF NO CARD.            70R97
                                                11W25
SET 1W25 TO 1ST NON-BLANK COL.                    J184
                                                30H0
QUIT IF CARD WAS BLANK.                         70R97
CREATE OUTPUT LIST (0)                            J90
AND SAVE IT IN W0.                                J50
                                          9-10  10B1
CREATE A CELL, MAKE IT INTO A                     J90
BLANK BCD DATA TERM                               J121
WITH A LOCAL NAME.                                J136
FILL DATA TERM WITH 5 CHAR. OF LINE.              J182
TO 9-200 IF DATA TERM ALL BLANKS.               709-200
                                                11W0
                                                  J6
ADD DATA TERM TO END OF OUTPUT LIST.              J65
                                                11W25
N80 IS INTEGER DATA TERM = 80.                  10N80
                                                  J116
TO 9-300 IF 1W25 = 80 OR MORE.                  70      9-300
M1 IS INTEGER DATA TERM = -1.                   10M1
1W25 POINTS TO LAST CHAR. OF TERM.                J161
                                                  J186
A FINAL BLANK STARTS THE NEXT TERM.             709-10
N1 IS INTEGER DATA TERM = 1.                    50N1
RESET 1W25 AND LOOP.                              J161  9-10
ERASE BLANK DATA TERM.                    9-200   J9
NAME OF OUTPUT LIST TO H0.                9-300 11W0
POP W0.                                           J30
CLEAN UP PRINT LINE CONTROLS, H5+.                R97   J4
R96: SET UP TO READ FROM LINE (0).      1
PRESERVE W21, W24, W25, W30. SET 1W21,  1
1W25 = 0. SET 1W24 = (0). SET 1W30 = 5. 1
                                          R96   40W24
1W24 = (0).                                     20W24
                                                40W21
                                                40W25
                                                40W30
CREATE AND                                        J90
CLEAR DATA TERM.                                  J124
1W21 = 0.                                       20W21
                                                  J90
                                                  J124
1W25 = 0.                                       20W25
                                                10N5
CREATE COPY OF N5                                 J120
FOR W30 AND                                     20W30
CLEAR PRINT LINE.                                 J154  0
R97: CLEAN UP AFTER READING. ERASE 1W21,1
1W25, 1W30. POP W21, W24, W25, W30.     1
                                          R97   11W21
                                                11W25
                                                11W30
ERASE DATA TERMS.                                 J9
                                                  J9
                                                  J9
                                                30W21
                                                30W24
                                                30W25
                                                30W30   0
P0 IS NOT IN THE MANUAL: R95, THEN PRINT1
THE LIST WITH R80 AND R81 OF SECTION 10.1
                                          P0      R95
                                                700
                                                10R81
                                                  R80   0
R80 AND R81, AS ABOVE.                  1
                                          R80     J154
                                                  J100
                                                  J155  0
                                          R81   609-30
                                                  J157
                                                70      0
                                                  J155
                                                  J154
                                                119-30  J157
                                          9-30 +  0     0
                                        5         P0
        THIS IS A SAMPLE SENTENCE FOR R95 TO READ.
```

```output
THIS IS A SAMPLE SENTENCE FOR R95 TO READ.
```

The sentence comes back whole. Not quite as the manual intends, though:
the list is built in five-column pieces that run across the words (` SAMP`,
`LE SE`, `NTENC`), and only the trailing blanks are lost, so it prints
correctly while being nothing like a list of words.

Two things here are not as the manual has them. W21 is set to 0 by R96
for reading, as a left margin; section 16 numbers print columns from 1,
so the manual seems to count from 0 here and from 1 there. And J180 here
puts the entry column back to 1 on reading a card, as the Logic Theorist
needs (see the main README); the manual's R96 has already set it to 0,
and its J184 scans from column 1W25 + 1, so it would look at column 1 and
this J184 at column 2. The sentence card is punched from column 9, as on
page 93, so either way the first non-blank is found.
