extern crate colored;

use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::thread::Scope;
use std::{env, fs};

use crate::instructions::{all_instruction_examples, GoodInstructions, InitialInstruction};
use anyhow::{anyhow, bail, Result};
use clap::{Parser as ClapParser, Subcommand};
use colored::Colorize;
use dotenv::dotenv;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use schemars::schema_for;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

mod instructions;
mod lang;
mod prompts;
mod search;

use crate::lang::{ProgItem, ProgLanguage, PythonProgItem};
use crate::prompts::{
    chatgpt_wrong_answer, get_system_prompt, wrap_user_message, CodeAction, CommonAction,
};
use crate::search::{apply_changes, extract_all_items_from_files, get_filenames, ItemChange};
use lang::LanguageItem;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env_variables();

    print_introduction();

    let system_prompt = get_system_prompt()?;
    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(system_prompt.clone()),
        name: None,
        function_call: None,
    }];

    loop {
        let user_message_content = get_user_input("User")?;
        messages.push(create_chat_message(
            ChatCompletionMessageRole::User,
            Some(user_message_content.clone()),
            None,
        ));

        if !process_user_message(&user_message_content, &mut messages, &system_prompt).await? {
            break;
        }
    }

    Ok(())
}

fn load_env_variables() {
    dotenv().expect("Failed to read .env file");
    set_key(env::var("OPENAI_KEY").expect("OPENAI_KEY not set"));
}

fn print_introduction() {
    println!(
        "{}",
        "Welcome to MechaTyper! Here you can interactively work with the program.\nType in your task, and get assistance!".bright_blue()
    );
}
async fn process_user_message(
    user_message_content: &str,
    messages: &mut Vec<ChatCompletionMessage>,
    system_prompt: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut tries = 0;

    while tries == 0 {
        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages.clone())
            .temperature(0.2)
            .create()
            .await?;

        if let Some(returned_message) = chat_completion.choices.first() {
            let maybe_json = returned_message.message.content.as_ref().unwrap().trim();
            println!("Raw answer:\n{}", maybe_json);
            let instructions: Result<InitialInstruction> =
                serde_json::from_str(maybe_json).map_err(|e| anyhow!(e));

            match instructions {
                Ok(InitialInstruction::GoodInstructions(good_instructions)) => {
                    println!("Mechatyper: {}", good_instructions.answer.green());
                    make_change(good_instructions).await?;
                    break;
                }
                Ok(InitialInstruction::UserError(user_error)) => {
                    println!("Mechatyper: {}", user_error.answer.red());
                    break;
                }
                Ok(InitialInstruction::ClarificationNeeded(mut clarification)) => {
                    // Inner loop for clarification
                    loop {
                        println!("Mechatyper: {}", clarification.answer.yellow());

                        let clarification_content = get_user_input("User")?;

                        messages.push(create_chat_message(
                            ChatCompletionMessageRole::User,
                            Some(clarification_content),
                            None,
                        ));

                        let chat_completion =
                            ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages.clone())
                                .temperature(0.2)
                                .create()
                                .await?;

                        if let Some(returned_message) = chat_completion.choices.first() {
                            let maybe_json =
                                returned_message.message.content.as_ref().unwrap().trim();
                            match serde_json::from_str::<InitialInstruction>(maybe_json) {
                                Ok(InitialInstruction::ClarificationNeeded(new_clarification)) => {
                                    clarification = new_clarification;
                                }
                                _ => break, // Break the inner loop if we have any other type of instruction.
                            }
                        }
                    }
                }
                Ok(InitialInstruction::Quit) => {
                    return Ok(false);
                }
                Ok(InitialInstruction::TooManyTries) => {
                    println!(
                        "{}",
                        "Mechatyper: Too many tries. Try to rephrase your query.".red()
                    );
                    break;
                }
                Err(err) => {
                    // Tell chat model that it sent a wrong answer
                    let error_message = chatgpt_wrong_answer(
                        maybe_json,
                        &user_message_content,
                        err.to_string().as_str(),
                    )?;
                    println!("Error message:\n{}", error_message);
                    messages.push(create_chat_message(
                        ChatCompletionMessageRole::User,
                        Some(error_message),
                        None,
                    ));
                    tries += 1;
                }
            }
        }
    }

    messages.clear();
    messages.push(create_chat_message(
        ChatCompletionMessageRole::System,
        Some(system_prompt.to_string()),
        None,
    ));

    Ok(true)
}

fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}: ", prompt);
    stdout().flush()?;
    let mut user_input = String::new();
    stdin().read_line(&mut user_input)?;
    Ok(user_input)
}

fn create_chat_message(
    role: ChatCompletionMessageRole,
    content: Option<String>,
    function_call: Option<String>,
) -> ChatCompletionMessage {
    ChatCompletionMessage {
        role,
        content,
        name: None,
        function_call: None,
    }
}

async fn make_change(good_instructions: GoodInstructions) -> Result<()> {
    println!("Instructions received: {:#?}", good_instructions);
    println!(
        "Scope: {:?}, Action: {:?}, Path: {:?}",
        good_instructions.item, good_instructions.code_action, good_instructions.folder
    );

    let folder: PathBuf = good_instructions
        .folder
        .clone()
        .unwrap_or(".".to_string())
        .into();

    if find_git_directory(folder.clone()).is_none() {
        bail!("The target directory or its parents should be inside a git repository (should contain a .git folder).");
    }

    let language: ProgLanguage = good_instructions.item.clone().into();

    let files = get_filenames(
        &folder,
        &language.file_extensions(),
        &language.get_excluded_directories(),
    )?;
    let functions =
        extract_all_items_from_files(files, language.clone(), good_instructions.item.clone())?;

    let mut changes = vec![];
    for function in functions {
        println!("Changing function in file: {:?}", function.filename);
        // println!("{:#?}", function);
        // println!("FUNCTION BEFORE:\n\n{}", function.definition);
        let prompt_text = good_instructions
            .code_action
            .to_chat_gpt_prompt()
            .replace("<CODE>", &function.definition);
        let messages = vec![ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(
                prompt_text + ". Answer with code only. Keep the original indentation. Code:\n",
            ),
            name: None,
            function_call: None,
        }];

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages)
            .create()
            .await?;
        let reply = chat_completion
            .choices
            .first()
            .unwrap()
            .message
            .content
            .clone()
            .unwrap();

        changes.push(ItemChange {
            before: function.clone(),
            after: reply.clone(),
        });

        // println!("FUNCTION AFTER:\n\n{}", reply);
    }

    // println!("Changes to apply: {}", changes.len());

    apply_changes(changes)?;

    Ok(())
}
