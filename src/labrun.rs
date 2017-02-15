extern crate time;
extern crate rand;

use std::fmt::Debug;
use labdef::*;
use std::marker::PhantomData;

use adapton::engine::*;
use adapton::engine::manage::*;
use rand::{Rng, SeedableRng};
use std::mem::swap;

pub trait SampleGen {
  fn sample(self:&mut Self) -> Option<Sample>;
}

pub struct LabEngineState<Input,EditSt,Output,
                           Editor:Generate<Input>+Edit<Input,EditSt>,
                           Archivist:ComputeDemand<Input,Output>> {
  pub engine:   Engine,
  pub input:    Option<(Input,EditSt)>,
  inputdist:    PhantomData<Editor>,
  computer:     PhantomData<Archivist>,
  output:       PhantomData<Output>,
}

pub struct LabState<R:Rng+Clone,
                     Input,EditSt,Output,
                     Editor:Generate<Input>+Edit<Input,EditSt>,
                     Archivist:ComputeDemand<Input,Output>> {
  pub params:           LabParams,
  pub rng:              Box<R>,
  pub change_batch_num: usize,
  pub dcg_state:   LabEngineState<Input,EditSt,Output,Editor,Archivist>,
  pub naive_state: LabEngineState<Input,EditSt,Output,Editor,Archivist>,
  pub samples:     Vec<Sample>,
}

      
fn get_engine_metrics<X,F:FnOnce() -> X> (params:&SampleParams, thunk:F) -> (X,EngineMetrics)
{
  if params.reflect_trace { reflect::dcg_reflect_begin(); };
  let time_start = time::precise_time_ns();
  let (x,cnt) = cnt(thunk);
  let time_end = time::precise_time_ns();
  let traces = if params.reflect_trace { reflect::dcg_reflect_end() } else { vec![ ] };
  let dcg    = if params.reflect_dcg   { reflect::dcg_reflect_now() } else { None };
  return (x, EngineMetrics{
    time_ns:time_end - time_start,
    engine_cnt:cnt,
    reflect_traces:traces,
    reflect_dcg:dcg,
  })
}

fn get_engine_sample
  <R:Rng+Clone,
   Input:Clone+Debug,
   EditSt,Output:Debug,   
   Editor:Generate<Input>+Edit<Input,EditSt>,
   Archivist:ComputeDemand<Input,Output>
   > 
  (rng:&mut R, params:&SampleParams, input:Option<(Input,EditSt)>) -> (Output,Input,EditSt,EngineSample) 
{
  let mut rng2 = rng;
  
  let ((edited_input, editst), process_input) : ((Input,EditSt),EngineMetrics) = 
    match input {
      None => 
        get_engine_metrics( params,
          move || ( Editor::generate(&mut rng2, &params.generate_params), 
                    Editor::edit_init(&mut rng2, &params.generate_params ))),
      Some((input, editst)) => 
        get_engine_metrics( params,
          move || Editor::edit(input, editst, &mut rng2, &params.generate_params))
    };

  let input2  = edited_input.clone();
  
  let input2r = 
    if params.reflect_dcg { 
      Some(reflect::reflect_val(&input2)) 
    } else { None };

  let (output, compute_output): (Output,EngineMetrics) 
    = ns(name_of_str("compute"),
         move || 
         get_engine_metrics( 
           params, move || 
             Archivist::compute(input2, params.demand) 
         ));

  let outputr = 
    if params.reflect_dcg { 
      Some(reflect::reflect_val(&output)) 
    } else { None };
  
  let engine_sample = EngineSample{
    process_input,
    input: input2r,
    compute_output,
    output: outputr,
  };

  return (output, edited_input, editst, engine_sample)
}

fn get_sample_gen
  <Input:Clone+Debug,
   EditSt,
   Output:Eq+Debug,
   Editor:Generate<Input>+Edit<Input,EditSt>,
   Archivist:ComputeDemand<Input,Output>> 
  (params:&LabParams) 
   -> LabState<rand::StdRng,Input,EditSt,Output,Editor,Archivist> 
{
  // Create empty DCG; TODO-Minor-- Make the API for this better.
  let _ = init_dcg(); assert!(engine_is_dcg());
  let empty_dcg = use_engine(Engine::Naive); // TODO-Minor: Rename this operation: "engine_swap" or something 
  let rng = SeedableRng::from_seed(params.sample_params.input_seeds.as_slice());
  //let editst_init = Editor::edit_init(&mut rng, & params.sample_params.generate_params);
  LabState{
    params:params.clone(),
    rng:Box::new(rng),
    dcg_state:LabEngineState{
      input:  None,
      engine: empty_dcg, // empty DCG      
      output: PhantomData, inputdist: PhantomData, computer: PhantomData,      
    },
    naive_state:LabEngineState{
      input:  None,
      engine: Engine::Naive, // A constant
      output: PhantomData, inputdist: PhantomData, computer: PhantomData,
    },
    change_batch_num: 0,
    samples:vec![],
  }
}

/// Advances the LabState forward by one sample of each engine.  For
/// each engine, we process the current input (either generating it,
/// or editing it) and we compute a new output over this processed input.
/// Optionally, we compare the outputs of the engines for equality.
impl<Input:Clone+Debug,EditSt,Output:Eq+Debug,
     Editor:Generate<Input>+Edit<Input,EditSt>,
     Archivist:ComputeDemand<Input,Output>>
  SampleGen for LabState<rand::StdRng,Input,EditSt,Output,Editor,Archivist> {
    fn sample (self:&mut Self) -> Option<Sample> {
      if self.change_batch_num > self.params.change_batch_loopc {
        None 
      } else { // Collect the next sample, for each engine, using get_engine_sample.
        let mut dcg_state = LabEngineState{ input: None, engine: Engine::Naive, 
                                             output: PhantomData, inputdist: PhantomData, computer: PhantomData };
        swap(&mut dcg_state, &mut self.dcg_state );
        let mut naive_state = LabEngineState{ input: None, engine: Engine::Naive, 
                                               output: PhantomData, inputdist: PhantomData, computer: PhantomData };
        swap(&mut naive_state, &mut self.naive_state );

        // Run Naive Version
        //println!("Naive - - - - - ({:?} / {:?})", self.change_batch_num, self.params.change_batch_loopc );
        let _ = use_engine(Engine::Naive); assert!(engine_is_naive());
        let mut rng = self.rng.clone(); // Restore Rng
        let (naive_output, naive_input_edited, naive_editst, naive_sample) = 
          get_engine_sample::<rand::StdRng,Input,EditSt,Output,Editor,Archivist>
          (&mut rng, &self.params.sample_params, naive_state.input);
        self.naive_state.input = Some((naive_input_edited, naive_editst)); // Save the input and input-editing state

        // Run DCG Version
        //println!("DCG - - - - - ");
        let _ = use_engine(dcg_state.engine); // Restore saved DCG
        assert!(engine_is_dcg()); // This really is the DCG version
        let mut rng = self.rng.clone(); // Restore Rng
        let (dcg_output, dcg_input_edited, dcg_editst, dcg_sample) = 
          get_engine_sample::<rand::StdRng,Input,EditSt,Output,Editor,Archivist>
          (&mut rng, &self.params.sample_params, dcg_state.input);
        self.dcg_state.engine = use_engine(Engine::Naive); // Swap out the DCG
        self.dcg_state.input = Some((dcg_input_edited, dcg_editst)); // Save the input and input-editing state
        
        // Save the Rng for the next sample.
        self.rng = Box::new(*rng);

        // Compare the two outputs for equality
        let output_valid = if self.params.sample_params.validate_output { 
          Some ( dcg_output == naive_output )
        } else { None } ;

        let sample = Sample{
          //params:self.params.sample_params.clone(),
          batch_name:self.change_batch_num,
          dcg_sample,
          naive_sample,
          output_valid,
        };
        self.change_batch_num += 1;
        Some(sample)
      }
    }
  }

/// Lab experiment implementation: Implements the LabDef trait for any
/// LabArchivist instantiation.
impl<Input:Clone+Debug,EditSt,Output:Eq+Debug,
     Editor:'static+Generate<Input>+Edit<Input,EditSt>,
     Archivist:'static+ComputeDemand<Input,Output>>
  Lab for LabDef<Input,EditSt,Output,Editor,Archivist> {
    fn name(self:&Self) -> Name { self.identity.clone() }
    fn url(self:&Self) -> &Option<String> { &self.url }
    fn run(self:&Self, params:&LabParams) -> LabResults 
    {            
      let mut st = get_sample_gen::<Input,EditSt,Output,Editor,Archivist>(params);
      loop {
        //println!("{:?}", self.name());
        let sample = (&mut st).sample();
        //println!("{:?}", sample);        

        // TODO: Dump to CSV/Tab-separated file; e.g., for GNUPLOT
        match sample {
          Some(s) => {st.samples.push(s); continue},
          None => break,
        }
      };
      return LabResults {
        samples: st.samples,
      }
    }
  }
