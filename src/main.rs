use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::path::Path;

extern crate time;
extern crate csv;

#[macro_use]
extern crate adapton;

use adapton::macros::*;
use adapton::collections::*;
use adapton::engine::*;


pub fn runtime_harness(max_len: isize) -> Vec<(isize, u64, isize)> {

  let mut runtimes: Vec<(isize, u64, isize)> = vec![];

  fn construct_input(len: isize) -> List<char> {
    panic!("")
  };

  fn prepend_input(input:List<char>, name:isize) -> List<char> {
    panic!("")
  };

  fn doit(input: Art<List<char>>) -> isize {
    panic!("")
  };

  fn csv_of_runtimes(path:&str, runtimes: Vec<(isize, u64, isize)>) {
    let path = Path::new(path);
    let mut writer = csv::Writer::from_file(path).unwrap();
    for r in runtimes.into_iter() {
      //println!("{:?}",r);
      writer.encode(r).ok().expect("CSV writer error");
    }
  };

  { // This should be really fast:
    init_dcg();
    assert!(engine_is_dcg());
    
    let mut input : List<char> = List::Nil;
    input = list_push(input, '1');
    for i in 1..max_len {
        input = prepend_input(input, i);
        let input = input.clone();
        let dcg_start = time::precise_time_ns();
        let input_cell = cell(name_of_str("input"), input);
        let (_, dcg_out) = eager!(doit, input:input_cell);
        let dcg_end = time::precise_time_ns();
        println!("DCG: {}, {} ms, {}", i, (dcg_end - dcg_start) as u64 / 1000000, dcg_out);
        runtimes.push((i,dcg_end - dcg_start, dcg_out));
    }
      csv_of_runtimes("./dcg.csv", runtimes.clone()); 
      //runtimes
  }

  if false 
  { // This is really, really slow:
    init_naive();
    assert!(engine_is_naive());
    
    let mut input : List<char> = List::Nil;
    input = list_push(input, '1');
    for i in 1..max_len {
      input = prepend_input(input, i);
      let naive_start = time::precise_time_ns();
      let naive_out = doit(cell(name_of_str("input"), input.clone()));
      let naive_end = time::precise_time_ns();
      //println!("Naive: {}, {}, {}", i, naive_end - naive_start, naive_out);
      runtimes.push((i,naive_end - naive_start, naive_out));}
    csv_of_runtimes("./naive.csv", runtimes.clone()); 
    //runtimes
  } else {}
  Vec::new()
}



#[test]
fn test_runtime_harness() {
  use std::thread;
  let child =
    thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(move || { 
      runtime_harness(16);
    });
  let _ = child.unwrap().join();
}



fn main() {
    println!("Hello, world!");
}
