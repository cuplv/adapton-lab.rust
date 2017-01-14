use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use adapton::engine::Name;
use adapton::engine::reflect::*;

use labdef::{LabResults};

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


pub trait WriteHTML {
  fn write_html<Wr:Write>(&self, wr: &mut Wr);
}

impl WriteHTML for Div {
  fn write_html<Wr:Write>(&self, wr: &mut Wr) {    
    writeln!(wr, "<div class=\"{:?} {:?}\">", 
             self.tag, 
             self.classes.iter().fold(
               String::new(), 
               |mut cs,c|{cs.push_str(" ");
                          cs.push_str(c.as_str()); cs}
             )
    );
    for div in self.extent.iter() {
      div.write_html(wr);
    }
    writeln!(wr, "</div>");
  }
}

pub fn write_test_results(testname:Name, results:&LabResults) {
  
  // For linking to rustdoc documentation from the output HTML
  let trace_url = "http://adapton.org/rustdoc/adapton/engine/reflect/trace/struct.Trace.html";
  
  fs::create_dir_all("lab-results").unwrap();
  let f = File::create(format!("lab-results/{:?}.html", testname)).unwrap();
  let mut writer = BufWriter::new(f);

  writeln!(writer, "{}", style_string()).unwrap();
  writeln!(writer, "<div class={:?}>{:?}</div>", "test-name", testname).unwrap();

  for sample in results.samples.iter() {
    writeln!(writer, "<div class=\"batch-name-lab\">batch name<div class=\"batch-name\">{:?}</div></div>", 
             sample.batch_name).unwrap();
    
    writeln!(writer, "<div class=\"editor\">").unwrap();
    writeln!(writer, "<div class=\"time-ns-lab\">time (ns)<div class=\"time-ns\">{:?}</div></div>", 
             sample.dcg_sample.process_input.time_ns).unwrap();    
    writeln!(writer, "<div class=\"traces-lab\">Traces (<a href={:?}>doc</a>)<div class=\"traces\">{:?}</div></div>", 
             trace_url,
             sample.dcg_sample.process_input.dcg_traces).unwrap();
    writeln!(writer, "</div>").unwrap();
    
    // - - - - - - - 
    
    writeln!(writer, "<div class=\"archivist\">").unwrap();
    writeln!(writer, "<div class=\"time-ns-lab\">time (ns)<div class=\"time-ns\">{:?}</div></div>", 
             sample.dcg_sample.compute_output.time_ns).unwrap();    
    writeln!(writer, "<div class=\"traces-lab\">Traces (<a href={:?}>doc</a>)<div class=\"traces\">{:?}</div></div>", 
             trace_url,
             sample.dcg_sample.compute_output.dcg_traces).unwrap();
    writeln!(writer, "</div>").unwrap();    
    writeln!(writer, "</div>").unwrap();

    writeln!(writer, "<hr/>").unwrap();
   
  }
  writer.flush().unwrap();  
}

pub fn style_string() -> &'static str {
"
<style>
hr {
  float: left;
}

.test-name {
  font-size: 66px;
  font-family: sans-serif;
}
.batch-name-lab {
  font-size: 0px;
}
.time-ns {
  font-size: 20px;
  font-family: sans-serif;
}
.batch-name {
  font-size: 20px;
  border: solid;
  display: inline;
  padding: 7px;
  margin: 5px;
  float: left;
}
.traces {
  font-size: 8px;
  border: solid 1px;
  display: inline;
  padding: 7px;
  margin: 5px;
  float: left;
  width: 98%;
  background: #dddddd;
}
.editor {
  font-size: 14px;
  border: solid;
  display: inline;
  padding: 7px;
  margin: 5px;
  float: left;
  width: 98%;
  background: #dddddd;
}
.archivist {
  font-size: 14px;
  border: solid;
  display: inline;
  padding: 7px;
  margin: 5px;
  float: left;
  width: 98%;
  background: #dddddd;
}
</style>
"
}
