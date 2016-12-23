use adapton::engine::Cnt; // Counters for engine costs
use adapton::engine::Name; // Names, for naming things uniquely
use rand::Rng;
use std::marker::PhantomData;

/// A bit that controls how names are placed in the input; See `README.md` for more.
#[derive(Clone,Debug)]
pub enum NominalStrategy {
  Regular,
  ByContent,
}

/// Parameters for generating and editing input; See `README.md` for more.
#[derive(Clone,Debug)]
pub struct GenerateParams {
  pub size: usize, 
  pub gauge: usize, 
  pub nominal_strategy:NominalStrategy
}

/// Generic method for generating a random input.
/// See `README.md` for more.
pub trait Generate<T> {
  fn generate<R:Rng>(rng:&mut R, params:&GenerateParams) -> T;
} 

/// Generic process for editing an input randomly, in a stateful sequence of edits.
/// See `README.md` for more.
pub trait Edit<T,S> : Clone {
  fn edit_init<R:Rng>(rng:&mut R, params:&GenerateParams) -> S;
  fn edit<R:Rng>(pre_edit:T, edit_state:S, rng:&mut R, params:&GenerateParams) -> (T, S);
}

/// Generic notion of a computation to run naively and incrementally.
/// It has specific `Input` and `Output` types, and a way to `compute`
/// the `Output` from the `Input`.
/// See `README.md` for more.
pub trait Compute<Input,Output> {
  fn compute(Input) -> Output;
}

/// Generic notion of an Incremental Computation to Evaluate and Test.
/// We instantiate this structure once for each test in our test suite.
/// We implement the `LabExp` trait generically for this structure.
pub struct TestComputer<Input,EditSt,Output,
                        InputDist:Generate<Input>+Edit<Input,EditSt>,
                        Computer:Compute<Input,Output>> {
  pub identity:  Name,
  pub computer:  PhantomData<Computer>,
  pub input:     PhantomData<Input>,
  pub editst:    PhantomData<EditSt>,
  pub inputdist: PhantomData<InputDist>,
  pub output:    PhantomData<Output>
}

/// Parameters to running a single lab experiment.
#[derive(Clone,Debug)]
pub struct LabExpParams {
  pub sample_params: SampleParams,
  // TODO: Pretty-print input and output structures; graphmovie dump of experiment
  /// Number of change-batches to perform in a loop; each is interposed with computing the new output.
  pub change_batch_loopc: usize,
}

/// Parameters for collecting a single sample.  In addition to these
/// parameters, the experiment maintains a Rng based on the
/// input_seeds, below; this Rng is given to Edit::edit to generate
/// psuedo-random edits, in batches.  For each engine, this Rng is
/// sequenced across successive samples.  Given an input_seeds vector,
/// there is one unique Rng sequence for each engine's sequence of
/// samples.
#[derive(Clone,Debug)]
pub struct SampleParams {
  /// We convert this seed into a random-number-generator before generating and editing.
  pub input_seeds:       Vec<usize>, 
  /// Other parameters for generating the input.
  pub generate_params:   GenerateParams, 
  /// Whether to validate the output after each computation using the naive and DCG engines
  pub validate_output:   bool,
  /// Size of each batch of changes.
  pub change_batch_size: usize,
}

/// The result of a lab is a sequence of samples.
#[derive(Clone,Debug)]
pub struct LabExpResults {
  pub samples: Vec<Sample>
}

/// The experiment consists of a loop over samples.  For each sample,
/// we switch back and forth between using the Naive engine, and using
/// the DCG engine.  We want to interleave this way for each sample in
/// order to compare outputs and metrics (counts and timings) on a
/// fine-grained scale.
#[derive(Clone,Debug)]
pub struct Sample {
  pub params:       SampleParams,
  pub batch_name:   usize,   // Index/name the change batches; one sample per compute + change batch
  pub dcg_sample:   EngineSample,
  pub naive_sample: EngineSample,
  pub output_valid: Option<bool>
}

/// To sample a single engine, we record metrics for processing the
/// input (left vertical edge in `README.md` diagram).
#[derive(Clone,Debug)]
pub struct EngineSample {
  pub process_input:    EngineMetrics,
  pub compute_output:   EngineMetrics,
}

/// For each engine, for each sampled subcomputation, we record the
/// real time (in nanoseconds) and engine-based counters for DCG costs.
#[derive(Clone,Debug)]
pub struct EngineMetrics {
  pub time_ns:    u64,
  pub engine_cnt: Cnt,
}
