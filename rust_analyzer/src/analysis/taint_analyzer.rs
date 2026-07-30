//! Taint analysis: Source → Sanitizer → Sink tracking.
//!
//! Implements the Source-Sink model (inspired by Coverity/CodeQL):
//! 1. Identify taint SOURCES (user input, file reads, network, env vars)
//! 2. Track propagation through function calls and assignments
//! 3. Check if taint reaches a SINK (SQL, command exec, file write, HTML output)
//! 4. Look for SANITIZERS (param bindings, escapes, validators) on the path

mod summary_cache;

pub use summary_cache::SummaryCache;

use crate::graph::dataflow::{DataFlowGraph, Definition, ValueSource};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Taint source types (where untrusted data enters the system)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaintSource {
    HttpRequest,       // req.body, req.query, request.params
    FileRead,          // open().read(), fs.readFileSync()
    EnvironmentVariable, // process.env, os.environ
    DatabaseRead,      // potentially tainted if no validation on write
    NetworkSocket,     // socket.recv()
    CliArgument,       // sys.argv, process.argv
}

/// Taint sink types (where tainted data causes harm)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaintSink {
    SqlQuery,
    OsCommand,
    FileWrite,
    HtmlOutput,
    Eval,
    Deserialization,
    LdapQuery,
    XPathQuery,
}

/// Sanitizer (validates or escapes tainted data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sanitizer {
    pub function: String,
    pub kind: SanitizerKind,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SanitizerKind {
    ParameterizedQuery,   // prepared statements
    HtmlEscape,           // htmlspecialchars, escapeHtml
    ShellEscape,          // shlex.quote
    Validation,           // isNaN, validator.isEmail
    TypeCast,             // int(), parseFloat()
    RegexFilter,          // whitelist regex
}

/// Result of taint analysis for a single flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintFlow {
    pub source: TaintSource,
    pub source_location: String,
    pub sink: TaintSink,
    pub sink_location: String,
    pub propagated_through: Vec<String>,   // intermediate functions
    pub sanitized: bool,
    pub sanitizer: Option<Sanitizer>,
    pub confidence: f64,                   // 0.0 - 1.0
}

/// Summary of a function's taint behavior (for cross-function caching)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTaintSummary {
    pub function_name: String,
    pub file_path: String,
    pub params: Vec<ParamTaintInfo>,
    pub return_tainted: bool,
    pub hash: String,                      // for cache invalidation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamTaintInfo {
    pub param_index: usize,
    pub param_name: String,
    pub propagates_to_return: bool,
    pub reaches_sinks: Vec<TaintSink>,
}

/// Main taint analyzer
pub struct TaintAnalyzer {
    known_sources: HashSet<String>,
    known_sinks: HashMap<String, TaintSink>,
    known_sanitizers: HashMap<String, SanitizerKind>,
    summary_cache: SummaryCache,
}

impl TaintAnalyzer {
    pub fn new() -> Self {
        let mut known_sources = HashSet::new();
        // HTTP request sources
        for s in ["request.body", "request.query", "request.params", "request.form",
                  "req.body", "req.query", "req.params", "@RequestBody",
                  "request.get_json", "request.args", "request.form.get"] {
            known_sources.insert(s.to_string());
        }
        // File sources
        for s in ["open(", "fs.readFile", "fs.readFileSync", "read_to_string"] {
            known_sources.insert(s.to_string());
        }
        // Environment
        for s in ["os.environ", "process.env", "System.getenv", "getenv"] {
            known_sources.insert(s.to_string());
        }
        // CLI
        for s in ["sys.argv", "process.argv", "args[", "clap::"] {
            known_sources.insert(s.to_string());
        }

        let mut known_sinks = HashMap::new();
        known_sinks.insert("execute(".to_string(), TaintSink::SqlQuery);
        known_sinks.insert("cursor.execute".to_string(), TaintSink::SqlQuery);
        known_sinks.insert("connection.query".to_string(), TaintSink::SqlQuery);
        known_sinks.insert("os.system".to_string(), TaintSink::OsCommand);
        known_sinks.insert("subprocess.call".to_string(), TaintSink::OsCommand);
        known_sinks.insert("subprocess.Popen".to_string(), TaintSink::OsCommand);
        known_sinks.insert("eval(".to_string(), TaintSink::Eval);
        known_sinks.insert("innerHTML".to_string(), TaintSink::HtmlOutput);
        known_sinks.insert("document.write".to_string(), TaintSink::HtmlOutput);
        known_sinks.insert("open(".to_string(), TaintSink::FileWrite);
        known_sinks.insert("pickle.loads".to_string(), TaintSink::Deserialization);
        known_sinks.insert("yaml.load".to_string(), TaintSink::Deserialization);

        let mut known_sanitizers = HashMap::new();
        known_sanitizers.insert("html.escape".to_string(), SanitizerKind::HtmlEscape);
        known_sanitizers.insert("escapeHtml".to_string(), SanitizerKind::HtmlEscape);
        known_sanitizers.insert("shlex.quote".to_string(), SanitizerKind::ShellEscape);
        known_sanitizers.insert("int(".to_string(), SanitizerKind::TypeCast);
        known_sanitizers.insert("float(".to_string(), SanitizerKind::TypeCast);
        known_sanitizers.insert("re.match".to_string(), SanitizerKind::RegexFilter);

        Self {
            known_sources,
            known_sinks,
            known_sanitizers,
            summary_cache: SummaryCache::new(),
        }
    }

    /// Analyze taint flow for a function
    pub fn analyze(&mut self, dfg: &DataFlowGraph, function_name: &str) -> Vec<TaintFlow> {
        // Check cache first
        if let Some(cached) = self.summary_cache.get(function_name) {
            return self.flows_from_summary(&cached, dfg);
        }

        let mut flows = Vec::new();

        // Find all definitions that originate from taint sources
        let tainted_defs: Vec<&Definition> = dfg
            .definitions
            .iter()
            .filter(|d| self.is_taint_source(&d.value_source))
            .collect();

        for tainted in &tainted_defs {
            // Trace where this tainted value flows
            let propagated = self.trace_value_flow(dfg, &tainted.variable);

            // Check if it reaches any sink
            for sink_pattern in self.known_sinks.keys() {
                if propagated.iter().any(|p| p.contains(sink_pattern.as_str())) {
                    let sanitized = propagated.iter().any(|p| {
                        self.known_sanitizers.keys().any(|s| p.contains(s.as_str()))
                    });

                    let flow = TaintFlow {
                        source: self.classify_source(&tainted.value_source),
                        source_location: format!("{}:{}",
                            tainted.location.file, tainted.location.line),
                        sink: self.known_sinks[sink_pattern].clone(),
                        sink_location: propagated.last().cloned().unwrap_or_default(),
                        propagated_through: propagated.clone(),
                        sanitized,
                        sanitizer: if sanitized {
                            self.find_sanitizer(&propagated)
                        } else {
                            None
                        },
                        confidence: if sanitized { 0.3 } else { 0.85 },
                    };
                    flows.push(flow);
                }
            }
        }

        // Cache the summary for this function
        let summary = self.build_summary(function_name, dfg, &flows);
        self.summary_cache.insert(function_name.to_string(), summary);

        flows
    }

    fn is_taint_source(&self, source: &ValueSource) -> bool {
        match source {
            ValueSource::UserInput(s) => self.known_sources.iter().any(|ks| s.contains(ks.as_str())),
            ValueSource::FunctionCall { name, .. } => {
                self.known_sources.iter().any(|ks| name.contains(ks.as_str()))
            }
            _ => false,
        }
    }

    fn trace_value_flow(&self, dfg: &DataFlowGraph, var_name: &str) -> Vec<String> {
        let mut path = Vec::new();
        let sources = dfg.trace_to_source(var_name);
        for source in &sources {
            match source {
                ValueSource::FunctionCall { name, .. } => path.push(name.clone()),
                ValueSource::Propagation(v) => path.push(v.clone()),
                ValueSource::UserInput(s) => path.push(s.clone()),
                _ => {}
            }
        }
        path
    }

    fn classify_source(&self, source: &ValueSource) -> TaintSource {
        match source {
            ValueSource::UserInput(s) => {
                if s.contains("request") || s.contains("req.") {
                    TaintSource::HttpRequest
                } else if s.contains("file") || s.contains("read") {
                    TaintSource::FileRead
                } else if s.contains("environ") || s.contains("env") {
                    TaintSource::EnvironmentVariable
                } else if s.contains("argv") || s.contains("arg") {
                    TaintSource::CliArgument
                } else {
                    TaintSource::NetworkSocket
                }
            }
            _ => TaintSource::HttpRequest,
        }
    }

    fn find_sanitizer(&self, path: &[String]) -> Option<Sanitizer> {
        for step in path {
            for (name, kind) in &self.known_sanitizers {
                if step.contains(name.as_str()) {
                    return Some(Sanitizer {
                        function: step.clone(),
                        kind: kind.clone(),
                        location: String::new(),
                    });
                }
            }
        }
        None
    }

    fn build_summary(
        &self,
        name: &str,
        dfg: &DataFlowGraph,
        flows: &[TaintFlow],
    ) -> FunctionTaintSummary {
        let return_tainted = flows.iter().any(|f| !f.sanitized);
        FunctionTaintSummary {
            function_name: name.to_string(),
            file_path: String::new(),
            params: Vec::new(),
            return_tainted,
            hash: format!("{:x}", md5::compute(name.as_bytes())),
        }
    }

    fn flows_from_summary(&self, summary: &FunctionTaintSummary, _dfg: &DataFlowGraph) -> Vec<TaintFlow> {
        if !summary.return_tainted {
            return vec![];
        }
        // Reconstruct flows from cached summary
        Vec::new()
    }
}

impl Default for TaintAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
