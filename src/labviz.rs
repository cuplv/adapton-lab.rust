use std::fs;
//use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::collections::HashMap;

use adapton::engine::Name;
use adapton::engine::reflect::*;
use adapton::engine::reflect::{trace, string_of_name};
use labdef::{LabParams,LabDef,LabResults};

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

pub fn div_of_name (n:&Name) -> Div {
  Div{ tag: String::from("name"),
       // TODO: Remove illegal chars for CSS classes (check spec)
       // classes: vec![ format!("{:?}", n) ],
       classes: vec![ string_of_name(n) ],
       extent: Box::new( vec![ ] ),
       text: Some( format!("{}", string_of_name(n) ) ) }
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

pub fn div_of_force_tree (dcg:&DCG, visited:&mut HashMap<Loc, ()>, loc:&Loc) -> Div {  
  let mut div = Div {
    tag:String::from("force-tree"),
    text:None,
    classes: vec![],
    extent: Box::new(vec![ div_of_loc( loc ) ]),
  };
  visited.insert( loc.clone(), () );
  let no_extent = match dcg.table.get( loc ) {
    None => panic!("dangling pointer in reflected DCG!"),
    Some( nd ) => {
      match succs_of_node( nd ) {
        None => true, // No succs; E.g., ref cells have no succs
        Some( succs ) => {
          let mut no_extent = true;
          for succ in succs {
            if succ.effect == Effect::Force {
              no_extent = false;
              let succ_div = div_of_force_tree (dcg, visited, &succ.loc);
              div.extent.push( succ_div )
            }
          };
          no_extent
        }
      }
    }
  };
  if no_extent {
    div.classes.push(String::from("no-extent"))
  };
  div
}

pub fn div_of_alloc_tree (dcg:&DCG, visited:&mut HashMap<Loc, ()>, loc:&Loc) -> Div {  
  let mut div = Div {
    tag:String::from("alloc-tree"),
    text:None,
    classes: vec![],
    extent: Box::new(vec![ div_of_loc( loc ) ]),
  };
  visited.insert( loc.clone(), () );
  let no_extent = match dcg.table.get( loc ) {
    None => panic!("dangling pointer in reflected DCG!"),
    Some( nd ) => {
      match succs_of_node( nd ) {
        None => true, // No succs; E.g., ref cells have no succs
        Some( succs ) => {
          let mut no_extent = true;
          for succ in succs {
            if succ.effect == Effect::Alloc {
              no_extent = false;
              let succ_div = div_of_alloc_tree (dcg, visited, &succ.loc);
              div.extent.push( succ_div )
            }
          };
          no_extent
        }
      }
    }
  };
  if no_extent {
    div.classes.push(String::from("no-extent"))
  };
  div
}

pub fn div_of_value_tree (dcg:&DCG, tr:&trace::Trace) -> Div {
  panic!("")  
}

pub fn div_of_trace (tr:&trace::Trace) -> Div {
  // For linking to rustdoc documentation from the output HTML
  let tr_eff_url = "http://adapton.org/rustdoc/adapton/engine/reflect/trace/enum.Effect.html";

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
          trace::Effect::Alloc(trace::AllocCase::LocFresh,_)     => "tr-alloc-loc-fresh",
          trace::Effect::Alloc(trace::AllocCase::LocExists,_)    => "tr-alloc-loc-exists",
          trace::Effect::Force(trace::ForceCase::CompCacheMiss)  => "tr-force-compcache-miss",
          trace::Effect::Force(trace::ForceCase::CompCacheHit)   => "tr-force-compcache-hit",
          trace::Effect::Force(trace::ForceCase::RefGet)         => "tr-force-refget",
        })
      ],
      extent: Box::new(
        vec![
          Div{ 
            tag: String::from("tr-effect"),
            text: Some(              
              format!("<a href={:?}>{}</a>", tr_eff_url, match tr.effect {
                trace::Effect::CleanRec  => "CleanRec",
                trace::Effect::CleanEval => "CleanEval",
                trace::Effect::CleanEdge => "CleanEdge",
                trace::Effect::Dirty     => "Dirty",
                trace::Effect::Remove    => "Remove",
                trace::Effect::Alloc(trace::AllocCase::LocFresh,_)     => "Alloc(LocFresh)",
                trace::Effect::Alloc(trace::AllocCase::LocExists,_)    => "Alloc(LocExists)",
                trace::Effect::Force(trace::ForceCase::CompCacheMiss)  => "Force(CompCacheMiss)",
                trace::Effect::Force(trace::ForceCase::CompCacheHit)   => "Force(CompCacheHit)",
                trace::Effect::Force(trace::ForceCase::RefGet)         => "Force(RefGet)",
              })),
            classes: vec![],
            extent: Box::new(vec![]),
          },
          Div{
            tag: String::from("tr-symbols"),
            text: match tr.effect {
              trace::Effect::Alloc(_,trace::AllocKind::RefCell) => Some(String::from("▣")),
              trace::Effect::Alloc(_,trace::AllocKind::Thunk)   => Some(String::from("◯")),
              _ => None,              
            },
            classes:vec![],
            extent: Box::new(vec![]),
          },
          div_of_edge(&tr.edge),
        ])}
  ;
  match tr.effect {
    trace::Effect::Alloc(_,trace::AllocKind::RefCell) => div.classes.push(String::from("alloc-kind-refcell")),
    trace::Effect::Alloc(_,trace::AllocKind::Thunk)   => div.classes.push(String::from("alloc-kind-thunk")),
    _ => ()
  };
  if tr.extent.len() > 0 {
    div.classes.push( String::from("has-extent") );
    div.extent.push(
      Div{ tag: String::from("tr-extent"),
           text: None,
           classes: vec![],
           extent: 
           Box::new(tr.extent.iter().map(div_of_trace).collect())
      }
    )
  } else {
    div.classes.push( String::from("no-extent") );
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
    ).unwrap();
    match self.text {
      None => (),
      Some(ref text) => writeln!(wr, "{}", text).unwrap()
    };
    for div in self.extent.iter() {
      div.write_html(wr);
    }
    writeln!(wr, "</div>").unwrap();
  }
}

impl<T:WriteHTML> WriteHTML for Vec<T> {
  fn write_html<Wr:Write>(&self, wr:&mut Wr) {
    for x in self.iter() {
      x.write_html(wr);
    }
  }
}

pub fn write_all_test_results(_params:&LabParams, 
                              tests:&Vec<Box<LabDef>>, 
                              results:&Vec<LabResults>) 
{
  // Create directories and files on local filesystem:
  fs::create_dir_all("lab-results").unwrap();
  let f = File::create(format!("lab-results/index.html")).unwrap();
  let mut writer = BufWriter::new(f);

  writeln!(writer, "{}", style_string()).unwrap();

  assert!( tests.len() == results.len() );

  for ((_i,test),(_j,_result)) in tests.iter().enumerate().zip(results.iter().enumerate()) {
    //writeln!(writer, "(({:?},{:?}),({:?},{:?}))", i, test.name(), j, result);
    writeln!(&mut writer, "<div class={:?}>", "test-summary-title").unwrap();
    write_test_name(&mut writer, test, false);
    write_cr(&mut writer);
    writeln!(&mut writer, "<a class={:?} href=./{}/traces.html>example traces</a>", 
             "test-summary-examples", 
             string_of_name(&test.name())
    ).unwrap();
    writeln!(&mut writer, "</div").unwrap();    
    write_cr(&mut writer);
    
    writeln!(&mut writer, "<div class={:?}>", "test-summary").unwrap();    
    writeln!(&mut writer, "<div class={:?}>", "test-summary-info").unwrap();
    //
    writeln!(&mut writer, "</div>").unwrap();    
    writeln!(&mut writer, "<div class={:?}>", "test-summary-large-results").unwrap();
    //
    writeln!(&mut writer, "</div>").unwrap();
    writeln!(&mut writer, "<div class={:?}>", "test-summary-small-results").unwrap();
    //
    writeln!(&mut writer, "</div>").unwrap();        
    writeln!(&mut writer, "</div>").unwrap();    
  }
}

pub fn write_cr<W:Write>(writer:&mut W) {
  /// We style this with clear:both, and without any appearance
  writeln!(writer, "<hr/>").unwrap();
}

pub fn write_test_name<W:Write>(writer:&mut W, test:&Box<LabDef>, is_title:bool) {
  let catalog_url = String::from("http://adapton.org/rustdoc/adapton_lab/catalog/index.html");

  let testname = string_of_name( &test.name() );
  let testurl  = test.url();

  writeln!(writer, "<div class={:?}><a href={:?} class={:?}>{}</a></div>", 
           "test-name",
           match *testurl {
             Some(ref url) => url,
             None => & catalog_url
           },
           format!("test-name {}", if is_title { "page-title" } else { "" }), 
           testname
  ).unwrap();
}

pub fn write_test_results_traces(_params:&LabParams, test:&Box<LabDef>, results:&LabResults) {
  
  let testname = string_of_name( &test.name() );
  //let testurl  = test.url();

  // For linking to rustdoc documentation from the output HTML
  let trace_url   = "http://adapton.org/rustdoc/adapton/engine/reflect/trace/struct.Trace.html";
  
  // Create directories and files on local filesystem:
  fs::create_dir_all(format!("lab-results/{}/", testname)).unwrap();
  let f = File::create(format!("lab-results/{}/traces.html", testname)).unwrap();
  let mut writer = BufWriter::new(f);

  writeln!(writer, "{}", style_string()).unwrap();
  
  write_test_name(&mut writer, test, true);

  writeln!(writer, "<div style=\"font-size:12px\" class=\"batch-name\"> step</div>").unwrap();
  writeln!(writer, "<div style=\"font-size:20px\" class=\"editor\">Editor</div>").unwrap();
  writeln!(writer, "<div style=\"font-size:20px\" class=\"archivist\">Archivist</div>").unwrap();
  write_cr(&mut writer);

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

    match sample.dcg_sample.process_input.dcg_reflect {
      Some(ref dcg) => {
        writeln!(writer, "<div class=\"alloc-tree\">").unwrap();
        for tr in sample.dcg_sample.process_input.dcg_traces.iter() {
          if tr.edge.succ.effect == Effect::Alloc {
            div_of_alloc_tree(dcg, 
                              &mut HashMap::new(), 
                              &tr.edge.succ.loc)
              .write_html(&mut writer)
          }
        }
        writeln!(writer, "</div>").unwrap();

        writeln!(writer, "<div class=\"force-tree\">").unwrap();
        for tr in sample.dcg_sample.process_input.dcg_traces.iter() {
          if tr.edge.succ.effect == Effect::Force {
            div_of_force_tree(dcg, 
                              &mut HashMap::new(), 
                              &tr.edge.succ.loc)
              .write_html(&mut writer)
          }
        }
        writeln!(writer, "</div>").unwrap();

      }
      None => {
        // No reflected DCG
      }
    }
    
    writeln!(writer, "</div>").unwrap();
    
    // - - - - - - - 
    
    writeln!(writer, "<div class=\"archivist\">").unwrap();
    
    writeln!(writer, "<div class=\"time-ns-lab\">time (ns): <div class=\"time-ns\">{:?}</div></div>", 
             sample.dcg_sample.compute_output.time_ns).unwrap();    

    writeln!(writer, "<div class=\"time-ms-lab\">time (ms): <div class=\"time-ms\">{:.*}</div></div>", 
             2, (sample.dcg_sample.compute_output.time_ns as f64) / (1000000 as f64),
    ).unwrap();    
    
    writeln!(writer, "<div class=\"traces-lab\">Traces (<a href={:?}>doc</a>):</div>", trace_url).unwrap();    
    writeln!(writer, "<div class=\"traces\">").unwrap();
    for tr in sample.dcg_sample.compute_output.dcg_traces.iter() {
      div_of_trace(tr).write_html(&mut writer)
    }
    writeln!(writer, "</div>").unwrap();    

    match sample.dcg_sample.compute_output.dcg_reflect {
      Some(ref dcg) => {
        writeln!(writer, "<div class=\"alloc-tree\">").unwrap();
        for tr in sample.dcg_sample.compute_output.dcg_traces.iter() {
          if tr.edge.succ.effect == Effect::Alloc {
            div_of_alloc_tree(dcg, 
                              &mut HashMap::new(), 
                              &tr.edge.succ.loc)
              .write_html(&mut writer)
          }
        }
        writeln!(writer, "</div>").unwrap();

        writeln!(writer, "<div class=\"force-tree\">").unwrap();
        for tr in sample.dcg_sample.compute_output.dcg_traces.iter() {
          if tr.edge.succ.effect == Effect::Force {
            div_of_force_tree(dcg, 
                              &mut HashMap::new(), 
                              &tr.edge.succ.loc)
              .write_html(&mut writer)
          }
        }
        writeln!(writer, "</div>").unwrap();
      }
      None => {
        // No reflected DCG
      }
    }

    writeln!(writer, "</div>").unwrap();

    // - - - - - - - - - - - - - - - 
    write_cr(&mut writer);
   
  }
  writer.flush().unwrap();  
}

pub fn style_string() -> &'static str {
"
<html>
<head>
<script src=\"https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js\"></script>

<style>
body {
  background: #552266;
  font-family: sans-serif;
  text-decoration: none;
  padding: 0px;
  margin: 0px;
}
:visited {
  color: black;
}
a {
  text-decoration: none;
}
a:hover {
  text-decoration: underline;
}
hr {
  float: left;
  clear: both;
  width: 0px;
  border: none;
}

.test-name {
  color: #ccaadd;
  margin: 1px;
  padding: 1px;
}

.test-summary-title {
  margin: 8px;
  font-size: 20px;
}
.test-summary {
  margin: 8px;
  padding: 2px;
}
.test-summary-examples {
  font-size: 14px;
  color: black;
  border: solid black 1px;
  padding: 2px;
  background-color: yellow;
  margin: 3px;
}
.test-summary-examples:hover {
  background-color: white;
}
.test-name:visited {
  color: #ccaadd;
}
.test-name:hover {
  color: white;
}

.batch-name-lab {
  font-size: 0px;
}
.batch-name {
  font-size: 16px;
  border: solid;
  display: inline;
  padding: 3px;
  margin: 3px;
  float: left;
  background: #aa88aa;
  width: 32px;
}
.time-ns {
  font-size: 20px;
  display: inline;
}
.time-ms {
  font-size: 20px;
  display: inline;
}
.editor {
  font-size: 14px;
  border: solid;
  display: block;
  padding: 1px;
  margin: 1px;
  float: left;
  width: 10%;
  background: #aaaaaa;
}
.archivist {
  font-size: 14px;
  border: solid;
  display: block;
  padding: 1px;
  margin: 1px;
  float: left;
  width: 85%;
  background: #dddddd;
}
.traces {
  font-size: 8px;
  border: solid 0px;
  border-top: solid 1px;
  padding: 0px;

  display: block;
  margin: 0px;
  float: left;
  width: 100%;
}

.trace, .force-tree, .alloc-tree {
  display: inline-block;
  border-style: solid;
  border-color: red;
  border-width: 1px;
  font-size: 0px;
  padding: 0px;
  margin: 1px;
  border-radius: 5px;
}
.tr-effect { 
  display: inline;
  display: none;
  font-size: 10px;
  background-color: white;
  border-radius: 2px;
}
.tr-symbols {  
  font-size: 10px;
  display: inline;
  display: none;
}

.path {  
  display: inline-block;
  display: none;

  margin: 0px;
  padding: 1px;
  border-radius: 1px;
  border-style: solid;
  border-width: 1px;
  border-color: #664466;
  background-color: #664466; 
}
.name {
  display: inline;
  display: none;

  font-size: 9px;
  color: black;
  background: white;
  border-style: solid;
  border-width: 1px;
  border-color: #664466; 
  border-radius: 2px;
  padding: 1px;
  margin: 1px;
}

.alloc-kind-thunk {
  border-color: green;
  border-radius:20px;
}
.alloc-kind-refcell {
  border-color: green;
  border-radius:0;
}
.tr-force-compcache-miss {  
  background: #ccccff;
  border-color: blue;
  padding: 0px;
}
.tr-force-compcache-hit {  
  background: #ccccff;
  border-color: blue;
  border-width: 4px;
  padding: 3px;
}
.tr-force-refget {  
  border-radius: 0;
  border-color: blue;
}
.tr-clean-rec {  
  background: #222244;
  border-color: #aaaaff;
  border-width: 1px; 
}
.tr-clean-eval {  
  background: #8888ff;
  border-color: white;
  border-width: 4px; 
}
.tr-clean-edge {  
  background: white;
  border-color: #aaaaff;
  border-width: 2px; 
  padding: 3px;
}
.tr-alloc-loc-fresh {  
  padding: 3px;
  background: #ccffcc;
}
.tr-alloc-loc-exists {  
  padding: 3px;
  background: #ccffcc;
  border-width: 4px;
  border-color: green;
}
.tr-dirty {  
  background: #550000;
  border-color: #ffaaaa;
  border-width: 1px;
}
.tr-remove {  
  background: red;
  border-color: black;
  border-width: 2px;
  padding: 2px;
}

.force-tree {
  background: #ccccff;
  border-color: blue;
}
.alloc-tree {
  background: #ccffcc;
  border-color: green;
}

.no-extent {
  padding: 3px;
}
.page-title {
  font-size: 32px;
  color: #ccaadd;
  margin: 8px;
}

</style>

<script>
function togglePaths() {
 var selection = document.getElementById(\"checkbox-1\");
 if (selection.checked) {
   $('.path').css('display', 'inline-block')
 } else {
   $('.path').css('display', 'none')
 }
}

function toggleNames() {
 var selection = document.getElementById(\"checkbox-2\");
 if (selection.checked) {
   $('.name').css('display', 'inline')
 } else {
   $('.name').css('display', 'none')
 }
}

function toggleEffects() {
 var selection = document.getElementById(\"checkbox-3\");
 if (selection.checked) {
   $('.tr-effect').css('display', 'inline')
 } else {
   $('.tr-effect').css('display', 'none')
 }
}
</script>
</head>

<body>

<fieldset>
 <legend>Toggle labels: </legend>
 <label for=\"show-paths-checkbox\">paths</label>
 <input type=\"checkbox\" name=\"show-paths-checkbox\" id=\"checkbox-1\" onchange=\"togglePaths()\">
 <label for=\"show-names-checkbox\">names</label>
 <input type=\"checkbox\" name=\"show-names-checkbox\" id=\"checkbox-2\" onchange=\"toggleNames()\">
 <label for=\"show-effects-checkbox\">effects</label>
 <input type=\"checkbox\" name=\"show-effects-checkbox\" id=\"checkbox-3\" onchange=\"toggleEffects()\">
</fieldset>
"
}
