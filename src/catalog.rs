use labdef::*;
use adapton::collections::*;
use adapton::engine::*;
use rand::{Rng};
use std::marker::PhantomData;
use std::rc::Rc;
use pmfp_collections::gauged_raz::{Raz,RazTree};
use pmfp_collections::split_btree_cursor::{gen_level};


#[derive(Clone,Debug)]
pub struct UniformInsert<T,S> { t:PhantomData<T>, s:PhantomData<S> }

impl<S> Generate<RazTree<usize>> for UniformInsert<RazTree<usize>, S> {
  fn generate<R:Rng> (rng:&mut R, params:&GenerateParams) -> RazTree<usize> {
    let mut r = Raz::new();
    for i in 0..params.size {
      if i % params.gauge == 0 {
        r.archive_left( gen_level() );
      } else { } ;
      r.push_left(i);
    }
    r.unfocus()
  }
}

impl Edit<RazTree<usize>, usize> for UniformInsert<RazTree<usize>, usize> {
  fn edit_init<R:Rng>(_rng:&mut R, params:&GenerateParams) -> usize { 
    return params.size // Initial editing state = The size of the generated input
  }
  fn edit<R:Rng>(tree:RazTree<usize>, i:usize,
                 rng:&mut R, params:&GenerateParams) -> (RazTree<usize>, usize) {
    let mut t = tree;
    let pos = rng.gen::<usize>() % ( i + 1 );
    let mut r = t.focus( pos ).unwrap();
    r.push_left( rng.gen() );
    let t = r.unfocus();    
    (t, i + 1)
  }
}



#[derive(Clone,Debug)]
pub struct UniformPrepend<T,S> { t:PhantomData<T>, s:PhantomData<S> }

impl<S> Generate<List<usize>> for UniformPrepend<List<usize>,S> {
  fn generate<R:Rng>(rng:&mut R, params:&GenerateParams) -> List<usize> {
    let mut l : List<usize> = list_nil();
    for i in 0..params.size {
      if i % params.gauge == 0 {
        l = list_art(cell(name_of_usize(i), l));
        l = list_name(name_of_usize(i), l);
      } else { } ;
      let elm : usize = rng.gen() ;
      let elm = elm % params.size ;
      l = list_cons(elm,  l);
    } ;
    l
  }
}

impl Edit<List<usize>, usize> for UniformPrepend<List<usize>,usize> {
  fn edit_init<R:Rng>(_rng:&mut R, params:&GenerateParams) -> usize { 
    return params.size // Initial editing state = The size of the generated input
  }
  fn edit<R:Rng>(l_preedit:List<usize>, 
                 next_name:usize,
                 rng:&mut R, params:&GenerateParams) -> (List<usize>, usize) {
    let mut l = l_preedit ;
    let i = next_name ;
    if i % params.gauge == 0 {
      l = list_art(cell(name_of_usize(i), l));
      l = list_name(name_of_usize(i), l);      
    } else { } ;
    let elm : usize = rng.gen() ;
    let elm = elm % params.size ;
    (list_cons(elm, l), i + 1)
  }
}


//#[derive(Clone,Debug)]
//pub struct UniformPrepend<T,S> { T:PhantomData<T>, S:PhantomData<S> }

type Pt2D = (usize,usize); // TODO Fix this

impl<S> Generate<List<Pt2D>> for UniformPrepend<List<Pt2D>,S> { // TODO
  fn generate<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> List<Pt2D> {
    //panic!("TODO")
    list_nil()
  }
}

impl Edit<List<Pt2D>,usize> for UniformPrepend<List<Pt2D>,usize> { // TODO
  fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 0 }
  fn edit<R:Rng>(state:List<Pt2D>, st:usize, _rng:&mut R, _params:&GenerateParams) -> (List<Pt2D>, usize) {
    //TODO
    (state, st)
  }
}


#[derive(Clone,Debug)]
pub struct LazyMap { }
#[derive(Clone,Debug)]
pub struct EagerMap { }

#[derive(Clone,Debug)]
pub struct LazyFilter { }
#[derive(Clone,Debug)]
pub struct EagerFilter { }

#[derive(Clone,Debug)]
pub struct ListTreeMax { }

#[derive(Clone,Debug)]
pub struct ListTreeSum { }

#[derive(Clone,Debug)]
pub struct ListReverse { }

#[derive(Clone,Debug)]
pub struct LazyMergesort { }
#[derive(Clone,Debug)]
pub struct EagerMergesort { }

#[derive(Clone,Debug)]
pub struct Quickhull { }

pub struct RazTest1 {} 

impl Compute<List<usize>,List<usize>> for EagerMap {
  fn compute(inp:List<usize>) -> List<usize> {
    list_map_eager(inp,Rc::new(|x| x * x))
  }
}

impl Compute<List<usize>,List<usize>> for EagerFilter {
  fn compute(inp:List<usize>) -> List<usize> {
    list_filter_eager(inp,Rc::new(|x:&usize| (*x) % 3 == 0))
  }
}

impl Compute<List<usize>,List<usize>> for LazyMap {
  fn compute(inp:List<usize>) -> List<usize> {
    list_map_lazy(inp,Rc::new(|x| x * x))
  }
}

impl Compute<List<usize>,List<usize>> for LazyFilter {
  fn compute(inp:List<usize>) -> List<usize> {
    list_filter_lazy(inp,Rc::new(|x:&usize| (*x) % 3 == 0))
  }
}

impl Compute<List<usize>,List<usize>> for ListReverse {
  fn compute(inp:List<usize>) -> List<usize> {
    list_reverse(inp, list_nil())
  }
}

impl Compute<List<usize>,usize> for ListTreeMax {
  fn compute(inp:List<usize>) -> usize {
    let tree : Tree<usize> = 
      ns(name_of_str("tree_of_list"),
         move|| tree_of_list(Dir2::Left,inp));
    monoid_of_tree(tree, 0, 
                   Rc::new(|x,y| if x > y { x } else { y }))
  }
}

impl Compute<List<usize>,usize> for ListTreeSum {
  fn compute(inp:List<usize>) -> usize {
    let tree : Tree<usize> = 
      ns(name_of_str("tree_of_list"),
         move|| tree_of_list(Dir2::Left,inp));
    monoid_of_tree(tree, 0, 
                   Rc::new(|x,y| x + y ))
  }
}

impl Compute<List<usize>,List<usize>> for LazyMergesort {
  fn compute(inp:List<usize>) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    mergesort_list_of_tree2(tree,None)
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = 
      ns( name_of_str("tree_of_list"), 
          move || tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let sorted : List<_> = 
      ns( name_of_str("mergesort"),
          move || mergesort_list_of_tree2(tree, None));
    let sorted2 = sorted.clone();
    let tree2 = // Demand the output of mergesort (making it "eager")
      ns ( name_of_str("tree_of_list2"),
           move || tree_of_list::<_,_,Tree<_>,List<_>>(Dir2::Left,sorted) );
    // ns ( name_of_str("list_of_tree"),
    //      move || list_of_tree(tree2, Dir2::Left ) )
    sorted2
  }
}

impl Compute<List<Pt2D>,List<Pt2D>> for Quickhull {
  fn compute(inp:List<Pt2D>) -> List<Pt2D> {
    //panic!("TODO")
    inp
  }
}

#[macro_export]
macro_rules! testcomputer {
  ( $name:expr, $inp:ty, $editst:ty, $out:ty, $dist:ty, $comp:ty ) => {{ 
    Box::new( 
      TestComputer
        ::<$inp,$editst,$out,$dist,$comp>
      { 
        identity:$name,
        input:PhantomData,
        editst:PhantomData,
        output:PhantomData,
        inputdist:PhantomData,
        computer:PhantomData
      }) 
  }}
}


/// This is the master list of all tests in the current Adapton Lab
pub fn all_tests() -> Vec<Box<LabDef>> {
  return vec![
    testcomputer!(name_of_str("list-lazy-map"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMap)
      ,
    testcomputer!(name_of_str("list-lazy-filter"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyFilter)
      ,
    testcomputer!(name_of_str("list-tree-max"),
                  List<usize>, usize,
                  usize,
                  UniformPrepend<_,_>,
                  ListTreeMax)
      ,
    testcomputer!(name_of_str("list-tree-sum"),
                  List<usize>, usize,
                  usize,
                  UniformPrepend<_,_>,
                  ListTreeSum)
      ,
    testcomputer!(name_of_str("eager-mergesort"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMergesort)
      ,
    testcomputer!(name_of_str("lazy-mergesort"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  LazyMergesort)
      ,
    testcomputer!(name_of_str("list-eager-map"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerMap)
      ,
    testcomputer!(name_of_str("list-eager-filter"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  EagerFilter)
      ,
    testcomputer!(name_of_str("list-reverse"),
                  List<usize>, usize,
                  List<usize>,
                  UniformPrepend<_,_>,
                  ListReverse)
      ,
    // testcomputer!(name_of_str("list-quickhull"),
    //               List<Pt2D>, usize,
    //               List<Pt2D>,
    //               UniformPrepend<_,_>,
    //               Quickhull)
    // ,
  ]
}
