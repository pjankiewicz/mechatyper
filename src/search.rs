// search
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{self, bail, Context, Result};
use tree_sitter::{Node, Parser, Query, QueryCursor};

use crate::lang::{LanguageEnum, LanguageItem};

#[derive(Clone, Debug)]
pub struct ItemDef {
    pub definition: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub start_byte: usize,
    pub end_byte: usize,
    pub filename: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ItemChange {
    pub before: ItemDef,
    pub after: String, // assuming you want to replace with a new string
}
pub fn get_filenames(
    path: &Path,
    extensions: &[&str],
    excluded_dirs: &[&str],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dir_name = entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            // Exclude directories starting with dot or in excluded_dirs list
            if entry_path.is_dir()
                && (dir_name.starts_with('.') || excluded_dirs.iter().any(|excl| excl == &dir_name))
            {
                continue;
            }

            if entry_path.is_dir() {
                files.extend(get_filenames(&entry_path, extensions, excluded_dirs)?);
            } else if let Some(extension) = entry_path.extension() {
                if let Some(extension_str) = extension.to_str() {
                    if extensions.iter().any(|ext| extension_str == *ext) {
                        files.push(entry_path);
                    }
                }
            }
        }
    }
    Ok(files)
}

pub fn extract_all_items_from_directory(
    directory_path: &Path,
    language_enum: LanguageEnum,
    item: LanguageItem,
) -> Result<Vec<ItemDef>> {
    let extensions = language_enum.file_extensions();
    let excluded = language_enum.get_excluded_directories();
    let files = get_filenames(directory_path, &extensions, &excluded)?;
    extract_all_items_from_files(files, language_enum, item)
}

pub fn extract_sexpr_from_string(
    source_code: &str,
    item: &LanguageItem,
    language_enum: &LanguageEnum,
) -> Result<Vec<ItemDef>> {
    let mut parser = Parser::new();
    let language = language_enum.tree_sitter_language();
    parser.set_language(language).unwrap();

    let tree = parser.parse(source_code, None).unwrap();
    let mut items = Vec::new();

    let query = Query::new(language, item.to_sexpr().as_str()).unwrap();
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());
    let capture_names = query.capture_names();

    for m in matches {
        for name in capture_names {
            let index = query.capture_index_for_name(name);
            let index = match index {
                Some(i) => i,
                None => bail!(
                    "Error while querying source code. Capture name: {} has no index associated.",
                    name
                ),
            };

            let node = m.captures.iter().find(|c| c.index == index);
            let node = match node {
                Some(v) => v,
                None => continue,
            };

            let value = node
                .node
                .utf8_text(source_code.as_bytes())
                .with_context(|| {
                    format!(
                        "Cannot match query result indices with source code for capture name: {}.",
                        name
                    )
                })?;

            let start_pos = node.node.start_position().row;
            let end_pos = node.node.end_position().row;
            let byte_range = node.node.byte_range();
            let definition = source_code[byte_range.start..byte_range.end].to_string();
            items.push(ItemDef {
                definition,
                start_pos,
                end_pos,
                start_byte: byte_range.start,
                end_byte: byte_range.end,
                filename: PathBuf::new(), // Modify this if needed
            });
        }
    }

    Ok(items)
}

pub fn extract_all_items_from_files(
    files: Vec<PathBuf>,
    language_enum: LanguageEnum,
    item: LanguageItem,
) -> Result<Vec<ItemDef>> {
    let mut all_functions = Vec::new();
    for file_path in files {
        let mut file = File::open(&file_path)?;
        let mut source_code = String::new();
        file.read_to_string(&mut source_code)?;

        all_functions.extend(extract_sexpr_from_string(
            &source_code,
            &item,
            &language_enum,
        )?);
    }
    Ok(all_functions)
}

pub fn apply_changes(changes: Vec<ItemChange>) -> anyhow::Result<()> {
    // Group changes by file
    let mut changes_by_file: HashMap<PathBuf, Vec<ItemChange>> = HashMap::new();
    for change in changes {
        changes_by_file
            .entry(change.before.filename.clone())
            .or_default()
            .push(change);
    }

    // Apply changes to each file
    for (file_path, changes) in changes_by_file.iter() {
        // Read the file contents line by line
        let contents = fs::read_to_string(file_path)?;
        let mut lines: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

        // Sort changes in descending order by start_pos, so that changes later in the file do not affect the position of earlier changes
        let mut changes = changes.clone();
        changes.sort_by(|a, b| b.before.start_pos.cmp(&a.before.start_pos));

        // Apply changes
        for change in changes {
            let start_line = change.before.start_pos;
            let end_line = change.before.end_pos;

            if start_line <= end_line && end_line < lines.len() {
                // Concatenate the new lines and replace the corresponding lines in the original content
                let replacement_lines: Vec<String> =
                    change.after.lines().map(|line| line.to_string()).collect();
                lines.splice(start_line..=end_line, replacement_lines.iter().cloned());
            }
        }

        // Write the modified contents back to the file
        let mut file = fs::File::create(file_path)?;
        for line in lines {
            writeln!(file, "{}", line)?;
        }
    }

    Ok(())
}
