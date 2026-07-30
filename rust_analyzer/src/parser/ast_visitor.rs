//! AST visitor for symbol extraction and pattern matching.

use super::{ParsedFile, SourceLocation, Symbol, SymbolKind, SupportedLanguage, Visibility};
use tree_sitter::{Node, Tree};

/// Pattern for matching AST nodes (used by rule engine)
#[derive(Debug, Clone)]
pub struct AstMatch {
    pub pattern_name: String,
    pub matched_node: String,
    pub location: SourceLocation,
    pub captured_groups: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    FunctionCall(String),
    StringConcat,
    BinaryExpression(String),
    Assignment(String),
    Import(String),
    Decorator(String),
}

/// Extract all symbols from a parsed tree
pub fn extract_symbols(
    tree: &Tree,
    source: &str,
    file_path: &str,
    lang: SupportedLanguage,
) -> Vec<Symbol> {
    let mut visitor = AstVisitor::new(source, file_path, lang);
    visitor.visit_tree(tree);
    visitor.symbols
}

pub struct AstVisitor<'a> {
    source: &'a str,
    file_path: String,
    language: SupportedLanguage,
    pub symbols: Vec<Symbol>,
}

impl<'a> AstVisitor<'a> {
    pub fn new(source: &'a str, file_path: &str, language: SupportedLanguage) -> Self {
        Self {
            source,
            file_path: file_path.to_string(),
            language,
            symbols: Vec::new(),
        }
    }

    pub fn visit_tree(&mut self, tree: &Tree) {
        self.visit_node(&tree.root_node());
    }

    fn visit_node(&mut self, node: &Node<'a>) {
        match node.kind() {
            // Function definitions
            "function_definition" | "function_declaration" | "method_definition" => {
                self.extract_function(node);
            }
            // Class definitions
            "class_definition" | "class_declaration" | "struct_item" => {
                self.extract_class(node);
            }
            // Variable declarations
            "variable_declaration" | "assignment" | "let_declaration" => {
                self.extract_variable(node);
            }
            // Import statements
            "import_statement" | "import_declaration" | "use_declaration" => {
                self.extract_import(node);
            }
            _ => {
                for child in node.children(&mut node.walk()) {
                    self.visit_node(&child);
                }
            }
        }
    }

    fn extract_function(&mut self, node: &Node<'a>) {
        let name_node = node.child_by_field_name("name");
        let name = name_node
            .map(|n| self.node_text(&n))
            .unwrap_or_else(|| "anonymous".to_string());

        let start = node.start_position();
        let end = node.end_position();

        let kind = match self.language {
            SupportedLanguage::Rust => {
                // Check if it's inside an impl block -> method
                SymbolKind::Function
            }
            _ => SymbolKind::Function,
        };

        self.symbols.push(Symbol {
            name,
            kind,
            location: SourceLocation {
                file: self.file_path.clone(),
                line_start: start.row + 1,
                line_end: end.row + 1,
                col_start: start.column,
                col_end: end.column,
            },
            signature: Some(self.extract_signature(node)),
            visibility: self.detect_visibility(node),
        });
    }

    fn extract_class(&mut self, node: &Node<'a>) {
        let name_node = node.child_by_field_name("name");
        if let Some(name_node) = name_node {
            let start = node.start_position();
            let end = node.end_position();

            self.symbols.push(Symbol {
                name: self.node_text(&name_node),
                kind: SymbolKind::Class,
                location: SourceLocation {
                    file: self.file_path.clone(),
                    line_start: start.row + 1,
                    line_end: end.row + 1,
                    col_start: start.column,
                    col_end: end.column,
                },
                signature: None,
                visibility: self.detect_visibility(node),
            });
        }
    }

    fn extract_variable(&mut self, node: &Node<'a>) {
        // Simplified - extract top-level variable names
        if let Some(declarator) = node.child_by_field_name("declarator") {
            if let Some(name) = declarator.child_by_field_name("name") {
                let start = name.start_position();
                self.symbols.push(Symbol {
                    name: self.node_text(&name),
                    kind: SymbolKind::Variable,
                    location: SourceLocation {
                        file: self.file_path.clone(),
                        line_start: start.row + 1,
                        line_end: start.row + 1,
                        col_start: start.column,
                        col_end: start.column,
                    },
                    signature: None,
                    visibility: Visibility::Unknown,
                });
            }
        }
    }

    fn extract_import(&mut self, node: &Node<'a>) {
        // Record imports for dependency tracking
        let start = node.start_position();
        self.symbols.push(Symbol {
            name: self.node_text(node),
            kind: SymbolKind::Import,
            location: SourceLocation {
                file: self.file_path.clone(),
                line_start: start.row + 1,
                line_end: node.end_position().row + 1,
                col_start: start.column,
                col_end: node.end_position().column,
            },
            signature: None,
            visibility: Visibility::Unknown,
        });
    }

    fn extract_signature(&self, node: &Node<'a>) -> String {
        let start = node.start_position();
        let end = node.end_position();
        let lines: Vec<&str> = self.source.lines().collect();
        let sig_line = lines.get(start.row).unwrap_or(&"");
        sig_line.trim().to_string()
    }

    fn detect_visibility(&self, _node: &Node<'a>) -> Visibility {
        // Simplified heuristic based on naming convention
        Visibility::Unknown
    }

    fn node_text(&self, node: &Node<'a>) -> String {
        node.utf8_text(self.source.as_bytes())
            .unwrap_or("")
            .to_string()
    }
}
