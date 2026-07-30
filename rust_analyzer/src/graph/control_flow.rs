//! Control Flow Graph construction from AST.
//!
//! Builds basic block representation for functions:
//! - Entry/Exit nodes
//! - Branch nodes (if/else, switch/match)
//! - Loop nodes (for/while/do-while)
//! - Sequential execution edges

use super::{GraphEdge, GraphNode};
use serde_json::json;

/// Basic block types in a control flow graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockKind {
    Entry,
    Exit,
    Statement,
    Branch,
    LoopHeader,
    LoopBody,
    LoopExit,
    SwitchCase,
    Return,
}

/// A basic block in the control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub kind: BlockKind,
    pub label: String,
    pub line_start: usize,
    pub line_end: usize,
    pub statements: Vec<String>,
}

/// Control Flow Graph for a single function
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub function_name: String,
    pub file_path: String,
    pub blocks: Vec<BasicBlock>,
    pub edges: Vec<(usize, usize)>,  // (from_block_id, to_block_id)
    pub entry_id: usize,
    pub exit_id: usize,
}

impl ControlFlowGraph {
    pub fn new(function_name: &str, file_path: &str) -> Self {
        let entry = BasicBlock {
            id: 0,
            kind: BlockKind::Entry,
            label: "entry".to_string(),
            line_start: 0,
            line_end: 0,
            statements: vec![],
        };
        let exit = BasicBlock {
            id: 1,
            kind: BlockKind::Exit,
            label: "exit".to_string(),
            line_start: 0,
            line_end: 0,
            statements: vec![],
        };

        Self {
            function_name: function_name.to_string(),
            file_path: file_path.to_string(),
            blocks: vec![entry, exit],
            edges: vec![],
            entry_id: 0,
            exit_id: 1,
        }
    }

    /// Add a basic block to the CFG
    pub fn add_block(&mut self, kind: BlockKind, label: &str, line: usize) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            id,
            kind,
            label: label.to_string(),
            line_start: line,
            line_end: line,
            statements: vec![],
        });
        id
    }

    /// Add a directed edge between two blocks
    pub fn add_edge(&mut self, from: usize, to: usize) {
        if !self.edges.contains(&(from, to)) {
            self.edges.push((from, to));
        }
    }

    /// Check reachability - can we get from entry to exit?
    pub fn all_paths_reach_exit(&self) -> bool {
        let mut visited = vec![false; self.blocks.len()];
        let mut stack = vec![self.entry_id];
        while let Some(current) = stack.pop() {
            if visited[current] {
                continue;
            }
            visited[current] = true;
            for &(from, to) in &self.edges {
                if from == current {
                    stack.push(to);
                }
            }
        }
        visited[self.exit_id]
    }

    /// Detect unreachable blocks (dead code)
    pub fn find_unreachable_blocks(&self) -> Vec<usize> {
        let mut reachable = vec![false; self.blocks.len()];
        let mut stack = vec![self.entry_id];
        while let Some(current) = stack.pop() {
            if reachable[current] {
                continue;
            }
            reachable[current] = true;
            for &(from, to) in &self.edges {
                if from == current {
                    stack.push(to);
                }
            }
        }
        self.blocks
            .iter()
            .filter(|b| !reachable[b.id])
            .map(|b| b.id)
            .collect()
    }

    /// Convert to generic graph nodes for serialization
    pub fn to_graph(&self) -> (Vec<GraphNode>, Vec<GraphEdge>) {
        let nodes: Vec<GraphNode> = self
            .blocks
            .iter()
            .map(|b| GraphNode {
                id: b.id,
                name: b.label.clone(),
                kind: format!("{:?}", b.kind),
                file_path: self.file_path.clone(),
                line: b.line_start,
                properties: json!({
                    "function": self.function_name,
                    "line_end": b.line_end,
                }),
            })
            .collect();

        let edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .map(|(from, to)| GraphEdge {
                from: *from,
                to: *to,
                kind: "control_flow".to_string(),
                line: None,
                properties: json!({}),
            })
            .collect();

        (nodes, edges)
    }
}
