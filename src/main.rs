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
  //fn construct_exp(len: isize) -> Vec<NameElse<char>>{
  //  let cap = len as _ ;
  //  let mut input = Vec::with_capacity(cap);
  //  for _ in 0..len {
  //    input.push(NameElse::Else('1'));
  //    input.push(NameElse::Else('+'));
  //  }
  //  input.push(NameElse::Else('1'));
  //  input
  //};

  fn push_char(name_step:usize, name:usize, ch: char, l:List<char>) -> List<char>{
    let l = if name % name_step == 0 {
      let l = <List<char> as ListIntro<char>>::art(cell(name_of_usize(name), l));
      let l = <List<char> as ListIntro<char>>::name(name_of_usize(name), l);
      l
    } else { l } ;
    let l = <List<char> as ListIntro<char>>::cons(ch, l);
    l
  };

  fn construct_input(len: isize) -> List<char> {
    let mut input: List<char> = List::Nil;
    for _ in 0..len {
      input = push(input.clone(), '1');
      input = push(input.clone(), '+');
    }
    input = push(input.clone(), '1');
    input
  };

  fn prepend_input(input:List<char>, name:isize) -> List<char> {
    let input = <List<char> as ListIntro<char>>::art(cell(name_of_isize(name), input));
    let input = <List<char> as ListIntro<char>>::name(name_of_isize(name), input);
    let input = push(input.clone(), '+');
    let input = push(input.clone(), '1');
    input
  };

  fn doit(input: Art<List<char>>) -> isize {
    //let list : List<char>  = list_of_vec(&input);
    let input = force(&input);
    let tree : Tree<char> = 
      ns(name_of_str("tree_of_list"),
         ||tree_of_list(Dir2::Left, input));
    
    // TODO: How to use names here?
    let tokenized_input : Tree<Tok> = 
      ns(name_of_str("tok_of_char"),
         ||tok_of_char(tree));
    // TODO: How to use names here?    
    let postfix : List<Tok> = 
      ns(name_of_str("postfix_of_infix"),
         ||postfix_of_infix(tokenized_input));
    
    let postfix_tree : Tree<Tok> = 
      ns(name_of_str("tree_of_list2"),
         ||tree_of_list(Dir2::Right, postfix));	
    
    // TODO: How to use names here?
    ns(name_of_str("evaluate_postfix"),
       ||evaluate_postfix(postfix_tree))
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
    input = push(input, '1');
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
    input = push(input, '1');
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
