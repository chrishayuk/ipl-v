# Cards

A deck is a pile of 80-column cards, and every card has the same layout.
Nothing is free-form: a symbol means what it means because of the columns
it is punched in.

| columns | field | holds |
|---|---|---|
| 1-40 | | comment: anything |
| 41 | TYPE | blank for a word of the program; a digit for a card that says something to the loader |
| 43-47 | NAME | the symbol that names this card's cell |
| 48 | SIGN | `+` or `-` (chapter 6) |
| 49 | P | a digit, 0 to 7 |
| 50 | Q | a digit, 0 to 7 |
| 51-55 | SYMB | a symbol |
| 57-61 | LINK | a symbol: the next cell |
| 63-80 | | comment, and the card's number in the deck |

Most fields of most cards are blank. Here is the first program again, with
a ruler over it:

```text
comments                                T NAME ±PQSYMB  LINK
```

```iplv
HELLO, WORLD                            9
                                        2 P                10
                                        2 T                10
                                        5       00
CLEAR THE PRINT LINE.                     P0      J154
                                                10T0
ENTER HELLO IN IT.                                J157
                                                10T1
                                                  J157
                                                10T2
                                                  J157
PRINT THE LINE.                                   J155  0
                                        5       01
                                          T0    21HELLO
                                          T1    21 WORL
                                          T2    21D
                                        5         P0
```

```output
HELLO WORLD
```

## The cards that are not words

The digit in column 41 says what kind of card it is.

**9** is the first card of a deck. The rest of it is comment.

**2** is a region card. A *regional symbol* is a letter or a punctuation
mark followed by a number: `P0`, `T2`, `*17`. All the symbols with one
letter are a *region*, and a region card says how many there are: `P` with
10 in LINK gives `P0` to `P9`. Each is the name of one cell, and the cells
of a region lie side by side in storage. The regions `H`, `J` and `W`
belong to the system and are already there.

**5** is a header card. What follows it is a *set* of routines if Q is 0
or blank, and a set of data if Q is 1. The loader has to be told, because
it treats P and Q differently in the two.

**5 with a symbol in SYMB** is the start card. It ends loading, and the
routine it names is executed. Here that is `P0`.

**1** is a comment card. Everything on it but column 41 is comment.

## The words

The eight cards after the first header are a routine named `P0`. Only the
first has a NAME. The others need none, because each card's LINK is blank,
and *a blank LINK means the next card*. The eight cells form a list, in
the order of the cards, and the `0` in the last LINK ends it.

Read down the SYMB column and you can nearly follow it. `J154`, `J157`
and `J155` are *basic processes*, the built-in operations: clear the print
line, enter a piece of text in it, print it. The cards with `10` in P and
Q each put a symbol where the next process will find it. Chapter 4 says
exactly how.

The three cards after the second header are data. `21` in P and Q makes a
card a *data term* holding five characters, which are in SYMB. So `T0`
names a cell holding `HELLO`. Five characters is all a data term holds,
which is why HELLO WORLD is three of them.

## Checking a deck

A symbol one column out of place is a different card, and nothing will
tell you so when it runs. `iplv check` reads a deck without running it:

```
target/release/iplv check examples/hello.iplv
```

It prints nothing when it finds nothing. It knows when a comment has run
into column 41, when P or Q is not a digit from 0 to 7, when a routine is
executed that no card defines, and a dozen more such things.

In Visual Studio Code, the extension in `editors/vscode` draws a line at
the edge of each field and makes Tab go from one field to the next, which
is what the Tab key was for.

## What goes wrong

A deck with no header card:

```iplv
                                          P0      J155  0
                                        5         P0
```

```output
! card 1: a word before any header card (TYPE 5)
```

A routine that executes a symbol nothing defines. `Q7` is a good symbol,
so the deck loads, and the machine stops when it gets there:

```iplv
                                        5       00
                                          P0      J154
                                                  Q7    0
                                        5         P0
```

```output
! Q7 is executed but was never defined
```
