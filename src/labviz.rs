/// This module of Adapton Lab extends Adapton's existing reflection
/// (see `adapton::engine::reflect`) with further reflections.  In
/// particular, we produce HTML output structure, for human user
/// interaction and consumption.
use adapton::engine::reflect::*;

/// The `Div` struct represents a restricted form of `<div>` tags in
/// HTML.  A distinguished `tag` class holds the name of the Rust
/// datatype reflected into this `Div`.  The other `classes` hold bits
/// that signal various subcases (e.g., of `enum`s in the `reflect`
/// module).  Finally, for Rust structures that have subfields and/or
/// substructure, the `Div`'s `extent` field lists their reflections
/// into `Div`s.  In principle, the produced `Div` structure has an
/// equivalent amount of information to the corresponding Rust
/// datatype, and could be "parsed" back into this Rust datatype later
/// (let's not do that, though!).
#[derive(Debug,Clone)]
pub struct Div {
  pub tag:     String,
  pub classes: Vec<String>,
  pub extent:  Box<Vec<Div>>,
}

// Questions:
// Reflections of names? 
// Do we expose their internal structure as `div`s, or stringify them?
// For paths, clearly we wanted to expose their structure.
// Perhaps for forked names such as `11.L.r.L` we'll want something similar?

// Thoughts:

// Both: We want names to be like strings when they are used as CSS
// classes that control when, during user interaction, certain div
// elements highlight, focus or toggle between hide/show.  On the
// other hand, we may also want the user to be able to see and inspect
// the internal structure of names, e.g., to select/highlight related
// names in the lineage.  E.g., `11.L.r.L` is likely related to
// `11.L.r.r` since a single `fork` operation produced them both.

// div![ loc ;;
//       x.path.reflect(),
//       x.name.reflect() ]

// div![ dcg-edge ;;
//       x.loc.reflect(),
//       x.succ.reflect() ]

// div![ dcg-trace ;;
//       x.effect.reflect(),
//       x.edge.reflect(),
//       x.extent.reflect() ]
