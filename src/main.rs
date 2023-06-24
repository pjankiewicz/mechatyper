// search
use std::collections::HashMap;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{self, bail, Context, Result};
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use tree_sitter::{Node, Parser, Query, QueryCursor};
use dotenv::dotenv;

mod lang;
mod prompts;
mod search;

use crate::search::{extract_all_items_from_files, get_filenames};
use lang::{LanguageEnum, LanguageItem};
use crate::prompts::{CodeAction, CommonAction};

fn find_git_directory(mut path: PathBuf) -> Option<PathBuf> {
    loop {
        if path.join(".git").is_dir() {
            return Some(path);
        }

        if !path.pop() {
            // We have reached the root directory without finding .git
            return None;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv()?;
    set_key(env::var("OPENAI_KEY")?);

    let project_path = ".";
    let path_buf = Path::new(project_path).to_path_buf();
    let path = Path::new(project_path);

    // Fail-safe: Check if .git directory is present in current or parent directories
    if find_git_directory(path_buf).is_none() {
        bail!("The target directory or its parents should be inside a git repository (should contain a .git folder).");
    }

    // Specifying Python language for example
    let language_enum = LanguageEnum::Python;
    let files = get_filenames(
        path,
        &language_enum.file_extensions(),
        &language_enum.get_excluded_directories(),
    )?;

    let item = LanguageItem::Python(lang::PythonItem::Function);
    let action = CodeAction::CommonAction(CommonAction::Refactor);
    let functions = extract_all_items_from_files(files, language_enum, item)?;

    for function in functions {
        println!("FUNCTION BEFORE:\n\n{}", function.definition);
        let prompt_text = action.to_chat_gpt_prompt().replace("<CODE>", &function.definition);

        // Send the prompt text to ChatGPT and receive the reply
        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(prompt_text),
                name: None,
                function_call: None,
            }
        ];

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages)
            .create()
            .await?;
        let reply = chat_completion.choices.first().unwrap().message.content.clone().unwrap();

        println!("FUNCTION AFTER:\n\n{}", reply);
    }

    Ok(())
}