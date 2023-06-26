// search
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{self, bail, Context, Result};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};

use crate::lang::{ProgItem, ProgLanguage};

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
    language_enum: ProgLanguage,
    item: ProgItem,
) -> Result<Vec<ItemDef>> {
    let extensions = language_enum.file_extensions();
    let excluded = language_enum.get_excluded_directories();
    let files = get_filenames(directory_path, &extensions, &excluded)?;
    extract_all_items_from_files(files, item)
}

pub fn extract_sexpr_from_string(
    source_code: &str,
    filename: &PathBuf,
    item: &ProgItem,
) -> Result<Vec<ItemDef>> {
    let (language, tree) = parse_code(source_code, item)?;
    let mut items = Vec::new();

    let query = Query::new(language, item.to_sexpr().as_str())?;
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

            let start_byte = node.node.start_byte();
            // Find the start of the line in the source code
            let line_start_byte = source_code[..start_byte]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let byte_range = line_start_byte..node.node.end_byte();
            let definition = source_code[byte_range.clone()].to_string();

            let start_pos = node.node.start_position().row;
            let end_pos = node.node.end_position().row;
            items.push(ItemDef {
                definition,
                start_pos,
                end_pos,
                start_byte: byte_range.start,
                end_byte: byte_range.end,
                filename: filename.clone(),
            });
        }
    }

    Ok(items)
}

pub fn parse_code(source_code: &str, item: &ProgItem) -> Result<(Language, Tree)> {
    let mut parser = Parser::new();
    let language_enum: ProgLanguage = (*item).clone().into();
    let language = language_enum.tree_sitter_language();
    parser.set_language(language).unwrap();
    let tree = parser
        .parse(source_code, None)
        .context("Cannot parse code")?;
    Ok((language, tree))
}

pub fn extract_all_items_from_files(files: Vec<PathBuf>, item: ProgItem) -> Result<Vec<ItemDef>> {
    let mut all_functions = Vec::new();
    for file_path in files {
        let mut file = File::open(&file_path)?;
        let mut source_code = String::new();
        file.read_to_string(&mut source_code)?;

        all_functions.extend(extract_sexpr_from_string(&source_code, &file_path, &item)?);
    }
    Ok(all_functions)
}

fn apply_indentation(old_code: &str, new_code: &str) -> String {
    let old_code_lines: Vec<&str> = old_code.lines().collect();
    let new_code_lines: Vec<&str> = new_code.lines().collect();

    // Detect the indentation in old code
    let old_indentation = old_code_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>()
        })
        .next()
        .unwrap_or_default();

    // Detect the number of leading whitespace characters in new code
    let new_indentation_count = new_code_lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Apply indentation to new code
    let indented_code = new_code_lines
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                old_indentation.clone() + &line[new_indentation_count..]
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    // Ensure the output ends with a newline character
    if indented_code.ends_with('\n') {
        indented_code
    } else {
        indented_code + "\n"
    }
}

pub fn apply_changes(changes: Vec<ItemChange>) -> Result<()> {
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
                // Apply the same indentation to the new code
                let indented_new_code = apply_indentation(&change.before.definition, &change.after);
                // Concatenate the new lines and replace the corresponding lines in the original content
                let replacement_lines: Vec<String> = indented_new_code
                    .lines()
                    .map(|line| line.to_string())
                    .collect();
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

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    use tempfile::tempdir;

    use crate::lang::PythonProgItem;

    use super::*;

    #[test]
    fn test_apply_changes() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.rs");

        let initial_content = "fn example() {\n    println!(\"Hello, world!\");\n}\n";

        // Write initial content to file
        let mut file = File::create(&file_path).unwrap();
        write!(file, "{}", initial_content).unwrap();

        // Prepare changes
        let changes = vec![ItemChange {
            before: ItemDef {
                definition: "fn example() {\n    println!(\"Hello, world!\");\n}\n".to_string(),
                start_pos: 0,
                end_pos: 2,
                start_byte: 0,
                end_byte: initial_content.len(),
                filename: file_path.clone(),
            },
            after: "fn modified_example() {\n    println!(\"Hello, ChatGPT!\");\n}\n".to_string(),
        }];

        // Apply changes
        let apply_result = apply_changes(changes);

        // Assert that there were no errors
        assert!(apply_result.is_ok());

        // Check if the changes were applied correctly
        let modified_content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(
            modified_content,
            "fn modified_example() {\n    println!(\"Hello, ChatGPT!\");\n}\n"
        );
    }

    #[test]
    fn test_apply_indentation_with_spaces() {
        let old_code = "    def old_function():\n        print('This is the old function')\n";
        let new_code = "def new_function():\n    print('This is the new function')\n";

        let expected_indented_new_code =
            "    def new_function():\n        print('This is the new function')\n";
        let indented_new_code = apply_indentation(old_code, new_code);

        assert_eq!(indented_new_code, expected_indented_new_code);
    }

    #[test]
    fn test_apply_indentation_with_tabs() {
        let old_code = "\tdef old_function():\n\t\tprint('This is the old function')\n";
        let new_code = "def new_function():\n\tprint('This is the new function')\n";

        let expected_indented_new_code =
            "\tdef new_function():\n\t\tprint('This is the new function')\n";
        let indented_new_code = apply_indentation(old_code, new_code);

        assert_eq!(indented_new_code, expected_indented_new_code);
    }

    #[test]
    fn test_extract_sexpr_from_string() {
        let code = r#"
import math

class CircleCalculator:
    def __init__(self, radius):
        self.radius = radius
        self.pi = 3.1415

    def calc_area(self):
        r = self.radius
        pi = self.pi
        a = pi * r * r
        return a

    def calc_circumference(self):
        r = self.radius
        pi = 3.1415
        c = 2 * pi * r
        return c"#;

        let functions = extract_sexpr_from_string(
            code,
            &PathBuf::new(),
            &ProgItem::Python(PythonProgItem::Function),
        );
        for function in functions.unwrap() {
            println!("{}", function.definition);
        }
    }
}
