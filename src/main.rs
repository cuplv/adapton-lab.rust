#![feature(field_init_shorthand)]
#![feature(rustc_private)]
//#![feature(custom_derive)]

extern crate serialize;
extern crate csv;
extern crate rand;

#[macro_use]
extern crate adapton;

// lab definitions:
pub mod labdef;
pub mod labrun;
pub mod catalog;

use labdef::*;
//use labrun::*;
//use catalog::*;

// fn csv_of_runtimes(path:&str, samples: Vec<Sample>) {
//   let path = Path::new(path);
//   let mut writer = csv::Writer::from_file(path).unwrap();
//   for r in samples.into_iter() {
//     //println!("{:?}",r);
//     //writer.encode(r).ok().expect("CSV writer error");
//   }
// }

fn labexp_params_defaults() -> LabExpParams {
  return LabExpParams {
    sample_params: SampleParams{
      input_seeds: vec![0],
      generate_params: GenerateParams{
        size:10,
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
  let params = labexp_params_defaults();
  let tests = catalog::all_tests();
  for test in tests.iter() {
    println!("Test: {:?}", test.name());
    let results = test.run(&params);
    drop(results)
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
