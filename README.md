This document describes _Adapton Laboratory_, or **Adapton
Lab** for short.  The Adapton Lab provides a generic (reusable) harness for testing
and evaluating Adapton application layers:

 - the Adapton engines:
    - Demanded-Computation Graph (DCG) and
    - Naive (No caching).
 - the Adapton collections library:
    - Sequences,
    - Finite maps, Sets,
    - Graphs
 - intresting algorithms and computations over the collections library, including:
    - standard graph algorithms
    - computational geometry algorithms
    - static analyses of programs

The test and evaluation harness introduces several data structures and
computations that can be instantiated generically:

 - `Inputi` -- The ith input (a data structure). Generically, this
   consists of abstract notions of **input generation** and
   **editing**:
     - Deterministic psuedo-random instance **generation**, based on
        - a **seed** for deterministic psuedo-randomness (`--seed`)
       	- a natural number representing **size** (`--size`)
        - q natural number representing **expected nominal sparsity**; ie,
               its **gauge** (`--gauge`). Bigger is more sparse; 1 is
               finest (e.g., names for each input element). 0 means no
               names at all.
        - a bit indicating whether nominal sparisity is _regular_ or
          _content-determined_ (`--name-regular` or
          `--name-bycontent`).
    - Deterministic psuedo-random **edit**, aka Compute-Input-Delta, aka the `DIn_i` in the figure.
 - `Outputi` -- The ith output (a data structure)
 - `Compute` -- The computation relating the `i`th Input to the `i`th
                Output (a computation)
 - `DIni` -- The input change (aka "input delta") relating the ith
              input to the `i+1`th input (a computation)
 - `DOuti` -- The output change (aka "ouput delta") relating the ith
              output to the `i+1`th output (a computation)

Note that while the input and outputs are data structures, their
relationships are all computations: The input is modified by a
computation `DIn1`, and to compute `Output2`, the system has two
choices:

 - Run `Compute` over `Input2`, (fully) computing `Output2` from
   `Input2`.
 - Reuse the computation of `Compute` over `Output1`, changing
   `Output1` into `Output2` in the process.  

From-scratch consistency is a key assumption of this methodology: This
   (side-effecting) computation is semantically equivalent to the full
   computation of `Compute` from Input2.

Suppose we consider `i` from 1 to 3, to show these relationships diagrammatically:

```       
      Input1 --> Compute --> Output1
        |                       | 
        |  DIn1                 |   DOut1
       \|/                     \|/
        `                       ` 
      Input2 --> Compute --> Output2
        |                       | 
        |  DIn2                 |   DOut2
       \|/                     \|/
        `                       ` 
      Input3 --> Compute --> Output3
        |                       | 
        |  DIn3                 |   DOut3
       \|/                     \|/
        `                       ` 
      Input4 --> Compute --> Output4
```

Adapton provides both a data structures collection and a runtime
library to write `Compute`, and to express the input changes `DIni`.
At the highest level, this approach consists of the programmer writing
functional programs over inductive, persistant structures,
specifically:

 - lists, 
 - balanced trees representing sequences, 
 - hash-tries representing finite maps, finite sets and graphs.

That is to say, to a first approximation, the Adapton methodology for
writing `Compute` consists of writing a functional (eager or lazy)
program.  Currently, the programmer thinks about programming
abstractions that we collectively refer to as (explicit) _nominal
memoization_.  In the future, perhaps _nominal memoization_ can be
made implicit; currently, only explicit techniques exist.  (Aside:
Past work on _implicit_ self-adjusting computation focused only on
making the use of so-called modifiable references implicit; this is a
complementary and orthogonal problem to implicitly choosing a naming
strategy for nominal memoization).

Nominal Adapton gave the first operational semantics for nominal
memoziation and it included preliminary techniques for encoding lists,
sequences, maps and sets was given by Nominal Adapton.  These
collections were heavily inspired by work on incremental computation
via function caching by Pugh and Teitelbaum (POPL 1989).  Nominal
Adapton replaced structural naming strategies (aka hash-consing) with
an explicit approach, permitting imperative cache effects.  It
suggested several naming straties for computations that use these
collections.  A central concern is authoring algorithms that do not
unintentionally overwrite their cache, causing either unintended churn
or feedback.

The type system we call Typed, Nominal Adapton gives a useful static
approximation of the store-naming effects of nominal memoization,
making it possible to program generic library code, while avoiding
unintended churn and feedback.  Unlike other type systems for
enforcing nominal structure, Typed Adapton uses a type and effect
system to enforce that the _dynamic scoping_ of nominal memoization is
_write-once_, aka, functional, not imperative or relational.  Other
nominal type systems focus on enforcing _lexical scoping_ of
first-class binders; this problem and its soltuions are orthogal to
enforcing the nominal structure of a nominal memoization.

Testing
---------

Rust does not (yet) implement Typed Adapton, only Nominal Adapton.  In
other words, it is possible to misuse the Rust interface and deviate
from what would be permitted by Typed Adapton.  One purpose of this
test harness is to test the program `Compute` commutes in the diagram
above: That naive recomputation always matches the behavior of nominal
memoization.

TODO -- Explain how to test an instance of `Input`, `Di`, `Compute` and `Output`.

Evaluation
-----------

After we test `Compute` and we validate enough test data, we want to
measure the performance differences between running `Compute` naively
and using nominal memoization.

TODO -- Explain how to evaluate an instance of `Input`, `Di`, `Compute` and `Output`.

