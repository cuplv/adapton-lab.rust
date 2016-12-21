#![feature(field_init_shorthand)]
#![feature(rustc_private)]

//use std::fmt::Debug;
//use std::hash::Hash;
use std::rc::Rc;
use std::path::Path;

extern crate serialize;
extern crate time;
extern crate csv;
extern crate rand;

#[macro_use]
extern crate adapton;

use adapton::macros::*;
use adapton::collections::*;
use adapton::engine::*;
use rand::Rng;
use std::marker::PhantomData;

#[derive(Clone,Debug,Encodable)]
pub enum NominalStrategy {
  Regular,
  ByContent,
}
#[derive(Clone,Debug,Encodable)]
pub struct GenerateParams {
  pub size: usize, 
  pub gauge: usize, 
  pub nominal_strategy:NominalStrategy
}

pub trait Generate {
  fn generate<R:Rng>(rng:&mut R, params:&GenerateParams) -> Self;
} 

pub trait Edit : Clone {
  fn edit<R:Rng>(Self, rng:&mut R, params:&GenerateParams) -> Self;
}

pub trait Compute<Input,Output> {
  fn compute(Input) -> Output;
}

pub struct Computer<Input,Output,
                    Computer:Compute<Input,Output>> {
  pub computer: Computer,
  input:        PhantomData<Input>,
  output:       PhantomData<Output>
}

pub struct TestComputer<Input:Generate+Edit,Output,
                        Computer:Compute<Input,Output>> {
  computer: PhantomData<Computer>,
  input:    PhantomData<Input>,
  output:   PhantomData<Output>
}


#[derive(Clone,Debug,Encodable)]
pub struct LabExpParams {
  pub sample_params: SampleParams,
  // TODO: Pretty-print input and output structures; graphmovie dump of experiment
  /// Number of change-batches to perform in a loop; each is interposed with computing the new output.
  pub change_batch_loopc: usize,
}

#[derive(Clone,Debug,Encodable)]
pub struct SampleParams {
  /// We convert this seed into a random-number-generator before generating and editing.
  pub input_seed:        u64, 
  /// Other parameters for generating the input.
  pub generate_params:   GenerateParams, 
  /// Whether to validate the output after each computation using the naive and DCG engines
  pub validate_output:   bool,
  /// Size of each batch of changes.
  pub change_batch_size: usize,
}

#[derive(Clone,Debug,Encodable)]
pub struct LabExpResults {
  pub samples: Vec<Sample>
}

#[derive(Clone,Debug,Encodable)]
pub struct Sample {
  pub params:       SampleParams,
  pub batch_name:   usize,   // Index/name the change batches; one sample per compute + change batch
  pub dcg_sample:   EngineSample,
  pub naive_sample: EngineSample,
  pub output_valid: Option<bool>
}

#[derive(Clone,Debug,Encodable)]
pub struct EngineSample {
  pub generate_input:   EngineMetrics,
  pub compute_output:   EngineMetrics,
  pub batch_edit_input: EngineMetrics,
}

#[derive(Clone,Debug,Encodable)]
pub struct EngineMetrics {
  pub time_ns:    u64,
  pub engine_cnt: Cnt,
}


pub trait SampleGen {
  fn sample(self:&mut Self) -> Option<Sample>;
}

pub struct TestEngineState<Input:Generate+Edit,Output,
                           Computer:Compute<Input,Output>> {
  pub engine:   Engine,
  pub input:    Input,
  computer:     PhantomData<Computer>,
  output:       PhantomData<Output>,
}

pub struct TestState<Input:Generate+Edit,Output,
                     Computer:Compute<Input,Output>> {
  pub params:           LabExpParams,
  pub change_batch_num: usize,
  pub change_batch_loopc: usize,
  pub dcg_state:   TestEngineState<Input,Output,Computer>,
  pub naive_state: TestEngineState<Input,Output,Computer>,
  pub samples:     Vec<Sample>,
}

      
fn get_engine_metrics<X,F:FnOnce() -> X> (thunk:F) -> (X,EngineMetrics)
{
  let time_start = time::precise_time_ns();
  let (x,cnt) = cnt(thunk);
  let time_end = time::precise_time_ns();
  return (x, EngineMetrics{
    time_ns:time_end - time_start,
    engine_cnt:cnt,
  })
}

fn get_engine_sample<R:Rng+Clone,Input:Generate+Edit,Output,Computer:Compute<Input,Output>> (rng:&mut R, params:&SampleParams, input:Option<Input>) -> (Output,Input,EngineSample) {
  let mut rng2 = rng.clone();
  let (input, generate_input) : (Input,EngineMetrics) = match input {
    None        => get_engine_metrics(move || Input::generate(&mut rng2, &params.generate_params) ),
    Some(input) => get_engine_metrics(move || { input } )
  };
  let input2 = input.clone();
  let (output, compute_output): (Output,EngineMetrics) = get_engine_metrics(move || Computer::compute(input2) );        
  let (input3, batch_edit_input): (_, EngineMetrics)   = get_engine_metrics(move || Input::edit(input, rng, &params.generate_params) );
  let engine_sample = EngineSample{
    generate_input,
    compute_output,
    batch_edit_input,
  };
  println!("{:?}", engine_sample); // XXX Temp
  return (output, input3, engine_sample)
}

impl<Input:Generate+Edit,Output:Eq,
     Computer:Compute<Input,Output>>
  SampleGen for TestState<Input,Output,Computer> {
    fn sample (self:&mut Self) -> Option<Sample> {
      if ( self.change_batch_num == self.params.change_batch_loopc ) { None } else { 
        
        // Run DCG Version (We want this to be really, really *fast*):
        let dcg = self.dcg_state.engine.clone(); // Not sure about whether this Clone will do what we want; XXX
        let _ = use_engine(dcg);
        assert!(engine_is_dcg());
        let rng = panic!("XXX: Todo, generate from a seed.");
        let (dcg_output, dcg_input, dcg_sample) = get_engine_sample::<rand::ThreadRng,Input,Output,Computer>(rng, &self.params.sample_params, None);
        self.dcg_state.engine = use_engine(Engine::Naive); // Save the DCG state for later.
        self.dcg_state.input = dcg_input;

        // Run Naive Version (We want this to be really, really *slow* compared to faster DCG version):
        let _ = use_engine(Engine::Naive);
        assert!(engine_is_naive());
        let rng = panic!("XXX: Todo, generate from a seed.");
        let (naive_output, naive_input, naive_sample) = get_engine_sample::<rand::ThreadRng,Input,Output,Computer>(rng, &self.params.sample_params, None);
        self.naive_state.input = naive_input;
        
        // Compare the two outputs for equality
        let output_valid = if self.params.sample_params.validate_output { 
          Some ( dcg_output == naive_output )
        } else { None } ;

        let sample = Sample{
          params:self.params.sample_params.clone(),
          // TODO: Index/name the change batches; one sample per compute + change batch
          batch_name:self.change_batch_num + 1,
          dcg_sample,
          naive_sample,
          output_valid,
        };
      }
    }
  }

// Lab experiment; Hides the Input, Output and Compute types, abstracting over them:
pub trait LabExp {
  fn run<R:Rng+Clone>(self:Self, params:&LabExpParams) -> LabExpResults;
}

impl<Input:'static+Generate+Edit,Output:Eq,
     Computer:'static+Compute<Input,Output>>
  LabExp for TestComputer<Input,Output,Computer> {
    
    fn run<R:Rng+Clone> (self:Self, params:&LabExpParams) -> LabExpResults 
    {
      // TODO: Want a loop here, where we switch back and forth between using the Naive engine, and using the DCG engine.
      // We want to interleave this way in order to compare outputs and metrics (counts and timings) on a fine-grained scale.
      fn get_sample_gen<Input:Generate+Edit,Output,Computer:Compute<Input,Output>> (params:&LabExpParams) -> TestState<Input,Output,Computer> {
        // We "want" this to be really, really *slow*:
        init_naive();
        assert!(engine_is_naive());
        let rng = panic!("XXX: Todo, generate from a seed.");
        let (naive_output, naive_input, naive_sample) = get_engine_sample::<rand::ThreadRng,Input,Output,Computer>(rng, &params.sample_params, None);
        
        // We want this to be really, really *fast*:
        let mut naive = init_dcg();
        assert!(engine_is_dcg());
        let rng = panic!("XXX: Todo, generate from a seed.");
        let (dcg_output, dcg_input, dcg_sample) = get_engine_sample::<rand::ThreadRng,Input,Output,Computer>(rng, &params.sample_params, None);
        // TODO: Compare the equality of the outputs, when command-line arguments say so; XXX
        let output_valid = None;
        let sample = Sample{
          params:params.sample_params.clone(),
          batch_name:0, // Index/name the change batches; one sample per compute + change batch
          dcg_sample,
          naive_sample,
          output_valid,
        };
        let dcg = use_engine(Engine::Naive); // TODO-Minor: Rename this operation: "engine_swap" or something 
        TestState{      
          params:params.clone(),
          dcg_state:TestEngineState{
            input: dcg_input,
            engine: dcg,
            output: PhantomData,
            computer: PhantomData,

          },
          naive_state:TestEngineState{
            input: naive_input,
            engine: panic!("TODO: use a special constant for this"),
            output: PhantomData,
            computer: PhantomData,
          },
          change_batch_loopc: params.change_batch_loopc,
          change_batch_num: 1,
          samples:vec![sample],
        }
      }
      
      let mut st = get_sample_gen::<Input,Output,Computer>(params);
      loop {
      let _ = (&mut st).sample();
      let _ = (&mut st).sample();
      
      return LabExpResults {
        samples: st.samples,
      }
    }
  }
}


fn forkboilerplate () {
  use std::thread;
  let child =
    thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(move || { 
      panic!("");
    });
  let _ = child.unwrap().join();
}
  

fn csv_of_runtimes(path:&str, samples: Vec<Sample>) {
  let path = Path::new(path);
  let mut writer = csv::Writer::from_file(path).unwrap();
  for r in samples.into_iter() {
    //println!("{:?}",r);
    //writer.encode(r).ok().expect("CSV writer error");
  }
}

fn main() {
    println!("Hello, world!");
}
