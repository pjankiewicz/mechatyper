// search
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::thread::Scope;
use std::{env, fs};

use crate::instructions::{all_instruction_examples, GoodInstructions, InitialInstruction};
use anyhow::{anyhow, bail, Result};
use clap::{Parser as ClapParser, Subcommand};
use colored::*;
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
    // Load environment variables
    dotenv().expect("Failed to read .env file");
    set_key(env::var("OPENAI_KEY").expect("OPENAI_KEY not set"));

    // Initialize chat
    let system_prompt = get_system_prompt()?;
    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(system_prompt.clone()),
        name: None,
        function_call: None,
    }];

    loop {
        // Read user input
        print!("User: ");
        stdout().flush()?;
        let mut user_message_content = String::new();
        stdin().read_line(&mut user_message_content)?;

        // Add user message to chat history
        messages.push(ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(user_message_content.clone()),
            name: None,
            function_call: None,
        });

        let mut tries = 0;

        // Outer loop for tries to get valid instructions
        while tries < 3 {
            let chat_completion =
                ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages.clone())
                    .create()
                    .await?;

            if let Some(returned_message) = chat_completion.choices.first() {
                let maybe_json = returned_message.message.content.as_ref().unwrap().trim();
                let instructions: Result<InitialInstruction> =
                    serde_json::from_str(maybe_json).map_err(|e| anyhow!(e));
                println!("Instructions: {:#?}", instructions);

                match instructions {
                    Ok(InitialInstruction::GoodInstructions(good_instructions)) => {
                        println!("Mechatyper: {}", good_instructions.answer);
                        make_change(good_instructions).await?;
                        break;
                    }
                    Ok(InitialInstruction::UserError(user_error)) => {
                        println!("Mechatyper: {}", user_error.answer);
                        break;
                    }
                    Ok(InitialInstruction::ClarificationNeeded(mut clarification)) => {
                        // Inner loop for clarification
                        loop {
                            println!("{}", clarification.answer);

                            print!("User: ");
                            stdout().flush()?;
                            let mut clarification_content = String::new();
                            stdin().read_line(&mut clarification_content)?;

                            messages.push(ChatCompletionMessage {
                                role: ChatCompletionMessageRole::User,
                                content: Some(clarification_content),
                                name: None,
                                function_call: None,
                            });

                            let chat_completion =
                                ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages.clone())
                                    .create()
                                    .await?;

                            if let Some(returned_message) = chat_completion.choices.first() {
                                let maybe_json =
                                    returned_message.message.content.as_ref().unwrap().trim();
                                match serde_json::from_str::<InitialInstruction>(maybe_json) {
                                    Ok(InitialInstruction::ClarificationNeeded(
                                        new_clarification,
                                    )) => {
                                        clarification = new_clarification;
                                    }
                                    _ => break, // Break the inner loop if we have any other type of instruction.
                                }
                            }
                        }
                    }
                    Ok(InitialInstruction::Quit) => {
                        return Ok(());
                    }
                    Ok(InitialInstruction::TooManyTries) => {
                        println!("Mechatyper: Too many tries. Try to rephrase your query.");
                        break;
                    }
                    Err(err) => {
                        // Tell chat model that it sent a wrong answer
                        let error_message = chatgpt_wrong_answer(
                            maybe_json,
                            &user_message_content,
                            err.to_string().as_str(),
                        )?;
                        messages.push(ChatCompletionMessage {
                            role: ChatCompletionMessageRole::User,
                            content: Some(error_message),
                            name: None,
                            function_call: None,
                        });
                        tries += 1;
                    }
                }
            }
        }

        // Reset message history for the next iteration
        messages = vec![ChatCompletionMessage {
            role: ChatCompletionMessageRole::System,
            content: Some(system_prompt.clone()),
            name: None,
            function_call: None,
        }];
    }
}

async fn make_change(good_instructions: GoodInstructions) -> Result<()> {
    println!(
        "Language: {:?}, Scope: {:?}, Action: {:?}, Path: {:?}",
        good_instructions.language,
        good_instructions.item,
        good_instructions.code_action,
        good_instructions.folder
    );

    let folder: PathBuf = good_instructions
        .folder
        .clone()
        .unwrap_or(".".to_string())
        .into();

    if find_git_directory(folder.clone()).is_none() {
        bail!("The target directory or its parents should be inside a git repository (should contain a .git folder).");
    }

    let files = get_filenames(
        &folder,
        &good_instructions.language.file_extensions(),
        &good_instructions.language.get_excluded_directories(),
    )?;
    let functions = extract_all_items_from_files(
        files,
        good_instructions.language.clone(),
        good_instructions.item.clone(),
    )?;

    let mut changes = vec![];
    for function in functions {
        println!("{:#?}", function);
        println!("FUNCTION BEFORE:\n\n{}", function.definition);
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

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages)
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

        println!("FUNCTION AFTER:\n\n{}", reply);
    }

    println!("Changes to apply: {}", changes.len());

    apply_changes(changes)?;

    Ok(())
}
