//! Main PyO3 analyzer class - the primary Python-facing API.

use pyo3::prelude::*;
use std::time::Instant;

use super::types::{
    PyAnalysisResult, PyCallEdge, PySymbolInfo, PyTaintFlow, PyTaintSummary,
};
use crate::analysis::{TaintAnalyzer, TypeInference};
use crate::analysis::diff_analyzer::{parse_diff, DiffAnalysis};
use crate::analysis::impact::{analyze_impact, ImpactResult, RiskLevel};
use crate::graph::call_graph::CallGraphBuilder;
use crate::graph::CodeGraph;
use crate::parser::MultiParser;

/// Main entry point for Python code to use the Rust analysis engine
#[pyclass]
pub struct PyCodeAnalyzer {
    parser: MultiParser,
    call_graph: Option<CodeGraph>,
    taint_analyzer: TaintAnalyzer,
    type_inference: TypeInference,
}

#[pymethods]
impl PyCodeAnalyzer {
    #[new]
    fn new() -> Self {
        Self {
            parser: MultiParser::new(),
            call_graph: None,
            taint_analyzer: TaintAnalyzer::new(),
            type_inference: TypeInference::new(),
        }
    }

    /// Parse a single source file and extract symbols
    fn parse_file(&mut self, file_path: &str, source_code: &str) -> PyResult<Vec<PySymbolInfo>> {
        let parsed = self.parser.parse(file_path, source_code);
        match parsed {
            Some(file) => {
                let symbols: Vec<PySymbolInfo> = file.symbols.iter().map(|s| {
                    PySymbolInfo {
                        name: s.name.clone(),
                        kind: format!("{:?}", s.kind),
                        file_path: s.location.file.clone(),
                        line_start: s.location.line_start,
                        line_end: s.location.line_end,
                        signature: s.signature.clone(),
                        return_type: None,
                        visibility: format!("{:?}", s.visibility),
                    }
                }).collect();
                Ok(symbols)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Build call graph from multiple files
    fn build_call_graph(&mut self, files: Vec<(String, String)>) -> PyResult<usize> {
        let mut builder = CallGraphBuilder::new();
        let graph = builder.build(&files);
        let node_count = graph.nodes.len();
        self.call_graph = Some(graph);
        Ok(node_count)
    }

    /// Analyze a git diff for changes and impact
    fn analyze_diff(&self, diff_text: &str) -> PyResult<PyAnalysisResult> {
        let start = Instant::now();
        let diff = parse_diff(diff_text);
        let mut result = PyAnalysisResult::new();

        result.impacted_files = diff.files_changed.iter().map(|f| f.path.clone()).collect();

        result.analysis_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// Analyze taint flows for a specific function (by name)
    fn analyze_taint(&mut self, function_name: &str) -> PyResult<Vec<PyTaintFlow>> {
        // Use data flow graph from the call graph to trace taint
        let flows = Vec::new(); // Would integrate with DFG from call graph
        Ok(flows)
    }

    /// Get callers of a function (upstream call graph traversal)
    fn get_callers(&self, function_name: &str, max_hops: usize) -> PyResult<Vec<String>> {
        match &self.call_graph {
            Some(graph) => {
                let callers: Vec<String> = graph
                    .get_callers(function_name, max_hops)
                    .iter()
                    .map(|n| format!("{} ({}:{})", n.name, n.file_path, n.line))
                    .collect();
                Ok(callers)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Get callees of a function (downstream call graph traversal)
    fn get_callees(&self, function_name: &str, max_hops: usize) -> PyResult<Vec<String>> {
        match &self.call_graph {
            Some(graph) => {
                let callees: Vec<String> = graph
                    .get_callees(function_name, max_hops)
                    .iter()
                    .map(|n| format!("{} ({}:{})", n.name, n.file_path, n.line))
                    .collect();
                Ok(callees)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Build impact subgraph for changed functions
    fn build_impact_subgraph(&self, changed_functions: Vec<String>) -> PyResult<Vec<String>> {
        match &self.call_graph {
            Some(graph) => {
                let subgraph = graph.build_impact_subgraph(&changed_functions);
                let affected: Vec<String> = subgraph
                    .nodes
                    .iter()
                    .map(|n| format!("{} ({}:{})", n.name, n.file_path, n.line))
                    .collect();
                Ok(affected)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Infer the type of a variable from context
    fn infer_type(&mut self, variable_name: &str, context: &str) -> PyResult<String> {
        let ty = TypeInference::infer_literal(context.trim());
        self.type_inference.add_variable(variable_name, ty.clone());
        Ok(format!("{:?}", ty))
    }

    /// Get cache statistics
    fn cache_stats(&self) -> PyResult<String> {
        Ok(format!("Cache entries: N/A (managed by Python side)"))
    }
}
