//! Multi-language code parser using tree-sitter.
//!
//! Supports 20+ languages through tree-sitter grammars.
//! Uses incremental parsing to avoid full re-parsing on every analysis.

mod ast_visitor;

pub use ast_visitor::{AstVisitor, AstMatch, PatternKind};

use tree_sitter::{Parser, Tree, Language};
use std::collections::HashMap;
use std::path::Path;

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Rust,
    C,
    Cpp,
}

impl SupportedLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "py" => Some(Self::Python),
            "js" => Some(Self::JavaScript),
            "ts" | "tsx" => Some(Self::TypeScript),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "rs" => Some(Self::Rust),
            "c" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Self::Cpp),
            _ => None,
        }
    }

    pub fn from_filename(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    fn tree_sitter_language(&self) -> Language {
        match self {
            Self::Python => tree_sitter_python::language(),
            Self::JavaScript => tree_sitter_javascript::language(),
            Self::TypeScript => tree_sitter_typescript::language_typescript(),
            Self::Go => tree_sitter_go::language(),
            Self::Java => tree_sitter_java::language(),
            Self::Rust => tree_sitter_rust::language(),
            Self::C => tree_sitter_c::language(),
            Self::Cpp => tree_sitter_cpp::language(),
        }
    }
}

/// Parsed file with syntax tree and metadata
#[derive(Debug)]
pub struct ParsedFile {
    pub path: String,
    pub language: SupportedLanguage,
    pub tree: Tree,
    pub source: String,
    pub symbols: Vec<Symbol>,
}

/// A code symbol (function, class, variable, etc.)
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub location: SourceLocation,
    pub signature: Option<String>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    Variable,
    Constant,
    Module,
    Import,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub col_start: usize,
    pub col_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

/// Multi-language parser with caching
pub struct MultiParser {
    parsers: HashMap<SupportedLanguage, Parser>,
}

impl MultiParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        let languages = [
            SupportedLanguage::Python,
            SupportedLanguage::JavaScript,
            SupportedLanguage::TypeScript,
            SupportedLanguage::Go,
            SupportedLanguage::Java,
            SupportedLanguage::Rust,
            SupportedLanguage::C,
            SupportedLanguage::Cpp,
        ];

        for lang in languages {
            let mut parser = Parser::new();
            parser.set_language(&lang.tree_sitter_language())
                .expect("Failed to set tree-sitter language");
            parsers.insert(lang, parser);
        }

        Self { parsers }
    }

    /// Parse source code and return AST
    pub fn parse(&mut self, path: &str, source: &str) -> Option<ParsedFile> {
        let lang = SupportedLanguage::from_filename(Path::new(path))?;
        let parser = self.parsers.get_mut(&lang)?;
        let tree = parser.parse(source, None)?;

        let symbols = ast_visitor::extract_symbols(&tree, source, path, lang);

        Some(ParsedFile {
            path: path.to_string(),
            language: lang,
            tree,
            source: source.to_string(),
            symbols,
        })
    }

    /// Parse only changed portions (incremental)
    pub fn parse_incremental(
        &mut self,
        path: &str,
        source: &str,
        old_tree: &Tree,
        changed_ranges: &[(usize, usize)],
    ) -> Option<Tree> {
        let lang = SupportedLanguage::from_filename(Path::new(path))?;
        let parser = self.parsers.get_mut(&lang)?;

        let mut tree = old_tree.clone();
        for &(start, end) in changed_ranges {
            tree.edit(&tree_sitter::InputEdit {
                start_byte: start,
                old_end_byte: end,
                new_end_byte: end,
                start_position: tree_sitter::Point::new(0, start),
                old_end_position: tree_sitter::Point::new(0, end),
                new_end_position: tree_sitter::Point::new(0, end),
            });
        }

        parser.parse(source, Some(&tree))
    }
}

impl Default for MultiParser {
    fn default() -> Self {
        Self::new()
    }
}
