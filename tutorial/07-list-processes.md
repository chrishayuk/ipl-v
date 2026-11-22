# List Processes

The processes J60 to J104 work on lists. They are the heart of the
language, and this chapter uses the ones you will need every day.

## Walking a list

**J60** LOCATE next symbol after cell (0). Its input is the name of a
cell. If there is a next cell, its name replaces (0) and H5 is `+`. If
there is none, (0) is left alone and H5 is `-`.

Note what it gives you: the name of the *cell*, not the symbol in it. To
get the symbol, use Q = 2 on H0 itself. `12 H0` is "input the symbol in
the cell named in H0", which pushes the symbol on top of the cell's name.
You will write `12 H0` often.

```iplv
PRINT EACH SYMBOL OF A LIST             9
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        5       00
(0) IS THE HEAD OF L1.                    P0    10L1
LOCATE THE NEXT CELL.                     9-1     J60
NONE: FINISH.                                   709-2
INPUT THE SYMBOL IN THAT CELL,                  12H0
PRINT IT, AND GO ROUND.                           J152  9-1
REMOVE THE NAME OF THE LAST CELL.         9-2   30H0    0
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

Starting from the head, the first `J60` gives the first list cell, which
is why the head's own symbol is never printed. At the end (0) is still the
name of the last cell, and the routine removes it before it ends: H0 must
be left as it was found.

## Making and changing a list

- **J90** Create an empty list: a new head. Its name is the output.
- **J91** to **J99** create a list of the 1 to 9 symbols in H0.
- **J64** INSERT (0) after symbol in cell (1).
- **J63** INSERT (0) before symbol in cell (1).
- **J65** INSERT (0) at end of list (1).
- **J66** the same, if (0) is not on the list already.
- **J68** DELETE symbol in cell (0).
- **J69** DELETE (0) from list (1).
- **J71** ERASE list (0): every cell goes back to available space.

The name of a list made by `J90` is an internal symbol, a number, since
that is all the machine has to call it. You keep it in a cell.

```iplv
MAKING A LIST                           9
                                        2 P                10
                                        2 S                10
                                        5       00
CREATE A LIST.                            P0      J90
W0 HOLDS ITS NAME.                                J50
                                                11W0
                                                10S1
INSERT S1 AT THE END OF LIST 1W0.                 J65
                                                11W0
                                                10S3
AND S3.                                           J65
                                                11W0
                                                10S0
INSERT S0 AFTER THE HEAD: AT THE FRONT.           J64
                                                11W0
                                                  J151
                                                11W0
                                                10S1
DELETE S1 FROM IT.                                J69
                                                11W0
                                                  J151
                                                11W0
ERASE IT.                                         J71
                                                  J30   0
                                        5         P0
```

```output
104      0
         S0
         S1
         S3
104      0
         S0
         S3
```

The number beside the head is the list's name: the address of its head.
Yours will be the same, because the machine always takes cells from
available space in the same order.

`J64` with the head as its cell puts a symbol at the front, because
"after the head" is the first place on the list.

## Finding things

- **J62** LOCATE (0) on list (1): the cell that holds it.
- **J77** TEST if (0) is on list (1).
- **J78** TEST if list (0) is not empty.
- **J81** to **J89** FIND the first to ninth symbol on list (0).
- **J61** LOCATE last symbol on list (0).
- **J126** Count list (0): the output is a new integer.

A process described as FIND or LOCATE may fail, and says so in H5. A FIND
that fails leaves no output in H0; a LOCATE leaves the last cell.

```iplv
FINDING THINGS ON A LIST                9
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        5       00
                                          P0    10L1
FIND THE SECOND SYMBOL.                           J82
                                                  J152
                                                10L1
COUNT THE LIST.                                   J126
KEEP THE COUNT'S NAME,                          60W0
PRINT IT,                                         J152
                                                11W0
AND ERASE ITS CELL.                               J9
                                                10L1
                                                10S9
TEST IF S9 IS ON L1.                              J77
                                                709-1
IT IS: SAY SO.                                  10S9
                                                  J152  0
IT IS NOT: PUT IT THERE.                  9-1   10L1
                                                10S9
                                                  J65
                                                10L1
                                                  J151  0
                                        5       01
                                          L1      0
                                                  S1
                                                  S2
                                                  S3    0
                                        5         P0
```

```output
         S2
         116      01 3
L1       0
         S1
         S2
         S3
         S9
```

`J152` printing a symbol that names a data term prints the data term
beside it. `J126` made a new cell for its count, and a cell you were
given is yours to give back: `J9` erases one cell.

## Erasing is your job

There is no garbage collector. Available space is one list, and a cell
is on it or in use. A list you stop using without erasing stays in use
until the run ends. `J126` will count the available space if you give it
`H2`, the name of that list, and chapter 10 uses that to show cells
going and coming back.
