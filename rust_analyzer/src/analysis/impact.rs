//! Change impact analysis using the code knowledge graph.
//!
//! Given a set of changed functions, computes the impact radius
//! (callers, callees, dependent modules) within N-hop range.

use crate::graph::{CodeGraph, GraphNode};
use std::collections::HashSet;

/// Result of impact analysis on a set of changes
#[derive(Debug, Clone)]
pub struct ImpactResult {
    pub changed_functions: Vec<String>,
    pub direct_callers: Vec<GraphNode>,
    pub direct_callees: Vec<GraphNode>,
    pub affected_modules: HashSet<String>,
    pub affected_files: HashSet<String>,
    pub impact_radius: usize,          // max hop distance affected
    pub estimated_risk: RiskLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,       // Only test files or internal utilities
    Medium,    // Public API with limited callers
    High,      // Public API with many callers or core module
    Critical,  // Database schema, auth module, or framework core
}

/// Analyze the impact of changing a set of functions
pub fn analyze_impact(
    graph: &CodeGraph,
    changed_functions: &[String],
    hop_limit: usize,
) -> ImpactResult {
    let mut direct_callers = Vec::new();
    let mut direct_callees = Vec::new();
    let mut affected_modules = HashSet::new();
    let mut affected_files = HashSet::new();
    let mut all_affected = HashSet::new();

    for func_name in changed_functions {
        // Upstream: who calls us?
        for caller in graph.get_callers(func_name, 1) {
            if !changed_functions.contains(&caller.name) {
                direct_callers.push(caller.clone());
                all_affected.insert(caller.id);
                affected_files.insert(caller.file_path.clone());
                if let Some(module) = caller.file_path.split('/').next() {
                    affected_modules.insert(module.to_string());
                }
            }
        }

        // Downstream: who do we call?
        for callee in graph.get_callees(func_name, 1) {
            if !changed_functions.contains(&callee.name) {
                direct_callees.push(callee.clone());
                all_affected.insert(callee.id);
                affected_files.insert(callee.file_path.clone());
            }
        }

        // Extended impact: N-hop range
        if hop_limit > 1 {
            let subgraph = graph.build_impact_subgraph(&[func_name.clone()]);
            for node in &subgraph.nodes {
                all_affected.insert(node.id);
                affected_files.insert(node.file_path.clone());
                if let Some(module) = node.file_path.split('/').next() {
                    affected_modules.insert(module.to_string());
                }
            }
        }
    }

    let risk = assess_risk(changed_functions, &direct_callers, &affected_modules);

    ImpactResult {
        changed_functions: changed_functions.to_vec(),
        direct_callers,
        direct_callees,
        affected_modules,
        affected_files,
        impact_radius: hop_limit,
        estimated_risk: risk,
    }
}

/// Estimate risk level based on impact breadth
fn assess_risk(
    changed_functions: &[String],
    callers: &[GraphNode],
    modules: &HashSet<String>,
) -> RiskLevel {
    let critical_modules = ["auth", "security", "database", "payment", "core"];
    let is_critical = modules.iter().any(|m| critical_modules.contains(&m.as_str()));

    if is_critical {
        return RiskLevel::Critical;
    }

    if callers.len() > 20 {
        return RiskLevel::High;
    }

    if callers.len() > 5 || !callers.is_empty() && modules.len() > 3 {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}
