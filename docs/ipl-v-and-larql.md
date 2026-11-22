# IPL-V and LARQL, side by side

IPL-V (1957–1964) is a language for holding what a program knows as symbols
on lists, with attributes and values on each symbol's description list.
[LARQL](https://github.com/chrishayuk/larql) treats a trained transformer as
a store of knowledge and gives it a query language, LQL. The two use some
of the same words: entity, attribute or relation, value or target, walk,
insert. This note puts them next to each other, operation by operation, to
see where those words mean the same thing and where they do not. It does
not assume that they mean the same thing anywhere.

What is said about LARQL is quoted from its main branch at commit
`d12e1c6d9` (22 November 2026), with the file it comes from. Results on
LARQL's research branches are not used. What is said about IPL-V is from
the manual of 1964 and can be run in this repository.

## One example in both

The README of LARQL's graph library, `crates/larql-core`, builds this:

```rust
let mut graph = Graph::new();
graph.add_edge(Edge::new("France", "capital", "Paris"));
graph.add_edge(Edge::new("Paris", "river", "Seine"));
let path = shortest_path(&graph, "France", "Seine");
```

The same two facts in IPL-V are two description lists. `J11` assigns a
value to an attribute of a symbol; `J10` finds it, or sets H5 minus if
there is none. The river of the capital of France is two `J10`s, one after
the other:

```iplv
FRANCE, CAPITAL, PARIS                  9
                                        2 A                10
                                        2 P                10
                                        2 R                10
                                        5       00
FRANCE:                                   P0    10A1
ITS CAPITAL                                     10A2
IS PARIS.                                       10R1
                                                  J11
PARIS:                                          10A2
ITS RIVER IS                                    10A3
THE SEINE.                                      10R2
                                                  J11
THE RIVER OF THE                                10A1
CAPITAL OF FRANCE:                              10R1
                                                  J10
(NO CAPITAL: STOP.)                             700
                                                10R2
                                                  J10
(NO RIVER: STOP.)                               700
PRINT IT.                                         J152
THE CAPITAL OF THE                              10A3
SEINE: THERE IS NONE,                           10R1
SO H5 IS - AND THERE                              J10
IS NO OUTPUT.                                   709-1
                                                  J152  0
PRINT FRANCE, WITH                        9-1   10A1
ITS DESCRIPTION LIST.                             J150  0
                                        5       01
FRANCE.                                   A1      0     0
PARIS.                                    A2      0     0
THE SEINE.                                A3      0     0
THE ATTRIBUTE CAPITAL.                    R1      0     0
THE ATTRIBUTE RIVER.                      R2      0     0
                                        5         P0
```

```output
         A3
A1       9-1
9-1      0
         R1
         A2
```

It takes 29 interpretation cycles. The last thing printed is France's own
list, which is empty, and its description list, local to it as `9-1`:
the attribute `R1` and the value `A2`. That is all there is. Nothing about
France is anywhere else.

## Operation by operation

| | IPL-V | LARQL |
|---|---|---|
| **The unit** | A pair on a description list: attribute and value, both symbols, in two cells. | In a vindex: "Every FFN feature is an edge: gate (what triggers it) → down (what it produces)" (`docs/findings.md` §1), 348,160 of them in Gemma 3 4B. In `larql-core`: "subject/relation/object edges", a graph "separate from VINDEX3's `SystemGraph`" (`crates/larql-core/README.md`). |
| **Its name** | The programmer's. `R1` is the attribute because the program uses it as one. | "Labels are external. Relation labels come from the `larql-knowledge` project (probes, Wikidata triples, WordNet, AST pairs)" (`crates/larql-lql/docs/spec.md` §1). |
| **Asking** | `J10`: the value, or H5 minus and no output. | `DESCRIBE`: "Show all knowledge for an entity, grouped by layer band". `WALK`: "Feature scan — what gate features fire for a token's embedding", "a knowledge browser operation, NOT inference" (spec). |
| **Is what is stored what is used?** | Yes. The interpreter runs on the cells, and on nothing else. The Logic Theorist's search, run here from its 1963 listing, makes the same number of subproblems as in 1963 on every theorem, because the search is the list structures. | Not as extracted: "Forward pass traces show zero overlap between statically-extracted features and actually-activated features"; the section is headed "Attention routing is the missing index" (`docs/findings.md` §6). Aggregating across layers: "64% match rate against model inference" (§5). |
| **Adding a fact** | `J11`. The next `J10` anywhere in the program sees it. | `INSERT`, in one of two modes (spec, `INSERT`). KNN: an override at `cos > 0.75` that "Doesn't participate in the forward pass", "validated at 25K edges". COMPOSE: features that "participate in the forward pass and can chain for multi-hop", with "a Hopfield-style cap at ~5–10 facts per layer under template-shared prompts". |
| **Two hops** | Two `J10`s, as above. Exact, and if either link is missing the second is not asked. | `Graph::walk()` "follows a sequence of relations, picking the highest-confidence edge at each hop" (`crates/larql-core/src/algo/walk.rs`). And: "A stored edge or a graph walk does not by itself establish causal influence in model execution" (`crates/larql-core/README.md`). |
| **Cost** | Counted. The deck above uses cells from H2 for each pair and 29 cycles in all; the Logic Theorist's "effort" was this count. | Measured in time and in agreement with the model's own output, as above. |

## Where the words part company

**Entity.** In IPL-V an entity is a symbol: a cell with a name, which holds
nothing until the program puts something on its description list. In a
vindex the nearest thing is whatever a gate vector responds to, which is
found, not declared, and named afterwards from an external label set.

**Relation.** An IPL-V attribute is any symbol used as one; there is no
list of relations anywhere, and nothing stops a program using a routine's
name as an attribute. In LQL a relation is a label brought from outside,
in priority order, to name what an edge is taken to do.

**Value.** `J10` returns one symbol or none. An FFN edge's "target" is what
its down vector is nearest to among the tokens; LARQL's own finding is that
85 per cent of features have down vectors that "don't align with any single
token embedding" (`docs/findings.md` §4).

**Insert.** `J11` changes the one place the value is kept, so every later
question gets the new answer. The two LQL modes split what `J11` does in
one: KNN is exact retrieval that the model does not use; COMPOSE is
something the model uses, at a capacity of a few facts per layer.

**Walk.** A `J10` after a `J10` composes because the value the first finds
is the symbol the second asks about. Whether the model's own computation
composes in this way is the question LARQL's `larql-core` README declines
to settle by walking a graph.

## What this does not show

- That either system is a model of the other. The pairing is by
  vocabulary, and the table is there to show how far the vocabulary goes.
- Anything about LARQL beyond the quoted documents. I did not run LARQL;
  every LARQL figure here is LARQL's own, and most are on one model,
  Gemma 3 4B.
- Anything about what IPL-V's programs knew. A description list holds what
  a programmer put there, and IPL-V's exactness is the exactness of a
  filing system: it is never wrong about what it was told, and it is told
  everything.

## A question

IPL-V's processes are a complete account of what its knowledge can do:
find, assign, add to a value list, erase, generate. For each of them, is
there an operation on a model that does the same thing *in the model's own
computation*, and at what capacity? On LARQL's own evidence, finding has a
partial counterpart and assigning has one capped at a few facts per layer.
Generating every attribute of a symbol, which an IPL-V program does by
running `J100` over its description list, would be the next to look for.
