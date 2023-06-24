use std::path::PathBuf;
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

pub enum LanguageEnum {
    Python,
    Rust,
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
