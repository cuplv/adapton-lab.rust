use adapton::engine::reflect::*;

/// The `Div` struct represents a restricted form of a `<div>` element
/// in HTML.  The field `tag` is a string, which corresponds to a a
/// distinguished `tag` CSS class indicates the Rust datatype
/// reflected into this `Div`.  The other CSS `classes` hold bits that
/// signal various subcases (e.g., of `enum`s in the `reflect`
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

// 
// Notes:
// use std::fs;
// try!(fs::create_dir_all("/some/dir"));

pub fn write() {
  use std::io;
  use std::io::prelude::*;
  use std::io::BufWriter;
  use std::fs::File;

  pub trait WriteHTML {
    fn write_html<Wr:Write>(&self, wr: &mut Wr) {
      writeln!(wr, "foo");
    }
  }
  
  let f = File::create("foo.txt").unwrap();
  {
    let mut writer = BufWriter::new(f);
    
    // write a byte to the buffer
    writeln!(writer, "hello {:?}", 1 + 2 + 3);
    
  } // the buffer is flushed once writer goes out of scope
}
