//! Rust Code Analysis Engine
//!
//! Core analysis engine using tree-sitter for multi-language parsing,
//! call graph construction, data flow analysis, and taint tracking.

pub mod parser;
pub mod graph;
pub mod analysis;
pub mod ffi;

use pyo3::prelude::*;

/// Python module entry point - exposes Rust analysis engine to Python
#[pymodule]
fn rust_analyzer(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ffi::analyzer::PyCodeAnalyzer>()?;
    m.add_class::<ffi::types::PyAnalysisResult>()?;
    m.add_class::<ffi::types::PySymbolInfo>()?;
    m.add_class::<ffi::types::PyTaintSummary>()?;
    m.add_class::<ffi::types::PyCallEdge>()?;
    Ok(())
}
