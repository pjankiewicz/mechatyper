use anyhow::{anyhow, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use tree_sitter::Language;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum ProgLanguage {
    Python,
    Rust,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum ProgItem {
    Rust(RustProgItem),
    Python(PythonProgItem)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum PythonProgItem {
    Function,
    Class
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum RustProgItem {
    Function,
    Struct,
    Enum
}

#[derive(Clone, Debug)]
pub enum LanguageItem {
    Python(PythonItem),
    Rust(RustItem),
}

#[derive(Clone, Debug)]
pub enum PythonItem {
    Function,
    Class,
}

#[derive(Clone, Debug)]
pub enum RustItem {
    Struct,
    Enum,
    Function,
}

impl FromStr for LanguageItem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Python.Function" => Ok(LanguageItem::Python(PythonItem::Function)),
            "Python.Class" => Ok(LanguageItem::Python(PythonItem::Class)),
            "Rust.Struct" => Ok(LanguageItem::Rust(RustItem::Struct)),
            "Rust.Enum" => Ok(LanguageItem::Rust(RustItem::Enum)),
            "Rust.Function" => Ok(LanguageItem::Rust(RustItem::Function)),
            _ => Err(anyhow!("Cannot parse {}", s)),
        }
    }
}

impl FromStr for PythonItem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "function" => Ok(PythonItem::Function),
            "class" => Ok(PythonItem::Class),
            _ => Err(anyhow!("Cannot parse {}", s)),
        }
    }
}

impl FromStr for RustItem {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "struct" => Ok(RustItem::Struct),
            "enum" => Ok(RustItem::Enum),
            "function" => Ok(RustItem::Function),
            _ => Err(anyhow!("Cannot parse {}", s)),
        }
    }
}

impl FromStr for ProgLanguage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "python" => Ok(ProgLanguage::Python),
            "rust" => Ok(ProgLanguage::Rust),
            _ => Err(anyhow!("Cannot parse {}", s)),
        }
    }
}

impl ProgLanguage {
    pub fn tree_sitter_language(&self) -> Language {
        match self {
            ProgLanguage::Python => tree_sitter_python::language(),
            ProgLanguage::Rust => tree_sitter_rust::language(),
        }
    }

    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            ProgLanguage::Python => vec!["py"],
            ProgLanguage::Rust => vec!["rs"],
        }
    }

    pub fn get_excluded_directories(&self) -> Vec<&'static str> {
        match self {
            ProgLanguage::Python => vec!["site-packages", "venv", "__pycache__", ".pytest_cache"],
            ProgLanguage::Rust => vec!["target", ".cargo"],
        }
    }
}

impl ProgItem {
    pub fn to_sexpr(&self) -> String {
        match self {
            ProgItem::Python(item) => match item {
                PythonProgItem::Function => "(function_definition) @item".into(),
                PythonProgItem::Class => "(class_definition) @item".into(),
            },
            ProgItem::Rust(item) => match item {
                RustProgItem::Struct => "(struct_item) @item".into(),
                RustProgItem::Enum => "(enum_item) @item".into(),
                RustProgItem::Function => "(function_item) @item".into(),
            },
        }
    }
}
