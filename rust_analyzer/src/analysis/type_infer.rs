//! Hindley-Milner style type inference for dynamic language analysis.
//!
//! Simplified type inference that:
//! 1. Collects type annotations (Python type hints, TypeScript types, Go types)
//! 2. Infers types through usage (function calls, operators, assignments)
//! 3. Tracks type changes across assignments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InferredType {
    Int,
    Float,
    String,
    Bool,
    List(Box<InferredType>),
    Dict(Box<InferredType>, Box<InferredType>),
    Tuple(Vec<InferredType>),
    Function(Vec<InferredType>, Box<InferredType>),
    Optional(Box<InferredType>),
    Union(Vec<InferredType>),
    Custom(String),     // user-defined class/struct
    Any,                // unknown type
    Null,
    Void,
}

/// Type environment mapping variable names to inferred types
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub variables: HashMap<String, InferredType>,
    pub functions: HashMap<String, FunctionType>,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub name: String,
    pub params: Vec<(String, InferredType)>,
    pub return_type: InferredType,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Type inference engine
pub struct TypeInference {
    env: TypeEnv,
}

impl TypeInference {
    pub fn new() -> Self {
        Self {
            env: TypeEnv {
                variables: HashMap::new(),
                functions: HashMap::new(),
            },
        }
    }

    /// Parse type annotation string to InferredType
    pub fn parse_annotation(annotation: &str) -> InferredType {
        match annotation.trim() {
            "int" | "i32" | "i64" | "int32" | "int64" | "integer" => InferredType::Int,
            "float" | "f64" | "f32" | "float64" | "double" | "number" => InferredType::Float,
            "str" | "string" | "String" => InferredType::String,
            "bool" | "boolean" | "bool" => InferredType::Bool,
            "None" | "null" | "nil" | "NoneType" => InferredType::Null,
            "Any" | "any" | "unknown" => InferredType::Any,
            "list" => InferredType::List(Box::new(InferredType::Any)),
            "dict" => InferredType::Dict(Box::new(InferredType::Any), Box::new(InferredType::Any)),
            s if s.starts_with("List[") => {
                let inner = &s[5..s.len()-1];
                InferredType::List(Box::new(Self::parse_annotation(inner)))
            }
            s if s.starts_with("Optional[") => {
                let inner = &s[9..s.len()-1];
                InferredType::Optional(Box::new(Self::parse_annotation(inner)))
            }
            s if s.starts_with("Union[") => {
                let inner = &s[6..s.len()-1];
                let types: Vec<InferredType> = inner
                    .split(',')
                    .map(|t| Self::parse_annotation(t.trim()))
                    .collect();
                InferredType::Union(types)
            }
            s => InferredType::Custom(s.to_string()),
        }
    }

    /// Infer type from literal value
    pub fn infer_literal(value: &str) -> InferredType {
        if value == "True" || value == "False" || value == "true" || value == "false" {
            InferredType::Bool
        } else if let Ok(_) = value.parse::<i64>() {
            InferredType::Int
        } else if let Ok(_) = value.parse::<f64>() {
            InferredType::Float
        } else if value.starts_with('"') || value.starts_with('\'') || value.starts_with('`') {
            InferredType::String
        } else if value == "None" || value == "null" || value == "nil" {
            InferredType::Null
        } else if value.starts_with('[') {
            InferredType::List(Box::new(InferredType::Any))
        } else if value.starts_with('{') {
            InferredType::Dict(Box::new(InferredType::Any), Box::new(InferredType::Any))
        } else {
            InferredType::Any
        }
    }

    /// Add a typed variable to the environment
    pub fn add_variable(&mut self, name: &str, ty: InferredType) {
        self.env.variables.insert(name.to_string(), ty);
    }

    /// Get inferred type for a variable (returns Any if unknown)
    pub fn get_type(&self, name: &str) -> InferredType {
        self.env.variables.get(name).cloned().unwrap_or(InferredType::Any)
    }

    /// Check if two types are compatible
    pub fn is_compatible(a: &InferredType, b: &InferredType) -> bool {
        if a == b {
            return true;
        }
        match (a, b) {
            (InferredType::Any, _) | (_, InferredType::Any) => true,
            (InferredType::Optional(inner), other)
            | (other, InferredType::Optional(inner)) => {
                *inner.as_ref() == *other || matches!(other, InferredType::Null)
            }
            (InferredType::Union(types), other) => types.iter().any(|t| Self::is_compatible(t, other)),
            (other, InferredType::Union(types)) => types.iter().any(|t| Self::is_compatible(other, t)),
            (InferredType::Float, InferredType::Int) | (InferredType::Int, InferredType::Float) => true,
            _ => false,
        }
    }

    /// Get all variables of uncertain type (Any) - potential dynamic language issues
    pub fn get_uncertain_variables(&self) -> Vec<String> {
        self.env
            .variables
            .iter()
            .filter(|(_, ty)| matches!(ty, InferredType::Any))
            .map(|(name, _)| name.clone())
            .collect()
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}
