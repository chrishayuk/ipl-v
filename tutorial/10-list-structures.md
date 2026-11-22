# List Structures

A list whose members are lists is where IPL-V stops being a language of
sequences and becomes one of trees and networks. Nothing new is needed:
a cell's SYMB can be the name of another list.

## Sublists and local symbols

A **list structure** is a list with a regional name, the *main list*,
together with every list whose *local* name occurs on it, and on those,
and so on. The local symbols are what hold it together: `9-1` in a cell
means "the sublist `9-1` of this structure". A regional symbol in a cell
is just a symbol, and what it names is no part of the structure.

This is the expression (X1 + X2 * (X3 - X4)) from section 2.10 of the
manual, with a sublist for each parenthesis. `J150` prints a list
structure: each list in turn, with the local names turned into small
numbers.

```iplv
AN EXPRESSION AS A LIST STRUCTURE       9
                                        2 P                10
                                        2 X                10
                                        2 +                 1
                                        2 -                 1
                                        2 *                 1
                                        5       00
                                          P0    10X0
PRINT LIST STRUCTURE (0).                         J150  0
                                        5       01
                                          X0      0
                                                  9-1   0
                                          9-1     0
                                                  X1
                                                  +0
                                                  9-2   0
                                          9-2     0
                                                  X2
                                                  *0
                                                  9-3   0
                                          9-3     0
                                                  X3
                                                  -0
                                                  X4    0
                                        5         P0
```

```output
X0       0
         9-1
9-1      0
         X1
         +0
         9-2
9-2      0
         X2
         *0
         9-3
9-3      0
         X3
         -0
         X4
```

`+0`, `-0` and `*0` are symbols like any others: the regions `+`, `-`
and `*` have one cell each.

That a symbol is local is not just how it was written. The machine
remembers it, in the Q of every cell that holds the symbol, and the
processes for whole structures follow local symbols and no others.

## Copying and erasing a structure

- **J74** COPY list structure (0). The output names a new structure with
  new cells and new sublists, and the same regional symbols.
- **J72** ERASE list structure (0): the list, its sublists, theirs.
- **J73** and **J71** do the same for one list, and follow nothing.

Erasing matters, and you can watch it. **J126** counts a list, and the
available space is a list, named `H2`. This program counts it, copies
`X0`, counts again, erases the copy, and counts again.

```iplv
CELLS TAKEN AND GIVEN BACK              9
                                        2 K                10
                                        2 P                10
                                        2 X                10
                                        2 +                 1
                                        2 -                 1
                                        2 *                 1
                                        5       00
COUNT THE AVAILABLE SPACE.                P0      P2
                                                10K2
                                                10K1
K1 = K2: THE COUNT AT THE START.                  J121
                                                30H0
                                                10X0
COPY THE LIST STRUCTURE.                          J74
K3 HOLDS THE NAME OF THE COPY.                  20K3
                                                  P2
HOW MANY CELLS DID IT TAKE.                       P1
                                                11K3
ERASE THE COPY.                                   J72
                                                  P2
HOW MANY ARE IN USE NOW.                          P1    0
P1 PRINTS K1 LESS K2.                   1
                                          P1    10K2
                                                10K1
                                                10K4
K1 - K2 INTO K4.                                  J111
                                                  J154
                                                  J157  J155
P2 COUNTS THE CELLS ON H2 INTO K2.      1
                                          P2    10H2
A NEW CELL HOLDS THE COUNT.                       J126
                                                40H0
                                                10K2
SET K2 IDENTICAL TO IT,                           J121
                                                30H0
AND ERASE IT.                                     J9    0
                                        5       01
                                          K1    010
                                          K2    010
                                          K3      0     0
                                          K4    010
                                          X0      0
                                                  9-1   0
                                          9-1     0
                                                  X1
                                                  +0
                                                  9-2   0
                                          9-2     0
                                                  X2
                                                  *0
                                                  9-3   0
                                          9-3     0
                                                  X3
                                                  -0
                                                  X4    0
                                        5         P0
```

```output
14
0
```

Fourteen cells for four lists of fourteen cells, and none after the
erase. This machine had 22,000 cells for everything, program included,
and a program that forgot to erase ran out.

## Making structures as you go

A list made by `J90` has an internal name. To make it a sublist of
something, make its name local with **J136** (Make (0) local) before you
insert it. From then on it will be copied and erased with the structure
it is on. A description list made by `J11` is given a local name for the
same reason: erase the list as a structure and its description goes too.

## Deleting the last cell

One odd thing is worth knowing, because you will meet it in traces.
`J68` deletes the symbol in a cell by moving the next cell's word up and
freeing the next cell. For the last cell of a list there is no next
cell, and nothing says which cell links to this one, so the cell cannot
be removed. Instead its word is made all zeros, and it becomes a *private
termination cell*: a second way of saying "the list ends here". `J60`
tidies them away when it meets one.

## What to look at

Open the page in `web/` (chapter 12), pick the deck `structure`, type
`X0` in the box above the drawing, and step. The copy appears as new
cells in the picture of storage, and goes back to available space cell by
cell.
