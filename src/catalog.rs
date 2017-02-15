use labdef::*;
use adapton::collections::*;
use adapton::engine::*;
use adapton::macros::*;
use rand::{Rng};
use std::marker::PhantomData;
use std::rc::Rc;
use pmfp_collections::inc_gauged_raz::{Raz,RazTree};
use pmfp_collections::inc_tree_cursor::{gen_level};


#[derive(Clone,Debug)]
pub struct UniformInsert<T,S> { t:PhantomData<T>, s:PhantomData<S> }
/// Simple example to explain Adapton's dirtying + cleaning algorithms.
///
/// This example demonstrates how, when input changes, dirtying
/// proceeds incrementally through the edges of the DCG, during
/// cleaning.  We show a particular input change for this DCG where a
/// subcomputation `h` is never dirtied nor cleaned by change
/// propagation. We show another change to the same input where this
/// subcomputation `h` *is* _eventually_ dirtied and cleaned by
/// Adapton, though not immediately.
/// 
/// ```
///   ref cell
///   inp
///   [ 2 ]
///     ^
///     | force                                                         
///     | 2                                                      
///     |                 cell                                   cell
///     |    alloc 4       b      force 4           alloc 4       c
///    (g)-------------->[ 4 ]<--------------(h)--------------->[ 4 ]
///     ^                                     ^
///     | force                               | force h,
///     | returns b                           | returns c
///     |                                     |
///    (f)------------------------------------+
///     ^
///     | force f,
///     | returns cell c
///     |
///  (root of demand)
/// ```
///
/// In this example DCG, thunk `f` allocates and forces two
/// sub-computations, thunks `g` and `h`.  The first consumes the
/// input `inp` and produces an intermediate result (ref cell `b`);
/// the second consumes this intermediate result and produces a final
/// result (ref cell `c`), which both thunks `h` and `f` return as
/// their final result.
///
/// When the input cell `inp` changes, e.g., from 2 to -2, thunks `f`
/// and `g` are dirtied.  Thunk `g` is dirty because it consumes the
/// changed input.  Thunk `f` is dirty because it demanded (consumed)
/// the output of thunk `g` in the extent of its own computation.
/// _Importantly, thunk `h` is *not* immediately dirtied when `inp`
/// changes._
///
/// In some very real sense, `inp` is an indirect ("transitive") input
/// to thunk `h`.  This fact may suggest that when `inp` is changed
/// from 2 to -2, we should dirty thunk `h` immediately.  However,
/// thunk `h` is related to this input only by reading a *different*
/// ref cell (ref cell b) that dependents, indirectly, on `inp`, via
/// the behavior of thunk `g`, on which thunk `h` does *not* directly
/// depend (e.g., thunk `h` does not force thunk `g`).
///
/// Rather, when thunk `f` is re-demanded (from the "root of demand",
/// maybe a larger DCG or a user), it will necessarily perform a
/// cleaning process (aka, "change propagation"), re-executing `g`,
/// its immediate dependent, which is dirty.  Since thunk `g` merely
/// squares its input, and 2 and -2 both square to 4, the output of
/// thunk `g` will not change in this case.  Consequently, the
/// consumers of cell `b`, which holds this output, will not be
/// dirtied or re-executed.  In this case, thunk `h` is this consumer.
/// In situations like these, Adapton's dirtying + cleaning algorithms
/// do not dirty nor clean thunk `h`.  (For some other change, e.g.,
/// from 2 to 3, thunk `h` would _eventually_ be dirtied and cleaned).
///
/// In sum, under this change, after `f` is re-demanded, the cleaning
/// process will first re-execute `g`, the immediate consumer of
/// `inp`.  Thunk `g` will again allocate cell `b` to hold 4, the same
/// value as before.  It also yields this same cell pointer (to cell
/// `b`).  Consequently, thunk `f` is not re-executed, and is cleaned.
/// Meanwhile, the outgoing (dependency) edges thunk of `h` are never
/// dirited.
///
#[derive(Clone,Debug)]
pub struct ExampleCleanDirty {}
impl Generate<Art<i32>> for ExampleCleanDirty {
  fn generate<R:Rng> (_rng:&mut R, _params:&GenerateParams) -> Art<i32> {
    cell(name_of_str("a"), 2)
  }
}
/// This editor helps in an example.  It creates an input cell holding
/// the integer 2, and then edits this cell to hold -2.  Then it edits
/// the cell to hold 3, then 2 again, and then loops (2, -2, 3, 2, ...).
impl Edit<Art<i32>, usize> for ExampleCleanDirty {
  fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 
    return 0
  }
  fn edit<R:Rng>(_inp:Art<i32>, i:usize,
                 _rng:&mut R, _params:&GenerateParams) -> (Art<i32>, usize) {
    if i == 0 {
      let inp = cell(name_of_str("a"), -2);
      (inp, 1)
    } 
    else if i == 1 {
      let inp = cell(name_of_str("a"), 3);
      (inp, 2)
    }
    else {
      let inp = cell(name_of_str("a"), 2);
      (inp, 0)
    }
  }
}
impl Compute<Art<i32>, Art<i32>> for ExampleCleanDirty {
  fn compute(inp:Art<i32>) -> Art<i32> {
    let c : Art<i32> = force 
      // thunk 'f' creates and forces thunks `g` and `h`, below:
      (& thunk![ name_of_str("f") =>> {
        let inp = inp.clone();
        
        let b : Art<i32> = force 
        // thunk `g` reads the input `inp` and writes its square (x
        // * x) to a new cell, `b`, returning the cell `b`.
          (& thunk![ name_of_str("g") =>> {
            let x = force(&inp);
            cell(name_of_str("b"), x * x)       
          }]);        
        
        let c : Art<i32> = force 
        // thunk `h` reads the output of `g`, cell `b`, and writes the
        // min of this number and 100 to another new cell, `c`,
        // returning the cell `c`.
          (& thunk![ name_of_str("h") =>> {
            let x = force(&b);
            cell(name_of_str("c"), if x < 100 { x } else { 100 })
          }]);        
        
        c
      }])
      ;
    return c
  }
}




/// This list editor mimics the editor in the example from
/// _Incremental Computation with Names_ (2015), Section 2 (Figs 1 and
/// 2).
///
/// This `generate` function creates the three-element, three-name,
/// three-ref-cell list in the first parts of Figures 1 and Figure 2.
///
/// This `edit` function implements the single insertion of `Cons(2,
/// ...)` into the generated list, as per the second parts of Figures
/// 1 and 2.  (It only performs this one edit, and performs no other
/// actions later).  This code shows two ways of inserting a new
/// element (and name and ref cell) into the list: The "functional"
/// way, and the imperative way.  The use of names here bridges the
/// gap, permitting the functional approach to express the mutation in
/// the imperative approach.
/// 
#[derive(Clone,Debug)]
pub struct EditorOopsla2015Sec2 { } 
impl Generate<List<usize>> for EditorOopsla2015Sec2 {
  fn generate<R:Rng> (_rng:&mut R, _params:&GenerateParams) -> List<usize> {
    let l = list_nil();
    
    let l = list_art( cell( name_of_str("d"), l) );
    let l = list_name( name_of_str("delta"), l );
    let l = list_cons(3, l);

    let l = list_art( cell( name_of_str("b"), l) );
    let l = list_name( name_of_str("beta"), l );
    let l = list_cons(1, l);

    let l = list_art( cell( name_of_str("a"), l) );
    let l = list_name( name_of_str("alpha"), l );
    let l = list_cons(0, l);

    let l = list_art( cell( name_of_str("z"), l) );
    let l = list_name( name_of_str("zeta"), l );
    let l = list_cons(99, l);

    let l = list_art( cell( name_of_str("y"), l) );
    let l = list_name( name_of_str("yellow"), l );
    let l = list_cons(98, l);

    let l = list_art( cell( name_of_str("x"), l) );
    let l = list_name( name_of_str("xray"), l );
    let l = list_cons(97, l);
    l
  }
}
impl Edit<List<usize>,usize> for EditorOopsla2015Sec2 {
  fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 
    return 0
  }
  fn edit<R:Rng>(list:List<usize>, i:usize,
                 _rng:&mut R, _params:&GenerateParams) -> (List<usize>, usize) {
    if i == 0 {
      let x = match list      { List::Cons(_, box List::Name(_, box List::Art(ref x))) => x.clone(), _ => unreachable!() };
      let y = match force(&x) { List::Cons(_, box List::Name(_, box List::Art(ref y))) => y.clone(), _ => unreachable!() };
      let z = match force(&y) { List::Cons(_, box List::Name(_, box List::Art(ref z))) => z.clone(), _ => unreachable!() };
      let a = match force(&z) { List::Cons(_, box List::Name(_, box List::Art(ref a))) => a.clone(), _ => unreachable!() };
      let b = match force(&a) { List::Cons(_, box List::Name(_, box List::Art(ref b))) => b.clone(), _ => unreachable!() };
      let l = force(&b);

      // Create the new Cons cell, new name and new ref cell, which
      // points at the tail of the existing list, `l`, above.
      let l = list_art(cell(name_of_str("c"), l));
      let l = list_name(name_of_str("gamma"), l);
      let l = list_cons(2, l);
      
      // The following ways of mutating cell b are equivalent for the
      // DCG, though only the first way is defined for the Naive
      // engine:
      if true {
        let l = list_art(cell( name_of_str("b"), l));
        
        // The rest of this is copied from the Generate impl.  We have
        // to do these steps to keep the Naive version (which does not
        // have a store) in sync with the DCG's input (which need not do
        // these steps):
        let l = list_name( name_of_str("beta"), l );
        let l = list_cons(1, l);      
        let l = list_art( cell( name_of_str("a"), l) );
        let l = list_name( name_of_str("alpha"), l );
        let l = list_cons(0, l);
        
        let l = list_art( cell( name_of_str("z"), l) );
        let l = list_name( name_of_str("zeta"), l );
        let l = list_cons(99, l);
        
        let l = list_art( cell( name_of_str("y"), l) );
        let l = list_name( name_of_str("yellow"), l );
        let l = list_cons(98, l);

        let l = list_art( cell( name_of_str("x"), l) );
        let l = list_name( name_of_str("xray"), l );
        let l = list_cons(97, l);

        return (l, 1)
      } else {
        // DCG only: The `set` operation is not supported by Naive
        // computation, since in the Naive computation, articulations
        // are just (immutable) reference cells holding values or
        // suspended computations.
        set(&b, l);
        return (list, i);
      }
    }
    else {
      // No more changes.
      (list, i)
    }
  }
}

/// Program `list_map` running on a small, changing input list.
/// It is simpler than the version from the Adapton collections library.
///
/// This `list_map` implementation and its `List<_>` datatype follow
/// the code listing in Section 2 of _Incremental Computation with
/// Names_ (OOPSLA 2015).
///
/// _Aside_: Compared to the version of `List<_>` in the Adapton
/// collections library, this version is simpler: It assumes only two
/// constructors, `Nil` and `Cons`, and that every list element has an
/// associated name and reference cell.
///
/// This `generate` function creates the three-element, three-name,
/// three-ref-cell list in the first parts of Figures 1 and Figure 2.
///
/// This `edit` function implements the single insertion of `Cons(2,
/// ...)` into the generated list, as per the second parts of Figures
/// 1 and 2.  (It only performs this one edit, and performs no other
/// actions later).  This code shows two ways of inserting a new
/// element (and name and ref cell) into the list: The "functional"
/// way, and the imperative way.  The use of names here bridges the
/// gap, permitting the functional approach to express the mutation in
/// the imperative approach.
/// 
pub mod oopsla2015_sec2 {
  use super::*;
  use std::hash::Hash;
  use std::fmt::Debug;
  use std::rc::Rc;
  //use adapton::macros::* ;
  //use adapton::engine::* ;
  
  /// `Cons` cells carry an element, name and reference cell for the rest of the list.
  #[derive(Debug,PartialEq,Eq,Hash,Clone)]
  pub enum List<X> {
    Nil,
    Cons(X, Name, Art<List<X>>)
  }

  /// The _Editor_ in this example generates a three-element initial list, then inserts an additional element.
  #[derive(Clone,Debug)]
  pub struct Editor { } 

  /// The _Archivist_ in this example maps the input list to an output list, using the names of the input list.
  #[derive(Clone,Debug)]
  pub struct Archivist { } 
  
  /// List map, as shown in 'Incremental Computation with Names',
  /// Section 2:
  pub fn list_map<X:Eq+Clone+Hash+Debug+'static,
                  Y:Eq+Clone+Hash+Debug+'static,
                  F:'static>
    (inp: List<X>, f:Rc<F>) -> List<Y> 
    where F:Fn(X) -> Y 
  {
    match inp {
      List::Nil => List::Nil,
      List::Cons(x, nm, xs) => {
        memo!(nm.clone() =>> list_map_cons =>> <X,Y,F>, 
              x:x, nm:nm, xs:xs 
              ;; 
              f:f)
      }
    }
  }

  /// This is the code that we memoize each time we see a name in a
  /// `Cons` cell.  We identify this memo point using the names from
  /// the list.
  pub fn list_map_cons<X:Eq+Clone+Hash+Debug+'static,
                       Y:Eq+Clone+Hash+Debug+'static,
                       F:'static>
    (x:X, nm:Name, xs: Art<List<X>>, f:Rc<F>) -> List<Y> 
    where F:Fn(X) -> Y 
  {    
    let (nm1, nm2) = name_fork(nm);
    let y = f(x);
    let rest = list_map(force(&xs), f);
    List::Cons(y, nm1, cell(nm2, rest))
  }

  impl Compute<List<usize>, List<usize>> for Archivist {
    fn compute(inp:List<usize>) -> List<usize> { list_map(inp, Rc::new(|x| x * x)) }
  }

  impl Generate<List<usize>> for Editor {
    fn generate<R:Rng> (_rng:&mut R, _params:&GenerateParams) -> List<usize> {
      let l = List::Nil;
      let l = List::Cons(3, name_of_str("delta"), cell(name_of_str("d"), l));
      let l = List::Cons(1, name_of_str("beta"), cell(name_of_str("b"), l));
      let l = List::Cons(0, name_of_str("alpha"), cell(name_of_str("a"), l));
      l
    }
  }
  impl Edit<List<usize>,usize> for Editor {
    fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 
      return 0
    }
    fn edit<R:Rng>(list:List<usize>, i:usize,
                   _rng:&mut R, _params:&GenerateParams) -> (List<usize>, usize) {
      if i == 0 {
        let a = match list.clone() { List::Cons(_, _, a) => a.clone(), _ => unreachable!() };
        let b = match force(&a)    { List::Cons(_, _, b) => b.clone(), _ => unreachable!() };
        let l = force(&b);
        
        // Create the new Cons cell, new name and new ref cell, which
        // points at the tail of the existing list, `b`, above.
        let l = List::Cons(2, name_of_str("gamma"), cell(name_of_str("c"), l));
        
        // The following ways of mutating cell b are equivalent for the
        // DCG, though only the first way is defined for the Naive
        // engine:
        if true {
          // Mutate the cell called 'b' to hold this new list:
          let l = cell(name_of_str("b"), l);
          
          // The rest of this is copied from the Generate impl.  We have
          // to do these steps to keep the Naive version (which does not
          // have a store) in sync with the DCG's input (which need not do
          // these steps):        
          let l = List::Cons(1, name_of_str("beta"), l);
          let l = List::Cons(0, name_of_str("alpha"), cell(name_of_str("a"), l));
          
          return (l, 1)
        } else {
          // DCG only: The `set` operation is not supported by Naive
          // computation, since in the Naive computation, articulations
          // are just (immutable) reference cells holding values or
          // suspended computations.
          set(&b, l);
          return (list, i);
        }
      }
      else {
        // No more changes.
        (list, i)
      }
    }
  }
}


/// Hammer - CSCI 7000, Spring 2017
/// ==============================
///
/// First homework assignment: #HW0
/// -------------------------------
/// 
/// There are three functions below whose bodies consist of
/// `panic!("TODO")`.  Using the types listed in their declarations,
/// implement these functions.  You will find the `list_map` example
/// from the `oopsla2015_sec2` module helpful, as a guide.
///
/// Test the behavior of your filter function, for instance:
/// 
/// ```
/// cargo run -- -L hammer-s17-hw0-filter
/// ```
/// 
/// 
pub mod hammer_s17_hw0 {
  use super::*;
  use std::hash::Hash;
  use std::fmt::Debug;
  use std::rc::Rc;

  /// `Cons` cells carry an element, name and reference cell for the rest of the list.
  #[derive(Debug,PartialEq,Eq,Hash,Clone)]
  pub enum List<X> {
    Nil,
    Cons(X, Name, Art<List<X>>)
  }

  /// List filter:
  pub fn list_filter<X:Eq+Clone+Hash+Debug+'static,
                     F:'static>
    (inp: List<X>, f:Rc<F>) -> List<X> 
    where F:Fn(X) -> bool
  {
    panic!("TODO")
  }

  /// List split:
  pub fn list_split<X:Eq+Clone+Hash+Debug+'static,
                    F:'static>
    (inp: List<X>, f:Rc<F>) -> (List<X>, List<X>)
    where F:Fn(X) -> bool
  {
    panic!("TODO")
  }

  /// List reverse:
  pub fn list_reverse<X:Eq+Clone+Hash+Debug+'static>
    (inp: List<X>) -> List<X>
  {
    panic!("TODO")
  }


  #[derive(Clone,Debug)]
  pub struct RunFilter { } 
  impl Compute<List<usize>, List<usize>> for RunFilter {
    fn compute(inp:List<usize>) -> List<usize> { list_filter(inp, Rc::new(|x| x % 2 == 0)) }
  }

  #[derive(Clone,Debug)]
  pub struct RunSplit { } 
  impl Compute<List<usize>, (List<usize>, List<usize>)> for RunSplit {
    fn compute(inp:List<usize>) -> (List<usize>,List<usize>) { list_split(inp, Rc::new(|x| x % 2 == 0)) }
  }

  #[derive(Clone,Debug)]
  pub struct RunReverse { } 
  impl Compute<List<usize>, List<usize>> for RunReverse {
    fn compute(inp:List<usize>) -> List<usize> { list_reverse(inp) }
  }  

  /// The _Editor_ in this example generates a three-element initial list, then inserts an additional element.
  /// It's the same as `oopsla2015_sec2::Editor`.
  #[derive(Clone,Debug)]
  pub struct Editor { } 

  impl Generate<List<usize>> for Editor {
    fn generate<R:Rng> (_rng:&mut R, _params:&GenerateParams) -> List<usize> {
      let l = List::Nil;
      let l = List::Cons(3, name_of_str("delta"), cell(name_of_str("d"), l));
      let l = List::Cons(1, name_of_str("beta"), cell(name_of_str("b"), l));
      let l = List::Cons(0, name_of_str("alpha"), cell(name_of_str("a"), l));
      l
    }
  }
  impl Edit<List<usize>,usize> for Editor {
    fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 
      return 0
    }
    fn edit<R:Rng>(list:List<usize>, i:usize,
                   _rng:&mut R, _params:&GenerateParams) -> (List<usize>, usize) {
      if i == 0 {
        let a = match list.clone() { List::Cons(_, _, a) => a.clone(), _ => unreachable!() };
        let b = match force(&a)    { List::Cons(_, _, b) => b.clone(), _ => unreachable!() };
        let l = force(&b);
        
        // Create the new Cons cell, new name and new ref cell, which
        // points at the tail of the existing list, `b`, above.
        let l = List::Cons(2, name_of_str("gamma"), cell(name_of_str("c"), l));
        
        // The following ways of mutating cell b are equivalent for the
        // DCG, though only the first way is defined for the Naive
        // engine:
        if true {
          // Mutate the cell called 'b' to hold this new list:
          let l = cell(name_of_str("b"), l);
          
          // The rest of this is copied from the Generate impl.  We have
          // to do these steps to keep the Naive version (which does not
          // have a store) in sync with the DCG's input (which need not do
          // these steps):        
          let l = List::Cons(1, name_of_str("beta"), l);
          let l = List::Cons(0, name_of_str("alpha"), cell(name_of_str("a"), l));
          
          return (l, 1)
        } else {
          // DCG only: The `set` operation is not supported by Naive
          // computation, since in the Naive computation, articulations
          // are just (immutable) reference cells holding values or
          // suspended computations.
          set(&b, l);
          return (list, i);
        }
      }
      else {
        // No more changes.
        (list, i)
      }
    }
  }
}

impl<S> Generate<RazTree<usize>> for UniformInsert<RazTree<usize>, S> {
  fn generate<R:Rng> (rng:&mut R, params:&GenerateParams) -> RazTree<usize> {
    let mut r = Raz::new();
    let mut n = 0;
    for i in 0..params.size {
      if i % params.gauge == 0 {
        r.archive_left( gen_level(rng),  Some(name_of_usize(n)));
        n += 1;
      } else { } ;
      // use random data (numbers 1000-1999 )
      // r.push_left( rng.gen::<usize>() % 1000 + 1000 );
      // use the insertion count, marked by adding a million
      r.push_left(i + 1_000_000);
    }
    r.unfocus()
  }
}

impl Edit<RazTree<usize>, usize> for UniformInsert<RazTree<usize>, usize> {
  fn edit_init<R:Rng>(_rng:&mut R, params:&GenerateParams) -> usize { 
    return params.size // Initial editing state = The size of the generated input
  }
  fn edit<R:Rng>(tree:RazTree<usize>, i:usize,
                 rng:&mut R, _params:&GenerateParams) -> (RazTree<usize>, usize) {
    let t = tree;
    let pos = rng.gen::<usize>() % ( i + 1 );
    let mut r = t.focus( pos ).unwrap();
    if i % _params.gauge == 0 {
      r.archive_left( gen_level(rng), Some(name_of_usize(i)) );
    } else { } ;
    // use random data (numbers 1000-1999 )
    // r.push_left( rng.gen::<usize>() % 1000 + 1000 );
    // use the insertion count, marked by adding a million
    r.push_left( i + 1_000_000 );
    let t = r.unfocus();    
    (t, i+1)
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
      let elm = elm % ( params.size * 100 ) ;
      l = list_cons(elm,  l);
      if i % params.gauge == 0 {
        //l = list_art(cell(name_of_usize(i), l));
        //l = list_name(name_of_usize(i), l);
      } else { } ;
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
    let elm = elm % ( params.size * 100 ) ;
    l = list_cons(elm, l);
    if i % params.gauge == 0 {
      //l = list_art(cell(name_of_usize(i), l));
      //l = list_name(name_of_usize(i), l);      
    } else { } ;
    (l, i + 1)
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
pub struct EagerMap2 { }
#[derive(Clone,Debug)]
pub struct SimpEagerMap { }

#[derive(Clone,Debug)]
pub struct LazyFilter { }
#[derive(Clone,Debug)]
pub struct EagerFilter { }

#[derive(Clone,Debug)]
pub struct ListTree { }
#[derive(Clone,Debug)]
pub struct ListTreeMax { }
#[derive(Clone,Debug)]
pub struct ListTreeSum { }

#[derive(Clone,Debug)]
pub struct ListReverse { }

#[derive(Clone,Debug)]
pub struct LazyMergesort3 { }
#[derive(Clone,Debug)]
pub struct EagerMergesort3 { }

#[derive(Clone,Debug)]
pub struct LazyMergesort2 { }
#[derive(Clone,Debug)]
pub struct EagerMergesort2 { }

#[derive(Clone,Debug)]
pub struct LazyMergesort1 { }
#[derive(Clone,Debug)]
pub struct EagerMergesort1 { }

#[derive(Clone,Debug)]
pub struct Quickhull { }

#[derive(Clone,Debug)]
pub struct RazMax {}

#[derive(Clone,Debug)]
pub struct RazDouble {}

/// Native Rust lab that finds the maximum random integer in an array.
#[derive(Clone,Debug)]
pub struct VecMax { }
impl Compute<Vec<usize>, usize> for VecMax {
  fn compute(inp:Vec<usize>) -> usize {
    *(inp.iter().max()).unwrap()
  }
}
impl Generate<Vec<usize>> for VecMax {
  fn generate<R:Rng> (rng:&mut R, params:&GenerateParams) -> Vec<usize> {
    let mut v = vec![];
    for _i in 0..params.size-1 {
      let j : usize = rng.gen();
      v.push( j );
    }
    return v
  }
}
impl Edit<Vec<usize>, usize> for VecMax {
  fn edit_init<R:Rng>(_rng:&mut R, _params:&GenerateParams) -> usize { 0 }
  fn edit<R:Rng>(inp:Vec<usize>, i:usize,
                 _rng:&mut R, _params:&GenerateParams) -> (Vec<usize>, usize) { 
    (inp, i+1) 
  }
}


impl Compute<List<usize>,List<usize>> for EagerMap {
  fn compute(inp:List<usize>) -> List<usize> {
    list_map_eager(inp,Rc::new(|x| x * x))
  }
}

impl Compute<List<usize>,List<usize>> for EagerMap2 {
  fn compute(inp:List<usize>) -> List<usize> {
    list_map_eager2(inp,Rc::new(|x| x * x))
  }
}

impl Compute<List<usize>,List<usize>> for EagerFilter {
  fn compute(inp:List<usize>) -> List<usize> {
    list_filter_eager(inp,Rc::new(|x:&usize| (*x) % 3 == 0))
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyMap {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {
    let out : List<usize> = list_map_lazy(inp,Rc::new(|x| x * x));
    list_demand( out.clone(), demand );
    out
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyFilter {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {
    let out : List<usize> = 
      list_filter_lazy(inp,Rc::new(|x:&usize| (*x) % 3 == 0));
    drop( list_demand( out.clone(), demand) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for ListReverse {
  fn compute(inp:List<usize>) -> List<usize> {
    list_reverse(inp, list_nil())
  }
}

impl Compute<List<usize>,Tree<usize>> for ListTree {
  fn compute(inp:List<usize>) -> Tree<usize> {
    tree_of_list(Dir2::Left,inp)
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

impl ComputeDemand<List<usize>,List<usize>> for LazyMergesort3 {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let out : List<usize> = mergesort_list_of_tree2(tree,None);
    drop( list_demand( out.clone(), demand ) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort3 {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = 
      ns( name_of_str("tree_of_list"), 
          move || tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let sorted : List<_> = 
      ns( name_of_str("mergesort"),
          move || mergesort_list_of_tree3(tree, None));
    let sorted2 = sorted.clone();
    let tree2 = // Demand the output of mergesort (making it "eager")
      ns ( name_of_str("tree_of_list2"),
           move || tree_of_list::<_,_,Tree<_>,List<_>>(Dir2::Left,sorted) );
    // ns ( name_of_str("list_of_tree"),
    //      move || list_of_tree(tree2, Dir2::Left ) )
    drop(tree2);
    sorted2
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyMergesort2 {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let out : List<usize> = mergesort_list_of_tree2(tree,None);
    drop( list_demand( out.clone(), demand) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort2 {
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
    drop(tree2);
    sorted2
  }
}

impl ComputeDemand<List<usize>,List<usize>> for LazyMergesort1 {
  fn compute(inp:List<usize>, demand:usize) -> List<usize> {    
    let tree = ns( name_of_str("tree_of_list"), 
                   move ||tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let out : List<usize> = mergesort_list_of_tree(tree);
    drop( list_demand( out.clone(), demand) );
    out
  }
}

impl Compute<List<usize>,List<usize>> for EagerMergesort1 {
  fn compute(inp:List<usize>) -> List<usize> {
    let tree = 
      ns( name_of_str("tree_of_list"), 
          move || tree_of_list::<usize,usize,Tree<_>,_>(Dir2::Right,inp) );
    let sorted : List<_> = 
      ns( name_of_str("mergesort"),
          move || mergesort_list_of_tree(tree));
    let sorted2 = sorted.clone();
    let tree2 = // Demand the output of mergesort (making it "eager")
      ns ( name_of_str("tree_of_list2"),
           move || tree_of_list::<_,_,Tree<_>,List<_>>(Dir2::Left,sorted) );
    // ns ( name_of_str("list_of_tree"),
    //      move || list_of_tree(tree2, Dir2::Left ) )
    drop(tree2);
    sorted2
  }
}

impl Compute<List<Pt2D>,List<Pt2D>> for Quickhull {
  fn compute(inp:List<Pt2D>) -> List<Pt2D> {
    //panic!("TODO")
    inp
  }
}

impl Compute<RazTree<usize>,usize> for RazMax {
  fn compute(inp:RazTree<usize>) -> usize {
    let max = inp.fold_up(Rc::new(|e:&usize|*e),Rc::new(|e1:usize,e2:usize|::std::cmp::max(e1,e2)));
    max.unwrap_or(0)
  }
}

impl Compute<RazTree<usize>,RazTree<usize>> for RazDouble {
  fn compute(inp:RazTree<usize>) -> RazTree<usize> {
    inp.map(Rc::new(|e:&usize|*e*2))
  }
}

#[macro_export]
macro_rules! labdef {
  ( $name:expr, $url:expr, $inp:ty, $editst:ty, $out:ty, $dist:ty, $comp:ty ) => {{ 
    Box::new( 
      LabDef
        ::<$inp,$editst,$out,$dist,$comp>
      { 
        identity:$name,
        url:$url,
        input:PhantomData,
        editst:PhantomData,
        output:PhantomData,
        editor:PhantomData,
        archivist:PhantomData
      }) 
  }}
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - 
/// `all_labs`: This is the master list of all labs in the current
/// Adapton Lab catalog (which consists of this module, but which
/// wraps the Adapton crate's collections module).
///
/// To add a new lab, just add a `labdef!` to the `vec!` in
/// this definition.  Doing so generally requires doing at least the
/// following:
/// 
/// 1. Add a new (empty struct) type that implements the `Compute` or
/// `ComputeDemand` trait for some input- and output-type pair.
///
/// 2. The input type must be a type parameter to some implementation
/// of the `Generate` and `Edit` traits, which you must also provide
/// to `labdef!`.  Together, these trait implementations give
/// the distribution of the input (how it is chosen, and how it
/// changes, respectively).
///
/// 3. The other arguments consist of the type of the `Editor` state
/// (e.g., a counter of type `usize` in many cases), and a name and
/// url to display in generated output.  The URL should link to the
/// rustdoc for this module, which in turn provides other related
/// documentation about Adapton Lab and Adapton.
// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - 
pub fn all_labs() -> Vec<Box<Lab>> {
  return vec![
    labdef!(name_of_str("eg-clean-dirty"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ExampleCleanDirty.html")),
            Art<i32>, usize,
            Art<i32>,
            ExampleCleanDirty,
            ExampleCleanDirty)
      ,

    labdef!(name_of_str("eg-oopsla2015-sec2"),
            Some(String::from("")),
            oopsla2015_sec2::List<usize>, usize,
            oopsla2015_sec2::List<usize>,
            oopsla2015_sec2::Editor,
            oopsla2015_sec2::Archivist)
      ,

    labdef!(name_of_str("eg-oopsla2015-sec2-rev1"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EditorOopsla2015Sec2.html")),
            List<usize>, usize,
            List<usize>,
            EditorOopsla2015Sec2,
            EagerMap)
      ,
    labdef!(name_of_str("eg-oopsla2015-sec2-rev2"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EditorOopsla2015Sec2.html")),
            List<usize>, usize,
            List<usize>,
            EditorOopsla2015Sec2,
            EagerMap2)
      ,


    labdef!(name_of_str("list-lazy-map"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMap.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            LazyMap)
      ,
    labdef!(name_of_str("list-lazy-filter"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyFilter.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            LazyFilter)
      ,
    
    
    labdef!(name_of_str("list-tree"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListTree.html")),
            List<usize>, usize,
            Tree<usize>,
            UniformPrepend<_,_>,
            ListTree)
      ,
    labdef!(name_of_str("list-tree-max"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListTreeMax.html")),
            List<usize>, usize,
            usize,
            UniformPrepend<_,_>,
            ListTreeMax)
      ,
    labdef!(name_of_str("list-tree-sum"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListTreeSum.html")),
            List<usize>, usize,
            usize,
            UniformPrepend<_,_>,
            ListTreeSum)
      ,
    
    labdef!(name_of_str("list-eager-mergesort3"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMergesort3.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            EagerMergesort3)
      ,
    labdef!(name_of_str("list-lazy-mergesort3"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMergesort3.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            LazyMergesort3)
      ,
    
    labdef!(name_of_str("list-eager-mergesort2"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMergesort2.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            EagerMergesort2)
      ,
    labdef!(name_of_str("list-lazy-mergesort2"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMergesort2.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            LazyMergesort2)
      ,
    
    labdef!(name_of_str("list-eager-mergesort1"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMergesort1.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            EagerMergesort1)
      ,
    labdef!(name_of_str("list-lazy-mergesort1"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.LazyMergesort1.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            LazyMergesort1)
      ,
    
    labdef!(name_of_str("list-eager-map"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerMap.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            EagerMap)
      ,
    labdef!(name_of_str("list-eager-filter"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.EagerFilter.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            EagerFilter)
      ,
    labdef!(name_of_str("list-reverse"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.ListReverse.html")),
            List<usize>, usize,
            List<usize>,
            UniformPrepend<_,_>,
            ListReverse)
      ,
    labdef!(name_of_str("raz-max"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.RazMax.html")),
            RazTree<usize>, usize,
            usize,
            UniformInsert<_,_>,
            RazMax)
      ,
    labdef!(name_of_str("vec-max"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.VecMax.html")),
            Vec<usize>, usize,
            usize,
            VecMax,
            VecMax)
      ,
    labdef!(name_of_str("raz-double"),
            Some(String::from("http://adapton.org/rustdoc/adapton_lab/catalog/struct.RazDouble.html")),
            RazTree<usize>, usize,
            RazTree<usize>,
            UniformInsert<_,_>,
            RazDouble)
      ,
    // labdef!(name_of_str("list-quickhull"),
    //               List<Pt2D>, usize,
    //               List<Pt2D>,
    //               UniformPrepend<_,_>,
    //               Quickhull)
    // ,

    labdef!(name_of_str("hammer-s17-hw0-filter"),
            Some(String::from("")),
            hammer_s17_hw0::List<usize>, usize,
            hammer_s17_hw0::List<usize>,
            hammer_s17_hw0::Editor,
            hammer_s17_hw0::RunFilter)
      ,

    labdef!(name_of_str("hammer-s17-hw0-split"),
            Some(String::from("")),
            hammer_s17_hw0::List<usize>, usize,
            (hammer_s17_hw0::List<usize>, hammer_s17_hw0::List<usize>),
            hammer_s17_hw0::Editor,
            hammer_s17_hw0::RunSplit)
      ,

    labdef!(name_of_str("hammer-s17-hw0-reverse"),
            Some(String::from("")),
            hammer_s17_hw0::List<usize>, usize,
            hammer_s17_hw0::List<usize>,
            hammer_s17_hw0::Editor,
            hammer_s17_hw0::RunReverse)
      ,

  ]
}
