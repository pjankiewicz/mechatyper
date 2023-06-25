// search
use std::collections::HashMap;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread::Scope;

use anyhow::{self, bail, Context, Result};
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use colored::*;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};
use dotenv::dotenv;
use clap::{Parser as ClapParser, Subcommand};

mod lang;
mod prompts;
mod search;

use crate::search::{apply_changes, extract_all_items_from_files, get_filenames, ItemChange};
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

#[derive(ClapParser)]
#[clap(version = "1.0", author = "Your Name", about = "Mechatyper: Code Assistant for Mass Code Edits")]
struct MechatyperCli {
    #[arg(value_enum)]
    language: LanguageEnum,

    #[arg(value_enum)]
    scope: LanguageItem,

    #[arg(value_enum)]
    action: CodeAction,

    #[clap(short, long)]
    path: PathBuf,
}


#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    set_key(env::var("OPENAI_KEY")?);
    let cli: MechatyperCli = MechatyperCli::parse();

    println!(
        "Language: {:?}, Scope: {:?}, Action: {:?}, Path: {:?}",
        cli.language, cli.scope, cli.action, cli.path
    );

    if find_git_directory(cli.path.clone()).is_none() {
        bail!("The target directory or its parents should be inside a git repository (should contain a .git folder).");
    }

    let files = get_filenames(
        &cli.path.clone(),
        &cli.language.file_extensions(),
        &cli.language.get_excluded_directories(),
    )?;
    let functions = extract_all_items_from_files(files, cli.language.clone(), cli.scope.clone())?;

    let mut changes = vec![];
    for function in functions {
        println!("{:#?}", function);
        println!("FUNCTION BEFORE:\n\n{}", function.definition);
        let prompt_text = cli.action.to_chat_gpt_prompt().replace("<CODE>", &function.definition);
        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(prompt_text + ". Answer with code only. Keep the original indentation. Code:\n"),
                name: None,
                function_call: None,
            }
        ];

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages)
            .create()
            .await?;
        let reply = chat_completion.choices.first().unwrap().message.content.clone().unwrap();

        changes.push(ItemChange{
            before: function.clone(),
            after: reply.clone(),
        });

        println!("FUNCTION AFTER:\n\n{}", reply);
    }

    println!("Changes to apply: {}", changes.len());

    apply_changes(changes)?;

    Ok(())
}

#[tokio::main]
async fn main2() -> Result<()> {
    // Load environment variables from .env file
    dotenv()?;
    set_key(env::var("OPENAI_KEY")?);

    let project_path = "examples";
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

    let mut changes = vec![];
    for function in functions {
        println!("{:#?}", function);
        println!("FUNCTION BEFORE:\n\n{}", function.definition);
        let prompt_text = action.to_chat_gpt_prompt().replace("<CODE>", &function.definition);
        // unimplemented!();
        // Send the prompt text to ChatGPT and receive the reply
        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(prompt_text + ". Answer with code only. Keep the original indentation. Code:\n"),
                name: None,
                function_call: None,
            }
        ];

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages)
            .create()
            .await?;
        let reply = chat_completion.choices.first().unwrap().message.content.clone().unwrap();

        changes.push(ItemChange{
            before: function.clone(),
            after: reply.clone(),
        });

        println!("FUNCTION AFTER:\n\n{}", reply);
    }

    println!("Changes to apply: {}", changes.len());

    apply_changes(changes)?;

    Ok(())
}