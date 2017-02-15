Adapton Lab: Generic Testing and Evaluation
==============================================

Quick Links
------------
 - [Rustdoc for Adapton Lab](http://adapton.org/rustdoc/adapton_lab/index.html)
 - [Adapton Homepage](http://adapton.org)

Quick Start
------------

Adapton uses the latest version of the Rust language and runtime.  To
use it, install **rust nightly** (the latest version of the compiler
and runtime).  Even better, install
[`rustup.rs`](https://github.com/rust-lang-nursery/rustup.rs) and
follow [its instructions for switching to the nightly
channel](https://github.com/rust-lang-nursery/rustup.rs#working-with-nightly-rust).


```
git clone https://github.com/cuplv/adapton-lab.rust
cd adapton-lab.rust
cargo run
```

This script will invoke the default behavior for Adapton Lab, which
consists of running a test suite over [Adapton's `dev`
branch](https://github.com/cuplv/adapton.rust/tree/dev).
Below, we give more introduction, background, details about
command-line parameters, and pointers to extend the test suite.

Introduction
--------------

This document describes _Adapton Laboratory_, or **Adapton
Lab** for short.  The Adapton Lab provides a generic (reusable) harness for testing
and evaluating a test suite that exercises various Adapton application layers:

 - **the Adapton engines**:
    - DCG: Demanded-Computation Graph-based caching, with generic change propagation.
    - Naive: No caching.
 - **the Adapton collections library**: sequences, finite maps, sets, graphs, etc.
 - **interesting algorithms over the collections library**, including:
    - standard graph algorithms
    - computational geometry algorithms
    - static analyses of programs

As a Rust library, **Adapton** provides both a data structures
collection and a runtime library to write generic incremental
computations.  At the highest level, this approach consists of the
programmer writing functional programs over inductive, persistant
structures, specifically:

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
allocation.

Background: Nominal memoization
-------------------------------

In the future, we hope to make nominal memoization implicit;
currently, only explicit techniques exist.  (Aside: Past work on
_implicit_ self-adjusting computation focused only on making the use
of so-called modifiable references implicit; this is a complementary
and orthogonal problem to implicitly choosing a naming strategy for
nominal memoization).

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


Bibliography
------------

*Adapton Papers*:

- [_Typed Adapton: Refinement types for nominal memoization_, Working draft](https://arxiv.org/abs/1610.00097).
- [_Incremental computation with names_, **OOPSLA 2015**](http://arxiv.org/abs/1503.07792).  
- [_Adapton: Composable, demand-driven incremental computation_, **PLDI 2014**](http://www.cs.umd.edu/~hammer/adapton/).  

*Other Papers*:

- [*Incremental computation via function caching*](http://dl.acm.org/citation.cfm?id=75305)  
  *Bill Pugh and Tim Teitelbaum.* **POPL 1989.**  
  - structural memoization, of hash-cons'd, purely-functional data structures
  - (structurally-) memoized function calls, to pure computations


Defining a Commutative Diagram of From-Scratch Consistency 
-----------------------------------------------------------

With testing and performance evalaution both in mind, Adapton Lab
introduces several data structures and computations that can be
instantiated generically.  These elements can be related
diagrammatically, shown further below.

 - `Input_i`: The `i`th input (a data structure). Generically, this
   consists of abstract notions of **input generation** and
   **editing**.  We capture these operations abstractly in Rust with
   traits
   [Edit](http://adapton.org/rustdoc/adapton_lab/labdef/trait.Edit.html)
   and
   [Generate](http://adapton.org/rustdoc/adapton_lab/labdef/trait.Generate.html).
 - `Output_i`: The `i`th output (a data structure). For validating
   incremental output against non-incremental output (see diagram
   below), we compare output types for **equality**.
 - `Compute`: The computation relating the `i`th Input to the `i`th
    Output (a computation).  We capture this abstraction in Rust with
    [The Compute trait](http://adapton.org/rustdoc/adapton_lab/labdef/trait.Compute.html).
    We use the same computation to define both incremental and non-incremental algorithms.
 - `Edit_i`: The input change (aka input _edit_ or _delta_) relating
   the ith input to the `i+1`th input (a computation).  ith output to
   the `i+1`th output (a computation).  We only require that values of
   each output type can be compared for equality.
 - `Update_i`: The output change relating the `i+1`th input to the
   `i+1`th output, reusing the computation of the computation of
   `Output_i` from `Input_i` in the process, using its DCG and change
   propagation. 

Note that while the input and outputs are data structures, their
relationships are all computations: The input is modified by a
computation `Edit_1`, and to compute `Output_2`, the system has two
choices:

 - **Naive**: Run `Compute` over `Input_2`, (fully) computing `Output_2` from
   `Input2`.  _This relationship is shown as horizontal edges in the diagram_.

 - **DCG**: Reuse the traced computation of `Compute` over `Output_1`,
   changing `Output_1` into `Output_2` in the process, via
   change-propagation over the DCG.  This relationship is shown as
   _vertical edges on the right of the diagram_.

**From-scratch consistency** is a meta-theoretical property that
   implies that the DCG approach is semantically equivalent to the
   naive approach.  That is, its the property of the diagram below
   commuting.

**Diagram Example.**
Suppose we consider `i` from `1` to `4`, to show these relationships diagrammatically:

```
        |
        |  Generate
       \|/ 
        `  
      Input_1 --> Compute --> Output_1
        |                       | 
        |  Edit_1               |   Update_1
       \|/                     \|/
        `                       ` 
      Input_2 --> Compute --> Output_2
        |                       | 
        |  Edit_2               |   Update_2
       \|/                     \|/
        `                       ` 
      Input_3 --> Compute --> Output_3
        |                       | 
        |  Edit_3               |   Update_3
       \|/                     \|/
        `                       ` 
      Input_4 --> Compute --> Output_4
```


Generation and Editing Parameters
---------------------------------

To get a quick list of command-line options for Adapton Lab, use `-h`:

```
cargo run -- -h
```

Adapton Lab generates and edits inputs generically (the vertical edges
on the left of the diagram above).

These operations are tuned by the lab user through several
**generation parameters** (which also control editing).  An
implementation chooses how to interpret these parameters, with the
following guidelines:

```
   -a, --artfreq <artfreq>      for the Editor: the frequency of articulations, measured in non-nominal constructors.
   -b, --batch <batch>          for the Editor: the number of edits that the Editor performs at once.
   -d, --demand <demand>        for the Archivist: the number of output elements to demand; only relevant for lazy Archivists.
   -L, --lab <labname>          determines the Editor and the Archivist, from the lab catalog
   -l, --loopc <loopc>          for the Editor and Archivist: the loop count of edit-and-compute.
   -s, --size <size>            for the Editor: the initial input size generated by the Editor.
       --validate <validate>    a boolean indicating whether to validate the output; the default is true.
```

Testing
---------

Rust does not (yet) implement Typed Adapton, only Nominal Adapton.  In
other words, it is possible to misuse the Rust interface and deviate
from what would be permitted by Typed Adapton.  These deviations can
lead to run-time type errors, to memory faults and stack overflow.

One purpose of this test harness is to test the program `Compute`
commutes in the diagram above: That naive recomputation always matches
the behavior of nominal memoization.


To visualize this behavior, try this command:

```
cargo run -- --run-viz
```

(Also: When no options are given to Adapton Lab, it defaults to this behavior.)

After the command completes, inspect this directory of generated HTML:

```
open lab-results/index.html
```


Evaluation
-----------

After we test `Compute` and we validate enough test data, we want to
measure the performance differences between running `Compute` naively
and using nominal memoization.

To run timing measurements on larger input sizes, try this command:

```
cargo run -- --run-bench
```

After it completes, inspect this directory of generated HTML:

```
open lab-results/index.html
```
