//! Data flow analysis: reaching definitions, live variables, use-def chains.
//!
//! Used by taint analysis to track how values propagate through the code.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A point in the program where data flow is analyzed
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ProgramPoint {
    pub function: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Definition of a variable at a program point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub variable: String,
    pub location: ProgramPoint,
    pub value_source: ValueSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueSource {
    Literal(String),
    FunctionCall { name: String, args: Vec<String> },
    Parameter(usize),      // function parameter index
    UserInput(String),     // source type: "http_request", "file_read", "env_var"
    Propagation(String),   // propagated from another variable
    Unknown,
}

/// Data flow graph for a function
#[derive(Debug, Clone)]
pub struct DataFlowGraph {
    pub function_name: String,
    pub definitions: Vec<Definition>,
    pub use_def_chains: HashMap<String, Vec<usize>>,  // variable -> def IDs
    pub reaching_defs: HashMap<ProgramPoint, HashSet<usize>>,
}

impl DataFlowGraph {
    pub fn new(function_name: &str) -> Self {
        Self {
            function_name: function_name.to_string(),
            definitions: Vec::new(),
            use_def_chains: HashMap::new(),
            reaching_defs: HashMap::new(),
        }
    }

    /// Add a variable definition
    pub fn add_definition(&mut self, def: Definition) -> usize {
        let id = self.definitions.len();
        self.use_def_chains
            .entry(def.variable.clone())
            .or_default()
            .push(id);
        self.definitions.push(def);
        id
    }

    /// Get all definitions that reach a given program point
    pub fn get_reaching_definitions(&self, point: &ProgramPoint) -> HashSet<usize> {
        self.reaching_defs.get(point).cloned().unwrap_or_default()
    }

    /// Compute reaching definitions via iterative data flow analysis
    pub fn compute_reaching_definitions(&mut self) {
        // Initialize: GEN and KILL sets per program point
        let mut gen: HashMap<ProgramPoint, HashSet<usize>> = HashMap::new();
        let mut kill: HashMap<ProgramPoint, HashSet<usize>> = HashMap::new();

        for (id, def) in self.definitions.iter().enumerate() {
            let gen_set = gen.entry(def.location.clone()).or_default();
            gen_set.insert(id);

            let kill_set = kill.entry(def.location.clone()).or_default();
            for (other_id, other_def) in self.definitions.iter().enumerate() {
                if other_def.variable == def.variable && other_id != id {
                    kill_set.insert(other_id);
                }
            }
        }

        // Iterative algorithm: IN[B] = ∪ OUT[predecessors]
        // OUT[B] = GEN[B] ∪ (IN[B] - KILL[B])
        // Simplified single-function version
        for def in &self.definitions {
            let mut reaching: HashSet<usize> = self.definitions.iter()
                .enumerate()
                .filter(|(id, _)| *id != self.definitions.len() - 1)
                .map(|(id, _)| id)
                .collect();

            if let Some(kill_set) = kill.get(&def.location) {
                for k in kill_set {
                    reaching.remove(k);
                }
            }
            if let Some(gen_set) = gen.get(&def.location) {
                reaching.extend(gen_set);
            }

            self.reaching_defs.insert(def.location.clone(), reaching);
        }
    }

    /// Trace back from a variable use to find possible source definitions
    pub fn trace_to_source(&self, variable: &str) -> Vec<ValueSource> {
        self.use_def_chains
            .get(variable)
            .map(|def_ids| {
                def_ids
                    .iter()
                    .filter_map(|&id| self.definitions.get(id))
                    .map(|d| d.value_source.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}
