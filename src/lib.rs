//! For more information and instructions, see the [Adapton Lab
//! README](https://github.com/cuplv/adapton-lab.rust).
#![feature(box_patterns)]
#![feature(field_init_shorthand)]
//#![feature(rustc_private)]
//#![feature(custom_derive)]

//extern crate serialize;
//extern crate csv;
extern crate rand;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate adapton;

//extern crate iodyn;

/// Defines lab parameters `LabParams` and `LabDef`, the parameters
/// for running the test diagram from the [Adapton Lab
/// README](https://github.com/cuplv/adapton-lab.rust).
pub mod labdef;

/// This module of Adapton Lab extends Adapton's existing reflection
/// (see `adapton::engine::reflect`) with further reflections.  In
/// particular, we produce HTML output structure, for human user
/// interaction and consumption.
pub mod labviz;

/// **Generically implements** the test diagram in the [Adapton Lab
/// README](https://github.com/cuplv/adapton-lab.rust).
pub mod labrun;
