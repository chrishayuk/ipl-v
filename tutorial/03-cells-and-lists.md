# Cells and Lists

## Words, cells, symbols

Storage is cells, and each cell holds one word:

```text
P  Q  SYMB  LINK
```

`P` and `Q` are single digits, the *prefixes*. `SYMB` and `LINK` are
*symbols*. A symbol is the name of a cell, and inside the machine it is
simply that cell's address. So a word can point at two other cells. That
is all the structure there is, and everything in IPL-V is built from it.

There are three kinds of symbol, and they differ only in how you write
them and in how far their meaning reaches:

- **Regional**: a letter or mark and a number, `L1`, `S3`, `+0`. It means
  the same cell everywhere in the deck. These are your names.
- **Local**: `9-` and a number, `9-1`, `9-27`. It means something only
  within one *list structure*, which for now is the cards from one
  regional NAME to the next. Every routine can have its own `9-1`.
- **Internal**: a plain number. These are the machine's own names for
  cells, and you will see them printed but rarely write one, with one
  exception: `0` is the **termination symbol**. It names no cell and means
  "nothing more".

## A list

A list is cells joined by their LINKs. The first cell is the **head**, and
the name of the head is the name of the list. The last cell has `0` for
its LINK.

```iplv
A LIST                                  9
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        5       00
                                          P0    10L1
PRINT LIST (0).                                   J151  0
                                        5       01
THE HEAD OF L1.                           L1      0
                                                  S1
                                                  S2
THE LAST CELL.                                    S3    0
                                        5         P0
```

```output
L1       0
         S1
         S2
         S3
```

`J151` prints a list, one cell to a line: the list's name beside the head,
then the symbol in each cell. Four cards made four cells. `L1` named the
first because its card said so; the loader took the other three from
available space and filled in each blank LINK with the address of the
cell of the next card.

The head holds `0`, not a symbol of the list. That is a convention, and
nearly every basic process assumes it: the head's SYMB is kept for
something else (chapter 9), and the symbols *on* the list are in the cells
after the head. A list whose head is kept free like this is called
*describable*. Make all your lists describable.

`S1`, `S2` and `S3` are just symbols. No card defines them, and none
needs to: a symbol can stand for itself. Their cells exist, in region `S`,
and are empty.

## Naming a cell in the middle

Cards need not come in the order of the list. Give a cell a local name and
any LINK can point at it:

```iplv
THE SAME LIST, SHUFFLED                 9
                                        2 L                10
                                        2 P                10
                                        2 S                10
                                        5       00
                                          P0    10L1
                                                  J151  0
                                        5       01
THE HEAD LINKS TO 9-1,                    L1      0     9-1
                                          9-2     S2    9-3
WHICH LINKS TO 9-2,                       9-1     S1    9-2
                                          9-3     S3    0
                                        5         P0
```

```output
L1       0
         S1
         S2
         S3
```

A routine is a list too, and this is how one jumps about in it.

## A blank SYMB

A blank LINK means the next card. So does a blank SYMB. That is how a list
gets a list as one of its members, written straight after it:

```iplv
                                          L1      0
                                                  S1
THIS CELL'S SYMBOL IS THE CELL BELOW.                   0
THE HEAD OF THE INNER LIST.                       0
                                                  S2    0
```

Chapter 10 does this properly. If the next card has a regional NAME, it
begins something new, and a blank SYMB or LINK before it means `0`.

## A storage cell

A cell with a regional name and `0` for its LINK is a place to keep one
symbol:

```iplv
X1 HOLDS S5.                              X1      S5    0
X2 IS EMPTY.                              X2      0     0
```

That is the nearest thing IPL-V has to a variable. The next chapter moves
symbols in and out of such cells, and the one after shows what the LINK,
here `0`, is for.
