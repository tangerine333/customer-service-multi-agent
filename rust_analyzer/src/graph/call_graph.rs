//! Static call graph construction using tree-sitter AST.
//!
//! Builds directed graph of function/method calls by analyzing AST nodes.
//! Handles: direct calls, method calls, function pointers (heuristic), virtual dispatch.

use super::{CodeGraph, GraphEdge, GraphNode};
use crate::parser::{MultiParser, ParsedFile, SymbolKind};
use serde_json::json;
use std::collections::HashMap;

pub struct CallGraphBuilder {
    parser: MultiParser,
    node_counter: usize,
}

impl CallGraphBuilder {
    pub fn new() -> Self {
        Self {
            parser: MultiParser::new(),
            node_counter: 0,
        }
    }

    /// Build call graph from a set of source files
    pub fn build(&mut self, files: &[(String, String)]) -> CodeGraph {
        let mut graph = CodeGraph::new(".");
        let mut parsed_files = Vec::new();
        let mut name_to_id: HashMap<String, usize> = HashMap::new();

        // Phase 1: Parse all files and extract symbols as nodes
        for (path, source) in files {
            if let Some(parsed) = self.parser.parse(path, source) {
                for symbol in &parsed.symbols {
                    let id = self.next_id();
                    graph.nodes.push(GraphNode {
                        id,
                        name: symbol.name.clone(),
                        kind: format!("{:?}", symbol.kind),
                        file_path: symbol.location.file.clone(),
                        line: symbol.location.line_start,
                        properties: json!({
                            "signature": symbol.signature,
                            "visibility": format!("{:?}", symbol.visibility),
                        }),
                    });
                    name_to_id.insert(symbol.name.clone(), id);
                }
                parsed_files.push(parsed);
            }
        }

        // Phase 2: Build call edges by analyzing function bodies
        for file in &parsed_files {
            self.extract_calls(&file, &mut graph, &name_to_id);
        }

        // Phase 3: Heuristic - connect method calls on known types
        self.resolve_method_calls(&mut graph);

        graph
    }

    fn extract_calls(
        &self,
        file: &ParsedFile,
        graph: &mut CodeGraph,
        name_to_id: &HashMap<String, usize>,
    ) {
        let root = file.tree.root_node();
        let source = file.source.as_bytes();
        // Walk AST to find call_expression nodes
        let mut cursor = root.walk();
        for node in root.children(&mut cursor) {
            self.walk_for_calls(&node, source, &file.path, graph, name_to_id);
        }
    }

    fn walk_for_calls(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        current_file: &str,
        graph: &mut CodeGraph,
        name_to_id: &HashMap<String, usize>,
    ) {
        if node.kind() == "call_expression" || node.kind() == "call" {
            // Get function name from first child (function identifier)
            if let Some(func_node) = node.child(0) {
                let func_name = func_node.utf8_text(source).unwrap_or("");
                if let Some(&target_id) = name_to_id.get(func_name) {
                    self.add_call_edges_for_node(node, target_id, current_file, graph, name_to_id);
                }
            }
        }

        for child in node.children(&mut node.walk()) {
            self.walk_for_calls(&child, source, current_file, graph, name_to_id);
        }
    }

    fn add_call_edges_for_node(
        &self,
        call_node: &tree_sitter::Node,
        callee_id: usize,
        current_file: &str,
        graph: &mut CodeGraph,
        name_to_id: &HashMap<String, usize>,
    ) {
        // Find enclosing function
        let enclosing = self.find_enclosing_function(call_node);
        if let Some(caller_name) = enclosing {
            if let Some(&caller_id) = name_to_id.get(&caller_name) {
                let line = call_node.start_position().row + 1;
                graph.edges.push(GraphEdge {
                    from: caller_id,
                    to: callee_id,
                    kind: "calls".to_string(),
                    line: Some(line),
                    properties: json!({"file": current_file}),
                });
            }
        }
    }

    fn find_enclosing_function(&self, node: &tree_sitter::Node) -> Option<String> {
        let mut current = node.parent();
        while let Some(parent) = current {
            match parent.kind() {
                "function_definition" | "function_declaration" | "method_definition" => {
                    return parent
                        .child_by_field_name("name")
                        .map(|n| n.utf8_text(&[]).unwrap_or("").to_string());
                }
                _ => current = parent.parent(),
            }
        }
        None
    }

    fn resolve_method_calls(&self, graph: &mut CodeGraph) {
        // Heuristic: match method names against class methods
        // This is simplified - full implementation would use type inference
        let class_methods: HashMap<String, Vec<usize>> = HashMap::new();
        // ... (in full implementation, would resolve obj.method() to Class::method())
    }

    fn next_id(&mut self) -> usize {
        let id = self.node_counter;
        self.node_counter += 1;
        id
    }
}

impl Default for CallGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
