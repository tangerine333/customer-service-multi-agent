//! Analysis module: taint analysis, type inference, impact analysis.

pub mod taint_analyzer;
pub mod type_infer;
mod summary_cache;

pub use taint_analyzer::{TaintAnalyzer, TaintFlow, TaintSource, TaintSink, Sanitizer, FunctionTaintSummary};
pub use type_infer::{TypeInference, InferredType, TypeEnv};
