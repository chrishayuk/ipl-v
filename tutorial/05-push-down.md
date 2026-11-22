# Preserve and Restore

Every storage cell is the top of a stack. That is the idea in IPL-V most
worth understanding, and it costs nothing: it falls out of the LINK that a
storage cell was not using.

## Preserve

To **preserve** a cell is to take a new cell from available space, copy
the word into it, and link it in just after the cell. The cell still holds
its symbol, and now there is a copy of it underneath. To **restore** is
the reverse: the word of the cell below moves up, and its cell goes back
to available space.

P = 4 preserves S and P = 3 restores it. `J151` prints a cell and the
list under it, so we can watch:

```iplv
PRESERVE AND RESTORE                    9
                                        2 B                10
                                        2 G                10
                                        2 P                10
                                        2 S                10
                                        2 Y                10
                                        5       00
Y3 CONTAINS S5.                           P0    10Y3
                                                  J151
PRESERVE Y3.                                    40Y3
                                                10B1
PUT B1 INTO Y3.                                 20Y3
                                                10Y3
                                                  J151
PRESERVE IT AGAIN.                              40Y3
                                                10G3
PUT G3 INTO Y3.                                 20Y3
                                                10Y3
                                                  J151
RESTORE Y3.                                     30Y3
                                                10Y3
                                                  J151
RESTORE IT AGAIN.                               30Y3
                                                10Y3
Y3 IS AS IT BEGAN.                                J151  0
                                        5       01
                                          Y3      S5    0
                                        5         P0
```

```output
Y3       S5
Y3       B1
         S5
Y3       G3
         B1
         S5
Y3       B1
         S5
Y3       S5
```

Two preserves and two restores leave the cell as it was. A routine that
wants to use a cell, and does not know whether its caller is using it,
preserves it first and restores it last. The manual calls a cell *safe*
over a routine that treats it so.

## H0 is one of these

"Input S" was described as pushing H0 down and putting S in it. Now you
can see what that is: preserve H0, then put S in H0. And "Output to S"
ends with restore H0. The stack of inputs is H0's push-down list, built
from cells of available space like every other.

So `40 H0` duplicates (0), which is useful before a process that will
consume it:

```iplv
DUPLICATING (0)                         9
                                        2 A                10
                                        2 P                10
                                        5       00
                                          P0    10A1
PRESERVE H0: (0) AND (1) ARE BOTH A1.           40H0
PRINT ONE AND REMOVE IT.                          J152
THE OTHER IS STILL THERE.                         J152  0
                                        5         P0
```

```output
         A1
         A1
```

And `30 H0` throws (0) away. `J8` does the same; it exists so that
"restore H0" can be written in a LINK.

## The working cells

A routine usually needs a few cells of its own. Ten are provided for all
routines to share, `W0` to `W9`, with processes to handle them in blocks.
For a routine that needs three:

- **J42** preserves W0, W1 and W2.
- **J22** moves (0), (1), (2) into W0, W1, W2, and removes them from H0.
- **J52** does both: preserve, then move.
- **J32** restores W0, W1 and W2.

The last digit is the number of the highest cell, so there are ten of
each: `J50` handles W0 alone and `J59` all ten.

The shape of nearly every routine that takes inputs is therefore

```iplv
THREE INPUTS, INTO W0, W1 AND W2.         R1      J52
... THE WORK, USING 1W0, 1W1, 1W2 ...           11W0
RESTORE THE WORKING CELLS, AND END.               J32   0
```

Because the cells are preserved on the way in and restored on the way
out, a routine can execute any other routine, or itself, and find its own
symbols in the W's afterwards.

```iplv
THE WORKING CELLS                       9
                                        2 A                10
                                        2 P                10
                                        5       00
                                          P0    10A2
(0) IS A1, (1) IS A2.                           10A1
PRESERVE W0 AND W1, AND MOVE THEM IN.             J51
                                                11W1
PRINT 1W1.                                        J152
                                                11W0
PRINT 1W0.                                        J152
RESTORE W0 AND W1.                                J31   0
                                        5         P0
```

```output
         A2
         A1
```
