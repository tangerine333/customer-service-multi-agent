//! PyO3 exposed types for cross-language data exchange.
//!
//! Uses JSON Schema as Intermediate Representation (IR) between Rust and Python,
//! avoiding duplicate type definitions on both sides.

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Analysis result passed back to Python
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyAnalysisResult {
    #[pyo3(get)]
    pub symbols: Vec<PySymbolInfo>,
    #[pyo3(get)]
    pub call_edges: Vec<PyCallEdge>,
    #[pyo3(get)]
    pub taint_flows: Vec<PyTaintFlow>,
    #[pyo3(get)]
    pub impacted_files: Vec<String>,
    #[pyo3(get)]
    pub analysis_time_ms: u64,
}

#[pymethods]
impl PyAnalysisResult {
    #[new]
    fn new() -> Self {
        Self {
            symbols: Vec::new(),
            call_edges: Vec::new(),
            taint_flows: Vec::new(),
            impacted_files: Vec::new(),
            analysis_time_ms: 0,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AnalysisResult(symbols={}, edges={}, flows={}, time={}ms)",
            self.symbols.len(),
            self.call_edges.len(),
            self.taint_flows.len(),
            self.analysis_time_ms
        )
    }
}

/// Symbol information (function, class, variable)
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PySymbolInfo {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub file_path: String,
    #[pyo3(get)]
    pub line_start: usize,
    #[pyo3(get)]
    pub line_end: usize,
    #[pyo3(get)]
    pub signature: Option<String>,
    #[pyo3(get)]
    pub return_type: Option<String>,
    #[pyo3(get)]
    pub visibility: String,
}

#[pymethods]
impl PySymbolInfo {
    #[new]
    fn new(
        name: String, kind: String, file_path: String,
        line_start: usize, line_end: usize,
    ) -> Self {
        Self {
            name, kind, file_path, line_start, line_end,
            signature: None, return_type: None,
            visibility: "unknown".to_string(),
        }
    }
}

/// Call graph edge
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyCallEdge {
    #[pyo3(get)]
    pub caller: String,
    #[pyo3(get)]
    pub callee: String,
    #[pyo3(get)]
    pub call_line: usize,
    #[pyo3(get)]
    pub call_type: String,
}

#[pymethods]
impl PyCallEdge {
    #[new]
    fn new(caller: String, callee: String, call_line: usize, call_type: String) -> Self {
        Self { caller, callee, call_line, call_type }
    }
}

/// Taint analysis flow
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyTaintFlow {
    #[pyo3(get)]
    pub source_type: String,
    #[pyo3(get)]
    pub source_location: String,
    #[pyo3(get)]
    pub sink_type: String,
    #[pyo3(get)]
    pub sink_location: String,
    #[pyo3(get)]
    pub sanitized: bool,
    #[pyo3(get)]
    pub confidence: f64,
}

#[pymethods]
impl PyTaintFlow {
    #[new]
    fn new(
        source_type: String, source_location: String,
        sink_type: String, sink_location: String,
        sanitized: bool, confidence: f64,
    ) -> Self {
        Self { source_type, source_location, sink_type, sink_location, sanitized, confidence }
    }
}

/// Function taint summary (for cross-function caching)
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyTaintSummary {
    #[pyo3(get)]
    pub function_name: String,
    #[pyo3(get)]
    pub return_tainted: bool,
    #[pyo3(get)]
    pub tainted_params: Vec<usize>,
    #[pyo3(get)]
    pub summary_hash: String,
}

#[pymethods]
impl PyTaintSummary {
    #[new]
    fn new(function_name: String, return_tainted: bool) -> Self {
        Self {
            function_name,
            return_tainted,
            tainted_params: Vec::new(),
            summary_hash: String::new(),
        }
    }
}
