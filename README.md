Adapton Lab: Generic Testing and Evaluation
==============================================

This document describes _Adapton Laboratory_, or **Adapton
Lab** for short.  The Adapton Lab provides a generic (reusable) harness for testing
and evaluating Adapton application layers:

 - **the Adapton engines**:
    - Demanded-Computation Graph (DCG) and
    - Naive (No caching).
 - **the Adapton collections library**:
    - Sequences,
    - Finite maps, Sets,
    - Graphs
 - **interesting algorithms over the collections library**, including:
    - standard graph algorithms
    - computational geometry algorithms
    - static analyses of programs

Adapton provides both a data structures collection and a runtime
library to write generic incremental computations.  At the highest
level, this approach consists of the programmer writing functional
programs over inductive, persistant structures, specifically:

 - **lists**, 
 - balanced trees representing **sequences**, 
 - hash-tries representing **finite maps**, **finite sets** and **graphs**.
 - coinductive (demand-driven) versions of the structures listed above.

To a first approximation, the Adapton methodology for writing
incremental algorithms consists of writing a functional (eager or
lazy) program over an unchanging input, producing an unchanging
output.  Refining that approximation, the programmer additionally uses
explicit abstractions for (explicit) _nominal memoization_, which
associates a first-class, dynamically-scoped name with each dynamic
allocation.  In the future, we hope to make nominal memoization
implicit; currently, only explicit techniques exist.  (Aside: Past
work on _implicit_ self-adjusting computation focused only on making
the use of so-called modifiable references implicit; this is a
complementary and orthogonal problem to implicitly choosing a naming
strategy for nominal memoization).

**Nominal Adapton** gave the first operational semantics for nominal
memoziation and it included preliminary techniques for encoding lists,
sequences, maps and sets (OOPSLA 2015).  These collections were
heavily inspired by work on incremental computation via function
caching by Pugh and Teitelbaum (POPL 1989).  Nominal Adapton replaces
structural naming strategies (aka hash-consing) with an explicit
approach, permitting imperative cache effects.  It suggests several
naming straties for computations that use these collections.  A
central concern is authoring algorithms that do not unintentionally
overwrite their cache, causing either unintended _churn_ or
_feedback_; each such effect deviates from purely-functional behavior,
which affects the programmer's reasoning about dynamic incremental
behavior.

**Typed (Nominal) Adapton** gives a useful static approximation of the
store-naming effects of nominal memoization, making it possible to
program generic library code, while avoiding unintended churn and
feedback.  Unlike other type systems for enforcing nominal structure,
Typed Adapton uses a type and effect system to enforce that the
_dynamic scoping_ of nominal memoization is _write-once_, aka,
functional, not imperative or relational.  Other nominal type systems
focus on enforcing _lexical scoping_ of first-class binders; this
problem and its soltuions are orthogal to enforcing the nominal
structure of a nominal memoization.

_Rust does not (yet) implement Typed Adapton, only Nominal Adapton_.
In other words, _it is possible to misuse the Rust interface and
deviate from what would be permitted by Typed Adapton_.  One purpose
of this test harness is to test that algorithms adhere to
**from-scratch consistency** when the programmer expects them to do
so.

Commutative Diagram: From-Scratch Consistency of Incremental Computation
-------------------------------------------------------------------------

With testing and performance evalaution both in mind, the test and
evaluation harness introduces several data structures and computations
that can be instantiated generically.  These elements can be related
diagrammatically, shown further below.

 - `Inputi` -- The ith input (a data structure). Generically, this
   consists of abstract notions of **input generation** and
   **editing**.
 - `Outputi` -- The ith output (a data structure). For validating output (see diagram below), we compare output types for **equality**.
 - `Compute` -- The computation relating the `i`th Input to the `i`th
                Output (a computation).
 - `DIni` -- The input change (aka "input delta") relating the ith
              input to the `i+1`th input (a computation).
 - `DOuti` -- The output change (aka "ouput delta") relating the ith
              output to the `i+1`th output (a computation).

Note that while the input and outputs are data structures, their
relationships are all computations: The input is modified by a
computation `DIn1`, and to compute `Output2`, the system has two
choices:

 - **Naive**: Run `Compute` over `Input2`, (fully) computing `Output2` from
   `Input2`.  This relationship is shown as horizontal edges in the diagram.
 - **DCG**: Reuse the traced computation of `Compute` over `Output1`,
   changing `Output1` into `Output2` in the process, via
   change-propagation over the DCG.  This relationship is shown as
   vertical edges on the right of the diagram.

From-scratch consistency is a key assumption of this methodology: This
   (side-effecting) computation is semantically equivalent to the full
   computation of `Compute` from Input2.

**Diagram Example.**
Suppose we consider `i` from 1 to 3, to show these relationships diagrammatically:

```       
      Input1 --> Compute --> Output1
        |                       | 
        |  DIn1 (mutation)      |   DOut1 (uses DCG + change propagation)
       \|/                     \|/
        `                       ` 
      Input2 --> Compute --> Output2
        |                       | 
        |  DIn2 (mutation)      |   DOut2 (uses DCG + change propagation)
       \|/                     \|/
        `                       ` 
      Input3 --> Compute --> Output3
        |                       | 
        |  DIn3 (mutation)      |   DOut3 (uses DCG + change propagation)
       \|/                     \|/
        `                       ` 
      Input4 --> Compute --> Output4
```


Generation Parameters
---------------------

Adapton Lab generates and edits inputs generically.  These operations
are tuned by the lab user through several **generation parameters**.
An implementation chooses how to interpret these parameters, with the
following guidelines:

 - a **seed** for deterministic psuedo-randomness (`--seed`)
 - a natural number representing **size** (`--size`)
 - q natural number representing **expected nominal sparsity**; ie,
    its **gauge** (`--gauge`). Bigger is more sparse; 1 is finest
    (e.g., names for each input element). 0 means no names at all.
 - a bit indicating whether nominal sparisity is _regular_ or
     _content-determined_ (`--name-regular` or `--name-bycontent`).


In turn, these parameters control the following processes on input:

 - Deterministic psuedo-random instance **generation**.
 - Deterministic psuedo-random **edits**, the `DIn_i` shown in the figure.


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

