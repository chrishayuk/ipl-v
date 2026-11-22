# Reference

## The card

| columns | field | |
|---|---|---|
| 1-40 | | comment |
| 41 | TYPE | blank or 0: a word. 1 comment, 2 region, 3 block, 5 header or start, 9 first card |
| 43-47 | NAME | |
| 48 | SIGN | `+` or `-`: this word of a routine is data; the sign of a number |
| 49 | P | |
| 50 | Q | |
| 51-55 | SYMB | blank: the next cell |
| 57-61 | LINK | blank: the next cell |
| 63-80 | | comment |

A symbol begins with its letter, or with `9` for a local symbol, and
after that only its digits are read, wherever they stand in the field:
`A1`, `A 1` and `A0001` are one symbol, and so are `9-3` and `93`.

| TYPE | |
|---|---|
| 2 | NAME is a region; LINK is its size in cells. Before the first header. |
| 3, Q = 1 | NAME is a print line; LINK is its length in words of six characters |
| 5 | header: Q even, routines follow; Q odd, data |
| 5, with a regional SYMB | the start card: execute SYMB |
| 9 | the first card |

## An instruction

| P | | | Q | S is |
|---|---|---|---|---|
| 0 | execute S | | 0 | SYMB |
| 1 | input S: preserve H0, put S in it | | 1 | the symbol in cell SYMB |
| 2 | output to S: put (0) in S, restore H0 | | 2 | the symbol in the cell named in cell SYMB |
| 3 | restore S | | 3 | SYMB; trace this routine |
| 4 | preserve S | | 4 | SYMB; trace if the routine above is traced |
| 5 | replace (0) by S | | | |
| 6 | copy (0) in S | | | |
| 7 | branch to S if H5 is - | | | |

## A data word

A word of data holds a symbol in SYMB and the next cell in LINK. With
Q = 1 it is a data term, and P is its type: `01` an integer, `11` a
floating point number, `21` five characters, `31` an octal number.

## The system's cells

| | |
|---|---|
| `H0` | the communication cell: inputs and outputs |
| `H1` | the current instruction address, and under it those of the routines in progress |
| `H2` | the available space list |
| `H3` | the count of interpretation cycles, an integer |
| `H5` | the test cell |
| `W0`-`W9` | working cells |
| `W10` | names the integer that the random numbers come from |
| `W11` | the remainder of the last integer division |
| `W12`, `W13` | name routines executed at a monitor point (Q = 3) and when its routine ends |
| `W21` | names the left margin of the print line |
| `W22` | names the line spacing: 1 single, 2 double |
| `W24` | names the print line |
| `W25` | names the entry column |
| `W26` | names the list of trap actions, for J170 |
| `W30` | names the size of the field the line read processes take |
| `W31` | names the trace mode: 0 none, 1 everything, 2 as marked |

## The basic processes

"Sets H5" marks a process that leaves `+` or `-` in the test cell. A TEST
does nothing else. A FIND or LOCATE sets it `-` when it fails. Inputs
are removed from H0; where a definition says "leave (0)", that name
stays.

**General**

| | | |
|---|---|---|
| `J0` | No operation |  |
| `J1` | Execute (0) after restoring H0 |  |
| `J2` | TEST (0) = (1) | sets H5 |
| `J3` | Set H5 - | sets H5 |
| `J4` | Set H5 + | sets H5 |
| `J5` | Reverse the sense of H5 | sets H5 |
| `J6` | Reverse (0) and (1) |  |
| `J7` | Halt, proceed on GO |  |
| `J8` | Restore H0 |  |
| `J9` | ERASE cell (0) |  |

**Description lists**

| | | |
|---|---|---|
| `J10` | FIND the value of attribute (0) of (1) | sets H5 |
| `J11` | Assign (1) as the value of attribute (0) of (2) |  |
| `J12` | Add (1) at the front of the value list of attribute (0) of (2) |  |
| `J13` | Add (1) at the end of the value list of attribute (0) of (2) |  |
| `J14` | ERASE attribute (0) of (1) |  |
| `J15` | ERASE all attributes of (0) |  |
| `J16` | FIND an attribute of (0) randomly | sets H5 |

**Generator housekeeping**

| | | |
|---|---|---|
| `J17` | Generator set-up: context (0), subprocess (1) |  |
| `J18` | Execute the subprocess of the generator | sets H5 |
| `J19` | Generator clean-up | sets H5 |

**Working cells**

| | | |
|---|---|---|
| `J2n` | MOVE (0)-(n) into W0-Wn |  |
| `J3n` | Restore W0-Wn |  |
| `J4n` | Preserve W0-Wn |  |
| `J5n` | Preserve W0-Wn, then MOVE (0)-(n) into W0-Wn |  |

**Lists**

| | | |
|---|---|---|
| `J60` | LOCATE the next symbol after cell (0) | sets H5 |
| `J61` | LOCATE the last symbol on list (0) | sets H5 |
| `J62` | LOCATE (0) on list (1) (first occurrence) | sets H5 |
| `J63` | INSERT (0) before the symbol in cell (1) |  |
| `J64` | INSERT (0) after the symbol in cell (1) |  |
| `J65` | INSERT (0) at the end of list (1) |  |
| `J66` | INSERT (0) at the end of list (1) if it is not on it |  |
| `J67` | Replace (1) by (0) on list (2) (first occurrence) |  |
| `J68` | DELETE the symbol in cell (0) | sets H5 |
| `J69` | DELETE (0) from list (1) (first occurrence) | sets H5 |
| `J70` | DELETE the last symbol from list (0) | sets H5 |
| `J71` | ERASE list (0) |  |
| `J72` | ERASE list structure (0) |  |
| `J73` | COPY list (0) |  |
| `J74` | COPY list structure (0) |  |
| `J75` | Divide the list after location (0); the name of the remainder is the output |  |
| `J76` | INSERT list (0) after cell (1), and locate its last symbol | sets H5 |
| `J77` | TEST if (0) is on list (1) | sets H5 |
| `J78` | TEST if list (0) is not empty | sets H5 |
| `J79` | TEST if cell (0) is not empty | sets H5 |
| `J80` | FIND the symbol in the head of list (0) | sets H5 |
| `J81-J89` | FIND the nth symbol on list (0) | sets H5 |
| `J90` | Create an empty list |  |
| `J91-J99` | Create a list of the n symbols (n-1) to (0) |  |
| `J100` | Generate the symbols on list (1) for subprocess (0) | sets H5 |
| `J101` | Generate the cells of list structure (1) for subprocess (0) | sets H5 |
| `J102` | Generate the cells of tree (1) for subprocess (0) | sets H5 |

**Auxiliary storage**

| | | |
|---|---|---|
| `J105` | MOVE list structure (0) in from auxiliary storage | sets H5 |
| `J108` | TEST if list structure (0) is on auxiliary storage | sets H5 |

**Arithmetic**

| | | |
|---|---|---|
| `J110` | (1) + (2) into (0), leave (0) |  |
| `J111` | (1) - (2) into (0), leave (0) |  |
| `J112` | (1) x (2) into (0), leave (0) |  |
| `J113` | (1) / (2) into (0), leave (0) |  |
| `J114` | TEST if (0) = (1) | sets H5 |
| `J115` | TEST if (0) > (1) | sets H5 |
| `J116` | TEST if (0) < (1) | sets H5 |
| `J117` | TEST if (0) = 0 | sets H5 |
| `J118` | TEST if (0) > 0 | sets H5 |
| `J119` | TEST if (0) < 0 | sets H5 |
| `J120` | COPY (0) |  |
| `J121` | Set (0) identical to (1), leave (0) |  |
| `J122` | Take the absolute value of (0), leave (0) |  |
| `J123` | Take the negative of (0), leave (0) |  |
| `J124` | Clear (0), leave (0) |  |
| `J125` | Tally 1 in (0), leave (0) |  |
| `J126` | Count list (0) |  |
| `J127` | TEST if the data type of (0) is that of (1) | sets H5 |
| `J128` | Translate (0) to the data type of (1), leave (0) |  |
| `J129` | Produce a random number between 0 and (0) |  |

**Kinds of symbol**

| | | |
|---|---|---|
| `J130` | TEST if (0) is a regional symbol | sets H5 |
| `J131` | TEST if (0) names a data term | sets H5 |
| `J132` | TEST if (0) is a local symbol | sets H5 |
| `J133` | TEST if list (0) has been marked processed | sets H5 |
| `J134` | TEST if (0) is an internal symbol | sets H5 |
| `J136` | Make (0) local, leave (0) |  |
| `J137` | Mark list (0) processed, leave (0) |  |
| `J138` | Make (0) internal, leave (0) |  |

**Trace marks**

| | | |
|---|---|---|
| `J147` | Mark routine (0) to trace |  |
| `J148` | Mark routine (0) to propagate trace |  |
| `J149` | Mark routine (0) not to trace |  |

**Printing**

| | | |
|---|---|---|
| `J150` | Print list structure (0) |  |
| `J151` | Print list (0) |  |
| `J152` | Print symbol (0) |  |
| `J153` | Print data term (0) without name or type |  |
| `J154` | Clear the print line |  |
| `J155` | Print the line |  |
| `J156` | Enter symbol (0) left-justified | sets H5 |
| `J157` | Enter data term (0) left-justified | sets H5 |
| `J158` | Enter symbol (0) right-justified | sets H5 |
| `J159` | Enter data term (0) right-justified | sets H5 |
| `J160` | Tab to column (0) |  |
| `J161` | Increment the column by (0) |  |

**Loading and restart**

| | | |
|---|---|---|
| `J165` | Load routines and data |  |
| `J166` | Save on unit (0) for restart | sets H5 |

**Traps**

| | | |
|---|---|---|
| `J170` | Trap on (0) |  |
| `J171` | Return unused regional cells to H2 |  |

**Reading a line**

| | | |
|---|---|---|
| `J180` | Read a line | sets H5 |
| `J181` | Input a symbol from the line | sets H5 |
| `J182` | Input a data term from the line into (0) | sets H5 |
| `J183` | Set (0) to the next blank, leave (0) | sets H5 |
| `J184` | Set (0) to the next non-blank, leave (0) | sets H5 |
| `J185` | Set (1) to the next occurrence of character (0), leave (0) | sets H5 |
| `J186` | Input a character from the line | sets H5 |

**Parts of a word**

| | | |
|---|---|---|
| `J190` | Input P of cell (0) |  |
| `J191` | Input Q of cell (0) |  |
| `J192` | Input SYMB of cell (0) |  |
| `J193` | Input LINK of cell (0) |  |
| `J194` | Set (1) to be P of cell (0) |  |
| `J195` | Set (1) to be Q of cell (0) |  |
| `J196` | Set (1) to be SYMB of cell (0) |  |
| `J197` | Set (1) to be LINK of cell (0) |  |

**Miscellaneous**

| | | |
|---|---|---|
| `J200` | LOCATE the (0)th symbol on list (1) | sets H5 |
| `J201` | ERASE routine (0) |  |
| `J202` | Print a post mortem and continue |  |

The rest of the manual's processes are here too, each with an example
in `examples/`: blocks and region control words (J103, J171 to J176),
auxiliary storage (J105 to J109), tapes (J140 to J146, J167), save for
restart (J166), the traps on H2 and H3, J162 and J189. A tape is a file
given with `--tape`; one that is not given is a blank tape that lasts as
long as the run. J141 reads the console, and there is none, so it always
sets H5 -. The compressed and binary forms of cards, which W16 and W17
choose, were each machine's own; only IPL standard form is here.

## Commands

```
iplv run DECK [CARDS...]   load the deck and run it; CARDS go behind it in the reader
     --trace               trace every instruction
     --no-trace            trace none, whatever the deck's marks say
     --list                print the assembly listing first
     --stats               report cycles, storage and the processes used
     --cells N             cells of available space (22000)
     --cycles N            stop after N cycles (100000000)
     --tape U=FILE         mount FILE as tape U, 1 to 10; saved back if written
iplv check DECK            say what is wrong with a deck, without running it
iplv format DECK           print it in upper case, each symbol at the left of its field
iplv lsp                   the language server, for an editor
```

## Where this differs from the manual

The main README lists them. The ones a program can notice: a symbol may
be indented in its field; a `9` by itself is the internal symbol 9; a
regional cell that is erased is emptied and not put on available space
(only J171 puts unused regional cells there);
and the random numbers of `J129` come from the multiplier of the IBM
1620 system.
