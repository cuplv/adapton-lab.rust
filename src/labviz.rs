use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use adapton::engine::Name;
use adapton::engine::reflect::*;
use adapton::engine::reflect::trace;

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
  pub text:    Option<String>,
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

pub fn div_of_name (n:&Name) -> Div {
  Div{ tag: String::from("name"),
       // TODO: Remove illegal chars for CSS classes (check spec)
       // classes: vec![ format!("{:?}", n) ],
       classes: vec![ ],
       extent: Box::new( vec![ ] ),
       text: Some( format!("{:?}", n) ) }
}

pub fn div_of_path (p:&Path) -> Div {
  Div{ tag: String::from("path"),
       //classes: vec![ format!("{:?}", p) ],
       classes: vec![ ],
       extent: Box::new(
         p.iter().map( div_of_name ).collect()
       ),
       text: None }
}

pub fn div_of_loc (l:&Loc) -> Div {
  Div{ tag: String::from("loc"),
       // TODO: Remove illegal chars for CSS classes (check spec)
       //classes: vec![ format!("{:?}", l) ],
       classes: vec![ ],       
       extent: Box::new(vec![ div_of_path(&l.path), div_of_name(&l.name) ]),
       //text: Some( format!("{:?}",l) )
       text:None,
  }
}

pub fn div_of_oploc (ol:&Option<Loc>) -> Div {
  if true {
    Div{ tag: String::from("oploc"), 
         classes: vec![],
         extent: Box::new(vec![]),
         text: None,
    }
  } else {
    Div{ tag: String::from("oploc"),
         classes: vec![],
         extent: Box::new(match *ol { 
           None => vec![],
           Some(ref l) => vec![ div_of_loc(l)]}),
         text: None
    }
  }
}

pub fn div_of_succ (s:&Succ) -> Div {
  Div{ tag: String::from("succ"),
       classes: vec![
         String::from(match s.effect {
           Effect::Alloc => "succ-alloc",
           Effect::Force => "succ-force"
         }),
         String::from(match s.dirty {
           true  => "succ-dirty",
           false => "succ-not-dirty"
         }),
       ],
       text: None,
       extent: Box::new(vec![
         div_of_loc(&s.loc),
       ])}
}

pub fn div_of_edge (e:&trace::Edge) -> Div {
  Div{ tag: String::from("edge"),
       classes: vec![],
       text: None,
       extent: Box::new(
         vec![ div_of_oploc(&e.loc),
               div_of_succ(&e.succ) ]) }
}

pub fn div_of_trace (tr:&trace::Trace) -> Div {
  let mut div = 
    Div{ 
      tag: String::from("trace"),
      text: None,
      classes: vec![
        String::from(match tr.effect {
          trace::Effect::CleanRec  => "tr-clean-rec",
          trace::Effect::CleanEval => "tr-clean-eval",
          trace::Effect::CleanEdge => "tr-clean-edge",
          trace::Effect::Dirty     => "tr-dirty",
          trace::Effect::Remove    => "tr-remove",
          trace::Effect::Alloc(trace::AllocCase::LocFresh)       => "tr-alloc-loc-fresh",
          trace::Effect::Alloc(trace::AllocCase::LocExists)      => "tr-alloc-loc-exists",
          trace::Effect::Force(trace::ForceCase::CompCacheMiss)  => "tr-force-compcache-miss",
          trace::Effect::Force(trace::ForceCase::CompCacheHit)   => "tr-force-compcache-hit",
          trace::Effect::Force(trace::ForceCase::RefGet)         => "tr-force-refget",
        })
      ],
      extent: Box::new(
        vec![
          Div{ 
            tag: String::from("tr-effect"),
            text: Some(String::from(match tr.effect {
              trace::Effect::CleanRec  => "CleanRec",
              trace::Effect::CleanEval => "CleanEval",
              trace::Effect::CleanEdge => "CleanEdge",
              trace::Effect::Dirty     => "Dirty",
              trace::Effect::Remove    => "Remove",
              trace::Effect::Alloc(trace::AllocCase::LocFresh)       => "Alloc(LocFresh)",
              trace::Effect::Alloc(trace::AllocCase::LocExists)      => "Alloc(LocExists)",
              trace::Effect::Force(trace::ForceCase::CompCacheMiss)  => "Force(CompCacheMiss)",
              trace::Effect::Force(trace::ForceCase::CompCacheHit)   => "Force(CompCacheHit)",
              trace::Effect::Force(trace::ForceCase::RefGet)         => "Force(RefGet)",
            })),
            classes:vec![],
            extent: Box::new(vec![]),
          },
          div_of_edge(&tr.edge),
        ])};  
  if tr.extent.len() > 0 {
    div.extent.push(
      Div{ tag: String::from("tr-extent"),
           text: None,
           classes: vec![],
           extent: 
           Box::new(tr.extent.iter().map(div_of_trace).collect())
      }
    )
  };
  return div
}

pub trait WriteHTML {
  fn write_html<Wr:Write>(&self, wr: &mut Wr);
}

impl WriteHTML for Div {
  fn write_html<Wr:Write>(&self, wr: &mut Wr) {    
    writeln!(wr, "<div class=\"{} {}\">", 
             self.tag, 
             self.classes.iter().fold(
               String::new(), 
               |mut cs,c|{cs.push_str(" ");
                          cs.push_str(c.as_str()); cs}
             )
    );
    match self.text {
      None => (),
      Some(ref text) => writeln!(wr, "{}", text).unwrap()
    };
    for div in self.extent.iter() {
      div.write_html(wr);
    }
    writeln!(wr, "</div>");
  }
}

impl<T:WriteHTML> WriteHTML for Vec<T> {
  fn write_html<Wr:Write>(&self, wr:&mut Wr) {
    for x in self.iter() {
      x.write_html(wr);
    }
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
    writeln!(writer, "<div class=\"time-ns-lab\">time (ns): <div class=\"time-ns\">{:?}</div></div>", 
             sample.dcg_sample.process_input.time_ns).unwrap();    
    writeln!(writer, "<div class=\"traces-lab\">Traces (<a href={:?}>doc</a>)</div>", trace_url).unwrap();
    writeln!(writer, "<div class=\"traces\">").unwrap();
    for tr in sample.dcg_sample.process_input.dcg_traces.iter() {
      div_of_trace(tr).write_html(&mut writer)
    }
    writeln!(writer, "</div>").unwrap();
    writeln!(writer, "</div>").unwrap();
    
    // - - - - - - - 
    
    writeln!(writer, "<div class=\"archivist\">").unwrap();
    writeln!(writer, "<div class=\"time-ns-lab\">time (ns): <div class=\"time-ns\">{:?}</div></div>", 
             sample.dcg_sample.compute_output.time_ns).unwrap();    
    writeln!(writer, "<div class=\"traces-lab\">Traces (<a href={:?}>doc</a>):</div>", trace_url).unwrap();
    writeln!(writer, "<div class=\"traces\">").unwrap();
    for tr in sample.dcg_sample.compute_output.dcg_traces.iter() {
      div_of_trace(tr).write_html(&mut writer)
    }
    writeln!(writer, "</div>").unwrap();    
    writeln!(writer, "</div>").unwrap();

    // - - - - - - - - - - - - - - - 
    writeln!(writer, "<hr/>").unwrap();
   
  }
  writer.flush().unwrap();  
}

pub fn style_string() -> &'static str {
"
<style>
body {
  background: #552266;
}
hr {
  float: left;
  clear: both;
  width: 0px;
  border: none;
}
.test-name {
  font-size: 66px;
  font-family: sans-serif;
  color: #ccaadd;
}
.batch-name-lab {
  font-size: 0px;
}
.time-ns {
  font-size: 20px;
  font-family: sans-serif;
  display: inline;
}
.batch-name {
  font-size: 16px;
  font-family: sans-serif;
  border: solid;
  display: inline;
  padding: 7px;
  margin: 5px;
  float: left;
  background: #aa88aa;
  width: 32px;
}
.traces {
  font-size: 8px;
  border: solid 1px;
  display: inline;
  padding: 1px;
  margin: 0px;
  float: left;
  width: 100%;
  background: #dddddd;
}
.editor {
  font-size: 14px;
  border: solid;
  display: inline;
  padding: 2px;
  margin: 2px;
  float: left;
  width: 20%;
  background: #dddddd;
}
.archivist {
  font-size: 14px;
  border: solid;
  display: inline;
  padding: 2px;
  margin: 2px;
  float: left;
  width: 70%;
  background: #dddddd;
}
.tr-extent, .trace {
  display: inline-block;
  border-style: solid;
  border-color: white;
  border-width: 1px;
  font-size: 0px;
  padding: 2px;
  margin: 1px;
  border-radius: 3px;
}
.tr-extent {
  border-style: dotted;
}
.path {
  display: flex;
  display: none;

  margin: 0px;
  padding: 0px;
  border-radius: 1px;
  border-style: solid;
  border-width: 1px;
  border-color: #664466;
  background-color: #664466; 
  autoflow: auto;
}
.name {
  display: inline;
  display: none;

  font-size: 9px;
  font-family: sans-serif;
  color: black;
  background: white;
  border-style: solid;
  border-width: 1px;
  border-color: #664466; 
  border-radius: 2px;
  padding: 1px;
  margin: 1px;
}
.tr-force-compcache-miss {  
  background: #ccccff;
  border-color: blue;
}
.tr-force-compcache-hit {  
  background: #ccccff;
  border-color: blue;
  border-width: 3px;
}
.tr-force-refget {  
  background: #ffccff;
  border-color: violet;  
}
.tr-clean-rec {  
  background: #000055;
  border-color: #aaaaff;
  border-width: 2px; 
}
.tr-clean-eval {  
  background: white;
  border-color: #aaaaff;
  border-width: 5px; 
}
.tr-clean-edge {  
  background: white;
  border-color: #aaaaff;
  border-width: 5px; 
}
.tr-alloc-loc-fresh {  
  background: #ccffcc;
  border-color: green;
}
.tr-alloc-loc-exists {  
  background: #ccffcc;
  border-color: green;
  border-width: 3px;
}
.tr-dirty {  
  background: #550000;
  border-color: #ffaaaa;
  border-width: 2px;
}
.tr-remove {  
  background: red;
  border-color: red;
  border-width: 2px;
}
</style>
"
}
