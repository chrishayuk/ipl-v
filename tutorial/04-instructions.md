# Instructions

A routine is a list whose words are instructions. In an instruction the
four parts of the word mean this:

- `SYMB` is a symbol to work on.
- `Q` says how to get from SYMB to the symbol actually meant, which the
  manual calls **S**, the *designated symbol*.
- `P` says what to do with S.
- `LINK` is the next instruction, as in any list.

## H0, the communication cell

Routines and basic processes have no argument lists. Whatever a process
needs, it finds in one cell, `H0`, and whatever it produces it leaves
there. H0 holds one symbol, but it is the top of a stack: putting a symbol
in pushes down what was there, and taking it out brings the old one back.
The manual writes **(0)** for the symbol in H0, **(1)** for the one under
it, **(2)** for the one under that.

## P: the eight operations

| P | | |
|---|---|---|
| 0 | Execute S | S is a routine or a basic process |
| 1 | Input S | push H0 down, and put S in it |
| 2 | Output to S | put (0) in cell S, and pop H0 up |
| 3 | Restore S | pop cell S up |
| 4 | Preserve S | push cell S down |
| 5 | Replace (0) by S | put S in H0 without pushing |
| 6 | Copy (0) in S | put (0) in cell S and leave H0 alone |
| 7 | Branch to S if H5 is - | chapter 6 |

P = 1 and P = 2 are the two you will write most: one carries a symbol
into H0 and the other carries it out.

## Q: which symbol

| Q | S is |
|---|---|
| 0 | SYMB itself |
| 1 | the symbol in the cell SYMB names |
| 2 | the symbol in the cell that *that* symbol names |

So `10 X1` inputs the symbol `X1`, and `11 X1` inputs whatever is in the
cell X1. The manual writes `1X1` for "the symbol in X1", and so will this
tutorial.

## Moving a symbol

```iplv
MOVING A SYMBOL FROM X1 TO X2           9
                                        2 P                10
                                        2 S                10
                                        2 X                10
                                        5       00
INPUT THE SYMBOL IN X1.                   P0    11X1
OUTPUT IT TO X2.                                20X2
                                                10X2
PRINT X2.                                         J151  0
                                        5       01
                                          X1      S5    0
                                          X2      0     0
                                        5         P0
```

```output
X2       S5
```

`11 X1` puts S5 in H0. `20 X2` puts (0) in X2 and pops H0, which leaves
H0 as it was before the two instructions. That is the rule every routine
follows: take your inputs out of H0, leave only your outputs.

Now with Q = 2. `X3` holds the name of the cell to read:

```iplv
THE SYMBOL IN THE CELL NAMED IN X3      9
                                        2 P                10
                                        2 S                10
                                        2 X                10
                                        5       00
INPUT THE SYMBOL IN THE CELL 1X3.         P0    12X3
PRINT SYMBOL (0).                                 J152  0
                                        5       01
                                          X1      S5    0
                                          X3      X1    0
                                        5         P0
```

```output
         S5
```

## Executing a process

P = 0 executes S. If S is a basic process it is done; if S is a routine,
the interpreter goes to it and comes back when its list ends. Processes
find their inputs in H0 in an order their definitions give. `J6` reverses
(0) and (1):

```iplv
(0) AND (1)                             9
                                        2 A                10
                                        2 P                10
                                        5       00
(0) IS A1.                                P0    10A1
(0) IS A2, AND (1) IS A1.                       10A2
REVERSE THEM.                                     J6
PRINT (0), AND REMOVE IT.                         J152
PRINT WHAT WAS (1).                               J152  0
                                        5         P0
```

```output
         A1
         A2
```

`J152` prints the symbol (0) and removes it from H0, as a well-behaved
process does with an input.

The basic processes are named `J0` to `J202`. The reference chapter lists
them. The definitions in the manual all read like these:

- **J6** Reverse (0) and (1).
- **J8** Restore H0.
- **J60** LOCATE next symbol after cell (0).
- **J65** INSERT (0) at end of list (1).
- **J110** (1) + (2) into (0).

Read them with H0 in mind: the inputs named are consumed, and nothing is
left in H0 unless the definition says it is.

## How a routine ends

When the interpreter reaches a LINK of `0`, the routine is over, and it
goes back to whoever executed it. If nobody did, the machine halts.

A LINK may also name a routine or a basic process. Then that is executed
as the last thing the list does. The last two cards of the first program,

```iplv
                                                  J157
                                                  J155  0
```

can be written as one:

```iplv
                                                  J157  J155
```
