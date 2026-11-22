# Numbers, Tests and Loops

## Data terms

A cell can hold a number or a few characters instead of a symbol and a
link. Such a word is a **data term**. Its Q is 1, and its P says what kind:

| P Q | | written |
|---|---|---|
| `01` | an integer | digits, anywhere in SYMB and LINK |
| `11` | a floating point number | a fraction in SYMB, an exponent in LINK |
| `21` | five characters | in SYMB |
| `31` | an octal number | digits |

A minus sign goes in column 48, SIGN. You never handle the number itself:
you handle the *symbol that names* the data term, and processes that want
a number look in the cell that symbol names.

In a set of data, a card with `01` is the integer. In a set of routines
`01` would be an instruction, so a data term among routines is marked
with a `+` or `-` in SIGN.

## The print line

Printing a number or some text is done in three steps: clear a line, enter
things in it one after another, and print it.

- **J154** Clear print line.
- **J157** Enter data term (0) left-justified.
- **J156** Enter symbol (0) left-justified.
- **J155** Print line.

Each entry goes in at the *entry column* and moves it along. `J161`
(Increment column by (0)) leaves a gap, and `J160` (Tab to column (0))
goes to a column. Text loses the blanks at its right when it is entered,
so a space between words has to be at the left of the next word, or made
with `J161`.

## Arithmetic

- **J110** (1) + (2) into (0).
- **J111** (1) - (2) into (0).
- **J112** (1) x (2) into (0).
- **J113** (1) / (2) into (0).

(0) names the cell the answer goes in, and its name is *left in H0*, since
you nearly always want the answer next. The operands are not changed,
unless one of them is also the result.

```iplv
SEVEN AND THIRTY-FIVE                   9
                                        2 K                10
                                        2 P                10
                                        5       00
(2) WILL BE K2,                           P0    10K2
(1) WILL BE K1,                                 10K1
AND (0) THE ANSWER'S CELL.                      10K3
K1 + K2 INTO K3. K3 IS LEFT IN H0.                J110
                                                  J154
ENTER K3, REMOVING IT.                            J157
                                                  J155  0
                                        5       01
                                          K1    017
                                          K2    0135
                                          K3    010
                                        5         P0
```

```output
42
```

Inputs go in backwards: the last one you input is (0).

## H5, the test cell

Some processes are tests. A test puts `+` or `-` in one cell, `H5`, and
does nothing else. Then P = 7 acts on it: **branch to S if H5 is -**. If
H5 is `+` the interpreter goes on to the LINK as usual.

- **J2** TEST (0) = (1), for any two symbols.
- **J114** TEST (0) = (1), for data terms.
- **J115** TEST (0) > (1). **J116** TEST (0) < (1).
- **J117** TEST (0) = 0. **J118** TEST (0) > 0. **J119** TEST (0) < 0.
- **J3** sets H5 `-`, **J4** sets it `+`, and **J5** reverses it.

Tests remove their inputs. Other processes leave H5 alone, so a test and
the branch that uses it need not be next to each other.

```iplv
IS K1 GREATER THAN K2                   9
                                        2 K                10
                                        2 P                10
                                        2 T                10
                                        5       00
                                          P0      J154
                                                10K2
                                                10K1
TEST K1 > K2.                                     J115
IF NOT, GO TO 9-1.                              709-1
IT IS.                                          10T1
                                                  J157  J155
IT IS NOT.                                9-1   10T2
                                                  J157  J155
                                        5       01
                                          K1    017
                                          K2    0135
                                          T1    21YES
                                          T2    21NO
                                        5         P0
```

```output
NO
```

Each branch ends by entering its word with a LINK of `J155`, which prints
the line and ends the routine.

## A loop

A loop is a branch backwards. This adds the integers from 10 down to 1.
`J125` and `J111` change a number where it lies, so `N0` counts down.

```iplv
THE SUM OF 1 TO 10                      9
                                        2 K                10
                                        2 N                10
                                        2 P                10
                                        2 T                10
                                        5       00
                                          P0    10N0
                                                10T0
                                                10T0
T0 + N0 INTO T0.                                  J110
REMOVE THE NAME T0.                             30H0
                                                10K1
                                                10N0
                                                10N0
N0 - 1 INTO N0. N0 IS LEFT IN H0,                 J111
AND IS TESTED: IS IT 0.                           J117
IF NOT, ROUND AGAIN.                            70P0
                                                  J154
                                                10T0
                                                  J157  J155
                                        5       01
                                          K1    011
                                          N0    0110
                                          T0    010
                                        5         P0
```

```output
55
```

`70 P0` branches to the head of the routine. A branch stays inside the
routine: it is not a call, and there is nothing to come back from.

## A table

`J159` enters a number right-justified, with its last digit at the entry
column, which is what a column of figures needs. `J160` counts from the
left margin, which is column 1, so a tab of 8 puts the entry column at 9.

```iplv
SQUARES                                 9
                                        2 I                10
                                        2 K                10
                                        2 P                10
                                        2 X                10
                                        5       00
                                          P0      J154
                                                10I0
ENTER I0 AT THE LEFT.                             J157
                                                10K8
TAB 8 COLUMNS FROM THE MARGIN.                    J160
                                                10I0
                                                10I0
                                                10X0
I0 X I0 INTO X0,                                  J112
ENTERED TO END AT THAT COLUMN.                    J159
                                                  J155
                                                10K6
                                                10I0
TALLY 1 IN I0.                                    J125
TEST I0 = 6.                                      J114
IF NOT, AGAIN; IF SO, END.                      70P0    0
                                        5       01
                                          I0    011
                                          K6    016
                                          K8    018
                                          X0    010
                                        5         P0
```

```output
1       1
2       4
3       9
4      16
5      25
```

The last card is worth a second look. If H5 is `-` it branches to P0. If
H5 is `+` it goes on to its LINK, which is `0`, and the routine ends.
