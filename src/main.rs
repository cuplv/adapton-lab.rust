#![feature(field_init_shorthand)]

//use std::fmt::Debug;
//use std::hash::Hash;
use std::rc::Rc;
use std::path::Path;

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

#[derive(Clone,Debug)]
pub enum NominalStrategy {
  Regular,
  ByContent,
}
#[derive(Clone,Debug)]
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

pub struct GenTest<Input:Generate+Edit,Output,
                   Computer:Compute<Input,Output>> {
  pub computer: Computer,
  input:        PhantomData<Input>,
  output:       PhantomData<Output>
}

#[derive(Clone,Debug)]
pub struct LabExpParams {
  pub sample_params: SampleParams,
  // TODO: Pretty-print input and output structures; graphmovie dump of experiment
}

#[derive(Clone,Debug)]
pub struct SampleParams {
  /// We convert this seed into a random-number-generator before generating and editing.
  pub input_seed:        u64, 
  /// Other parameters for generating the input.
  pub generate_params:   GenerateParams, 
  /// Whether to validate the output after each computation using the naive and DCG engines
  pub validate_output:   bool,
  /// Size of each batch of changes.
  pub change_batch_size: usize,
  /// Number of change-batches to perform in a loop; each is interposed with computing the new output.
  pub change_batch_loop: usize,
}

#[derive(Clone,Debug)]
pub struct LabExpResults {
  pub samples: Vec<Sample>
}

#[derive(Clone,Debug)]
pub struct Sample {
  pub params:       SampleParams,
  pub batch_name:   usize,   // Index/name the change batches; one sample per compute + change batch
  pub dcg_sample:   EngineSample,
  pub naive_sample: EngineSample,
  pub output_valid: Option<bool>
}

#[derive(Clone,Debug)]
pub struct EngineSample {
  pub generate_input:   EngineMetrics,
  pub compute_output:   EngineMetrics,
  pub batch_edit_input: EngineMetrics,
}

#[derive(Clone,Debug)]
pub struct EngineMetrics {
  pub time_ns:    u64,
  pub engine_cnt: Cnt,
}

// Lab experiment; Hides the Input, Output and Compute types, abstracting over them:
pub trait LabExp {
  fn run<R:Rng+Clone>(self:Self, params:&LabExpParams) -> LabExpResults;
}

impl<Input:'static+Generate+Edit,Output,
     Computer:'static+Compute<Input,Output>>
  LabExp for GenTest<Input,Output,Computer> {

    fn run<R:Rng+Clone> (self:Self, params:&LabExpParams) -> LabExpResults 
    {
      
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
        return (output, input3, EngineSample{
          generate_input,
          compute_output,
          batch_edit_input,
        })
      }

      // TODO: Want a loop here, where we switch back and forth between using the Naive engine, and using the DCG engine.
      // We want to interleave this way in order to compare outputs and metrics (counts and timings) on a fine-grained scale.
      fn get_sample<Input:Generate+Edit,Output,Computer:Compute<Input,Output> > (params:&SampleParams) -> Sample {
        // We "want" this to be really, really *slow*:
        init_naive();
        assert!(engine_is_naive());
        let rng = panic!("XXX: Todo, generate from a seed.");
        let (naive_output, _, naive_sample) = get_engine_sample::<rand::ThreadRng,Input,Output,Computer>(rng, params, None);

        // We want this to be really, really *fast*:
        init_dcg();
        assert!(engine_is_dcg());
        let rng = panic!("XXX: Todo, generate from a seed.");
        let (dcg_output, _, dcg_sample) = get_engine_sample::<rand::ThreadRng,Input,Output,Computer>(rng, params, None);
        // TODO: Compare the equality of the outputs, when command-line arguments say so; XXX
        let output_valid = None;
        return Sample{
          params:params.clone(),
          // TODO: Index/name the change batches; one sample per compute + change batch
          batch_name:0,
          dcg_sample,
          naive_sample,
          output_valid,
        }
      }

      let mut samples : Vec<Sample> = vec![];
      
      let sample1 = get_sample::<Input,Output,Computer>(&params.sample_params);
      samples.push(sample1);
      let sample2 = get_sample::<Input,Output,Computer>(&params.sample_params);
      samples.push(sample2);
      
      return LabExpResults {
        samples: samples,
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
  

fn csv_of_runtimes(path:&str, runtimes: Vec<(isize, u64, isize)>) {
  let path = Path::new(path);
  let mut writer = csv::Writer::from_file(path).unwrap();
  for r in runtimes.into_iter() {
    //println!("{:?}",r);
    writer.encode(r).ok().expect("CSV writer error");
  }
}

fn main() {
    println!("Hello, world!");
}
