use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::{anyhow, Error};
use tree_sitter::Language;

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

#[derive(Clone, Debug)]
pub enum LanguageEnum {
    Python,
    Rust,
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

impl FromStr for LanguageEnum {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "python" => Ok(LanguageEnum::Python),
            "rust" => Ok(LanguageEnum::Rust),
            _ => Err(anyhow!("Cannot parse {}", s)),
        }
    }
}

impl LanguageEnum {
    pub fn tree_sitter_language(&self) -> Language {
        match self {
            LanguageEnum::Python => tree_sitter_python::language(),
            LanguageEnum::Rust => tree_sitter_rust::language(),
        }
    }

    pub fn file_extensions(&self) -> Vec<&'static str> {
        match self {
            LanguageEnum::Python => vec!["py"],
            LanguageEnum::Rust => vec!["rs"],
        }
    }

    pub fn get_excluded_directories(&self) -> Vec<&'static str> {
        match self {
            LanguageEnum::Python => vec!["site-packages", "venv", "__pycache__", ".pytest_cache"],
            LanguageEnum::Rust => vec!["target", ".cargo"],
        }
    }
}

impl LanguageItem {
    pub fn to_sexpr(&self) -> String {
        match self {
            LanguageItem::Python(item) => match item {
                PythonItem::Function => "(function_definition) @item".into(),
                PythonItem::Class => "(class_definition) @item".into(),
            },
            LanguageItem::Rust(item) => match item {
                RustItem::Struct => "(struct_item) @item".into(),
                RustItem::Enum => "(enum_item) @item".into(),
                RustItem::Function => "(function_item) @item".into(),
            },
        }
    }
}
