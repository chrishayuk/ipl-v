# Description Lists

So far a symbol has been a name for a list or a number, or for nothing.
IPL-V has a way to say *things about* a symbol, and it is what the head
of a describable list was being kept for.

## Attributes and values

Any list may have a **description list**. It holds pairs: an *attribute*
and its *value*, both symbols. The name of the description list is kept
in the SYMB of the list's head, which is why the head was left empty.

You do not handle the description list yourself. Four processes do:

- **J10** FIND the value of attribute (0) of (1).
- **J11** Assign (1) as the value of attribute (0) of (2).
- **J14** ERASE attribute (0) of (1).
- **J15** ERASE all attributes of (0).

In each, the list being described is the *last* input. `J10` is a FIND:
if the list has that attribute, the value is the output and H5 is `+`; if
not, there is no output and H5 is `-`.

Any symbol can be an attribute and any symbol a value. A value can name a
list, a data term, or another described symbol, so there is no limit to
what can be said.

```iplv
GRASS IS GREEN                          9
                                        2 C                10
                                        2 G                10
                                        2 K                10
                                        2 N                10
                                        2 P                10
                                        2 T                10
                                        2 V                10
                                        5       00
                                          P0    10G1
SAY WHAT COLOUR G1 IS.                            P2
                                                10G1
                                                10V2
                                                10C0
ASSIGN V2 AS THE C0 OF G1.                        J11
                                                10G1
                                                  P2    0
P1: ENTER THE NAME OF (0) IN THE PRINT  1
LINE.                                   1
                                          P1    10N0
FIND THE VALUE OF N0 OF (0).                      J10
IT NAMES A DATA TERM: ENTER IT.                   J157  0
P2: PRINT WHAT COLOUR (0) IS.           1
                                          P2      J50
                                                  J154
                                                11W0
ITS NAME,                                         P1
                                                10T0
IS,                                               J157
                                                10K1
                                                  J161
                                                11W0
                                                10C0
FIND THE VALUE OF C0 OF 1W0,                      J10
AND ENTER THE NAME OF THAT.                       P1
                                                  J155
                                                  J30   0
                                        5       01
                                          K1    011
                                          T0    21 IS
G1 IS DESCRIBED BY 9-0.                   G1      9-0   0
                                          9-0     0
ITS NAME                                          N0
IS A DATA TERM.                                   9-1
ITS COLOUR                                        C0
IS V1.                                            V1    0
                                          9-1   21GRASS
                                          V1      9-0   0
                                          9-0     0
                                                  N0
                                                  9-1   0
                                          9-1   21GREEN
                                          V2      9-0   0
                                          9-0     0
                                                  N0
                                                  9-1   0
                                          9-1   21BROWN
                                        5         P0
```

```output
GRASS IS GREEN
GRASS IS BROWN
```

`G1` is a list with nothing on it: its LINK is `0`. All it has is a
description. `N0` and `C0` are symbols used purely as attributes; nothing
defines them, and they mean "name" and "colour" only because the program
uses them so.

The data shows how a description list is written on cards. The head of
`G1` holds a local symbol, `9-0`. Then comes the list `9-0`: an empty
head of its own (description lists are describable too), and then
attribute, value, attribute, value. The value of `N0` is `9-1`, a data
term written just below. The three symbols each have a `9-0` and a `9-1`
of their own, since each regional NAME starts the local symbols afresh.

`J11` does the same thing while the program runs. If the attribute is
there, its value is replaced; if it is not, the pair is added at the
front; if the list has no description list, one is made.

## Lists of values

An attribute has one value. When you want several, make the value a list:

- **J12** Add (1) at front of value list of attribute (0) of (2).
- **J13** Add (1) at end of value list of attribute (0) of (2).

They make the list the first time, so you need not.

## What this is for

A description list is a table that travels with its symbol. The programs
of the time used them for everything a later language would use a record
or a property list for: the Logic Theorist keeps, for each expression,
its main connective, its variables and how it was derived, all as
attributes. LISP's property lists are this idea, a year or two later.

`examples/describe.iplv` is a longer version of the program above, which
also adds an attribute to a symbol that has none, erases one, and prints
a description list.
