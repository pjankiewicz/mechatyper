extern crate colored;

use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::thread::Scope;
use std::{env, fs};

use anyhow::{anyhow, bail, Result};
use clap::{Parser as ClapParser, Subcommand};
use colored::Colorize;
use dotenv::dotenv;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use schemars::schema_for;
use tree_sitter::{Language, Node, Parser, Query, QueryCursor};

use crate::instructions::{all_instruction_examples, GoodInstructions, InitialInstruction};
use crate::lang::{ProgItem, ProgLanguage, PythonProgItem};
use crate::prompts::{
    chatgpt_wrong_answer, chatgpt_wrong_code_proposal, get_system_prompt,
    user_action_to_chatgpt_prompt, wrap_user_message,
};
use crate::search::{
    apply_changes, extract_all_items_from_files, get_filenames, parse_code, ItemChange,
};

mod code_cleaning;
mod instructions;
mod lang;
mod llm;
mod prompts;
mod search;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    utils::load_env_variables();

    utils::print_introduction();

    let system_prompt = get_system_prompt()?;
    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some(system_prompt.clone()),
        name: None,
        function_call: None,
    }];

    loop {
        let user_message_content = utils::get_user_input("User")?;
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
            // println!("Raw answer:\n{}", maybe_json);
            let instructions: Result<InitialInstruction> =
                serde_json::from_str(maybe_json).map_err(|e| anyhow!(e));

            match instructions {
                Ok(InitialInstruction::GoodInstructions(good_instructions)) => {
                    mechatype_answer(&good_instructions.answer);
                    make_change(good_instructions).await?;
                    break;
                }
                Ok(InitialInstruction::UserError(user_error)) => {
                    mechatype_answer(&user_error.answer.red());
                    break;
                }
                Ok(InitialInstruction::ClarificationNeeded(mut clarification)) => {
                    // Inner loop for clarification
                    loop {
                        mechatype_answer(&clarification.answer.red());

                        let clarification_content = utils::get_user_input("User")?;

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
                    mechatype_answer("Too many tries. Try to rephrase your query.");
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

fn mechatype_answer(text: &str) {
    println!("{}: {}", "MechaTyper".green().bold(), text.green());
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
        "Scope: {:?}, Path: {:?}",
        good_instructions.item, good_instructions.folder
    );

    let folder: PathBuf = good_instructions
        .folder
        .clone()
        .unwrap_or(".".to_string())
        .into();

    if utils::find_git_directory(folder.clone()).is_none() {
        bail!("The target directory or its parents should be inside a git repository (should contain a .git folder).");
    }

    let language: ProgLanguage = good_instructions.item.clone().into();

    let files = get_filenames(
        &folder,
        &language.file_extensions(),
        &language.get_excluded_directories(),
    )?;
    let functions = extract_all_items_from_files(files, good_instructions.item.clone())?;

    let mut changes = vec![];
    for function in functions {
        println!("Changing item in file: {:?}", function.filename);
        let mut new_code = function.definition.clone();
        let mut retry_count = 0;
        loop {
            let prompt_text = if retry_count == 0 {
                // First iteration: prompt to apply the suggested action
                user_action_to_chatgpt_prompt(
                    &good_instructions.item,
                    &good_instructions.user_message,
                )
                .replace("<CODE>", &new_code)
            } else {
                // Subsequent iterations: prompt indicating that the previous change was incorrect
                match chatgpt_wrong_code_proposal(
                    &function.definition,
                    &new_code,
                    "Error message from parser",
                ) {
                    Ok(wrong_code_prompt) => wrong_code_prompt,
                    Err(_) => {
                        println!("Error generating prompt for wrong code proposal. Skipping...");
                        break;
                    }
                }
            };

            let messages = vec![ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(prompt_text),
                name: None,
                function_call: None,
            }];

            let chat_completion = ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages)
                .create()
                .await?;
            new_code = chat_completion
                .choices
                .first()
                .unwrap()
                .message
                .content
                .clone()
                .unwrap();

            // Check if the reply from ChatGPT can be parsed
            if parse_code(&new_code, &good_instructions.item).is_ok() {
                // If the parsing is successful, save the change
                changes.push(ItemChange {
                    before: function.clone(),
                    after: new_code.clone(),
                });
                break;
            } else {
                // Retry up to 3 times before skipping
                retry_count += 1;
                if retry_count >= 3 {
                    println!(
                        "Failed to parse the code for function: {:?} after 3 attempts. Skipping...",
                        function.filename
                    );
                    break;
                }
            }
        }
    }

    apply_changes(changes)?;

    Ok(())
}
