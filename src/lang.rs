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
    Python(PythonProgItem),
}

impl From<ProgItem> for ProgLanguage {
    fn from(value: ProgItem) -> Self {
        match value {
            ProgItem::Rust(_) => ProgLanguage::Rust,
            ProgItem::Python(_) => ProgLanguage::Python,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum PythonProgItem {
    Function,
    Class,
    Method,
    Decorator,
    Generator,
    Comprehension,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum RustProgItem {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Macro,
    Const,
    Static,
    TypeAlias,
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
                PythonProgItem::Method => "(class_definition method_definition) @item".into(),
                PythonProgItem::Decorator => "(decorator) @item".into(),
                PythonProgItem::Generator => "(function_definition yield) @item".into(),
                PythonProgItem::Comprehension => {
                    "(list_comprehension set_comprehension dictionary_comprehension) @item".into()
                }
            },
            ProgItem::Rust(item) => match item {
                RustProgItem::Function => "(function_item) @item".into(),
                RustProgItem::Struct => "(struct_item) @item".into(),
                RustProgItem::Enum => "(enum_item) @item".into(),
                RustProgItem::Trait => "(trait_item) @item".into(),
                RustProgItem::Impl => "(impl_item) @item".into(),
                RustProgItem::Macro => "(macro_definition) @item".into(),
                RustProgItem::Const => "(const_item) @item".into(),
                RustProgItem::Static => "(static_item) @item".into(),
                RustProgItem::TypeAlias => "(type_alias) @item".into(),
            },
        }
    }
}
