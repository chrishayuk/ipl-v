# Generators

Chapter 7 walked a list with a loop: locate the next cell, test, get the
symbol, do something, go round. That loop mixes two things, how to go
through a list and what to do with each member. A **generator** separates
them. It produces the members one at a time and, for each, executes a
routine you give it, the **subprocess**.

## J100

**J100** Generate symbols from list (1) for subprocess (0). It executes
the subprocess once for each symbol on the list, with the symbol as (0).

The subprocess must remove that input, and must say in H5 whether to go
on: `+` for the next symbol, `-` to stop.

```iplv
EACH SYMBOL OF A LIST, WITH A GENERATOR 9
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        5       00
                                          P0    10L1
THE SUBPROCESS IS 9-1.                          109-1
                                                  J100  0
PRINT THE SYMBOL, WHICH REMOVES IT.       9-1     J152
SET H5 +: GO ON.                                  J4    0
                                        5       01
                                          L1      0
                                                  S1
                                                  S2
                                                  S3    0
                                        5         P0
```

```output
         S1
         S2
         S3
```

The subprocess is a local subroutine of the routine that uses the
generator. That is the usual way to write one.

## Stopping early

When the generator ends, H5 says how: `+` if it went through the whole
list, `-` if the subprocess stopped it. That makes a generator a search.
*Is there a number on this list?*

```iplv
THE FIRST DATA TERM ON A LIST           9
                                        2 K                10
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        2 T                10
                                        5       00
                                          P0      J154
                                                10L1
                                                109-1
                                                  J100
H5 -: THE SUBPROCESS STOPPED IT.                709-2
                                                10T2
                                                  J157  J155
                                          9-2   10T1
                                                  J157  J155
TEST IF (0) NAMES A DATA TERM.            9-1     J131
REVERSE H5: FOUND MEANS STOP.                     J5    0
                                        5       01
                                          L1      0
                                                  S1
                                                  K7
                                                  S3    0
                                          K7    017
                                          T1    21FOUND
                                          T2    21NONE
                                        5         P0
```

```output
FOUND
```

## The subprocess shares your working cells

A subprocess is part of the routine that uses the generator, and it sees
that routine's W cells, though the generator runs in between. That is how
it gets at anything besides its one input, and how it hands results back.

This reverses a list. The routine keeps the new list's name in W0, and
the subprocess puts each symbol at the front of list 1W0.

```iplv
REVERSING A LIST                        9
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        5       00
CREATE A LIST,                            P0      J90
AND KEEP ITS NAME IN W0.                          J50
                                                10L1
                                                109-1
                                                  J100
                                                11W0
                                                  J151
                                                11W0
                                                  J71
                                                  J30   0
(0) IS THE NEW LIST, (1) THE SYMBOL.      9-1   11W0
                                                  J6
INSERT THE SYMBOL AFTER THE HEAD.                 J64
                                                  J4    0
                                        5       01
                                          L1      0
                                                  S1
                                                  S2
                                                  S3    0
                                        5         P0
```

```output
110      0
         S3
         S2
         S1
```

## Writing a generator

A generator of your own has a problem to solve. It wants working cells
too, and the subprocess, which runs in the middle of it, must find the
*caller's* symbols in the W cells, not the generator's. Three processes
do the swapping:

- **J17** Generator set-up. (0) is the highest W cell the generator will
  use, and (1) is the subprocess. It preserves those W cells.
- **J18** Execute subprocess. It takes the generator's symbols out of the
  W cells, executes the subprocess with the caller's there instead, and
  puts the generator's back.
- **J19** Generator clean-up. It restores the W cells and sets H5 to
  what the last subprocess said.

*G1 generates the integers from 1 to (1) for subprocess (0).* Both G1 and
the routine that uses it keep something in W0, to show that neither
disturbs the other.

```iplv
A GENERATOR OF INTEGERS                 9
                                        2 G                10
                                        2 K                10
                                        2 N                10
                                        2 P                10
                                        2 T                10
                                        2 X                10
                                        5       00
                                          P0    10T0
W0 NAMES THE TOTAL.                               J50
UP TO N0,                                       10N0
FOR THE SUBPROCESS 9-1.                         109-1
                                                  G1
                                                  J154
                                                11W0
                                                  J157
                                                  J155
                                                  J30   0
THE SUBPROCESS: (0) IS THE INTEGER.       9-1   40H0
                                                10X0
ITS SQUARE, INTO X0.                              J112
W0 IS THE CALLER'S W0 HERE.                     11W0
                                                11W0
ADD IT TO THE TOTAL.                              J110
                                                30H0
H5 +: THE NEXT INTEGER.                           J4    0
G1: GENERATE THE INTEGERS 1 TO (1) FOR  1
SUBPROCESS (0).                         1
IT USES W0 AND W1.                        G1    10W1
J17 TAKES W1 AND THE SUBPROCESS.                  J17
W0 NAMES THE LIMIT.                             20W0
                                                10K0
                                                  J120
W1 NAMES A COUNTER, A COPY OF 0.                20W1
                                          9-1   11W0
                                                11W1
TALLY 1 IN THE COUNTER.                           J125
TEST COUNTER > LIMIT.                             J115
IF NOT, 9-2.                                    709-2
                                          9-3   11W1
ERASE THE COUNTER,                                J9
AND CLEAN UP.                                     J19   0
                                          9-2   11W1
EXECUTE THE SUBPROCESS ON THE COUNTER.            J18
STOP IF IT SAID -, ELSE GO ROUND.               709-3   9-1
                                        5       01
                                          K0    010
                                          N0    0110
                                          T0    010
                                          X0    010
                                        5         P0
```

```output
385
```

The sum of the squares from 1 to 10 is 385.

The same pattern gives a generator of anything: the moves in a chess
position, the theorems that match an expression, the lines of a file.
The Logic Theorist is, to a first approximation, generators of things to
try and subprocesses that try them.

## The other two

**J101** generates the *cells* of a whole list structure, and **J102**
those of a tree, for a subprocess that wants to visit every cell.
