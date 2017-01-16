//! For more information and instructions, see the [Adapton Lab
//! README](https://github.com/cuplv/adapton-lab.rust).

#![feature(field_init_shorthand)]
//#![feature(rustc_private)]
//#![feature(custom_derive)]

//extern crate serialize;
//extern crate csv;
extern crate rand;

#[macro_use]
extern crate adapton;

extern crate pmfp_collections;

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

/// Provides **concrete instances** of the test diagram from the
/// [Adapton Lab README](https://github.com/cuplv/adapton-lab.rust).
pub mod catalog;

use labdef::*;

// fn csv_of_runtimes(path:&str, samples: Vec<Sample>) {
//   let path = Path::new(path);
//   let mut writer = csv::Writer::from_file(path).unwrap();
//   for r in samples.into_iter() {
//     //println!("{:?}",r);
//     //writer.encode(r).ok().expect("CSV writer error");
//   }
// }

fn lab_params_defaults() -> LabParams {
  return LabParams {
    sample_params: SampleParams{
      input_seeds: vec![0],
      generate_params: GenerateParams{
        size: 256,
        //size: 100000,
        gauge:1,
        nominal_strategy:NominalStrategy::Regular,
      },
      validate_output: true,
      change_batch_size: 1,
    },
    change_batch_loopc:10,
  }
}

// TODO -- Put these implementations into a 'catalog' module.

fn run_all_tests() {
  let params = lab_params_defaults();
  let tests = catalog::all_tests();
  for test in tests.iter() {
    println!("Running: {:?}", test.name());
    let results = test.run(&params);
    labviz::write_test_results(test.name(), test.url(), &results);
  }  
}

#[test]
fn test_all() { run_all_tests() }
fn main2() { run_all_tests() }

fn main () {
  use std::thread;
  let child =
    thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(move || { 
      main2()
    });
  let _ = child.unwrap().join();
}
