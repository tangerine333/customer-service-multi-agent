//! Graph construction: Call Graph, Control Flow Graph, Data Flow Graph.

pub mod call_graph;
pub mod control_flow;
pub mod dataflow;

use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

/// Node in any code graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: usize,
    pub name: String,
    pub kind: String,       // function, class, variable, basic_block, expression
    pub file_path: String,
    pub line: usize,
    pub properties: serde_json::Value,
}

/// Edge in any code graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub kind: String,       // calls, references, data_flow, control_flow
    pub line: Option<usize>,
    pub properties: serde_json::Value,
}

/// Represents the full code knowledge graph for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub repo_root: String,
    pub commit_sha: Option<String>,
}

impl CodeGraph {
    pub fn new(repo_root: &str) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            repo_root: repo_root.to_string(),
            commit_sha: None,
        }
    }

    /// Find all callers of a function (upstream, N hops)
    pub fn get_callers(&self, target_name: &str, max_hops: usize) -> Vec<&GraphNode> {
        let target_ids: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| n.name == target_name)
            .map(|n| n.id)
            .collect();

        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut current = target_ids.clone();

        for _ in 0..max_hops {
            let callers: Vec<usize> = self
                .edges
                .iter()
                .filter(|e| current.contains(&e.to) && e.kind == "calls")
                .map(|e| e.from)
                .collect();

            current.clear();
            for caller_id in callers {
                if visited.insert(caller_id) {
                    if let Some(node) = self.nodes.iter().find(|n| n.id == caller_id) {
                        result.push(node);
                        current.push(caller_id);
                    }
                }
            }
        }

        result
    }

    /// Find all callees of a function (downstream, N hops)
    pub fn get_callees(&self, source_name: &str, max_hops: usize) -> Vec<&GraphNode> {
        let source_ids: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| n.name == source_name)
            .map(|n| n.id)
            .collect();

        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut current = source_ids.clone();

        for _ in 0..max_hops {
            let callees: Vec<usize> = self
                .edges
                .iter()
                .filter(|e| current.contains(&e.from) && e.kind == "calls")
                .map(|e| e.to)
                .collect();

            current.clear();
            for callee_id in callees {
                if visited.insert(callee_id) {
                    if let Some(node) = self.nodes.iter().find(|n| n.id == callee_id) {
                        result.push(node);
                        current.push(callee_id);
                    }
                }
            }
        }

        result
    }

    /// Build impact subgraph centered on changed functions (2-hop range)
    pub fn build_impact_subgraph(&self, changed_functions: &[String]) -> CodeGraph {
        let mut affected_ids = std::collections::HashSet::new();

        for func_name in changed_functions {
            // Add the function itself
            for node in &self.nodes {
                if node.name == *func_name {
                    affected_ids.insert(node.id);
                }
            }
            // Add 2-hop callers
            for caller in self.get_callers(func_name, 2) {
                affected_ids.insert(caller.id);
            }
            // Add 2-hop callees
            for callee in self.get_callees(func_name, 2) {
                affected_ids.insert(callee.id);
            }
        }

        let nodes: Vec<GraphNode> = self
            .nodes
            .iter()
            .filter(|n| affected_ids.contains(&n.id))
            .cloned()
            .collect();

        let edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .filter(|e| affected_ids.contains(&e.from) && affected_ids.contains(&e.to))
            .cloned()
            .collect();

        CodeGraph {
            nodes,
            edges,
            repo_root: self.repo_root.clone(),
            commit_sha: self.commit_sha.clone(),
        }
    }

    /// Filter to only include nodes reachable within N hops from seeds
    pub fn filter_hops(&self, seed_names: &[String], max_hops: usize) -> CodeGraph {
        // Use BFS from seeds, respecting edge direction
        let seed_ids: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| seed_names.contains(&n.name))
            .map(|n| n.id)
            .collect();

        let mut reachable = std::collections::HashSet::new();
        let mut queue: Vec<(usize, usize)> = seed_ids.iter().map(|&id| (id, 0)).collect();

        while let Some((node_id, hop)) = queue.pop() {
            if hop > max_hops || !reachable.insert(node_id) {
                continue;
            }
            // Traverse both directions
            for edge in &self.edges {
                if edge.from == node_id && !reachable.contains(&edge.to) {
                    queue.push((edge.to, hop + 1));
                }
                if edge.to == node_id && !reachable.contains(&edge.from) {
                    queue.push((edge.from, hop + 1));
                }
            }
        }

        CodeGraph {
            nodes: self.nodes.iter().filter(|n| reachable.contains(&n.id)).cloned().collect(),
            edges: self.edges.iter().filter(|e| reachable.contains(&e.from) && reachable.contains(&e.to)).cloned().collect(),
            repo_root: self.repo_root.clone(),
            commit_sha: self.commit_sha.clone(),
        }
    }
}
