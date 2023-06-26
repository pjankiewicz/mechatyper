use anyhow::{anyhow, Result};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::lang::{ProgItem, ProgLanguage, PythonProgItem};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GoodInstructions {
    pub item: ProgItem,
    /// helpful message of how you understand the prompt
    pub answer: String,
    /// the original prompt
    pub user_message: String,
    /// if the user mentions any folder
    /// leave empty if a folder is not mentioned
    pub folder: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct UserError {
    pub user_message: String,
    /// explain why you cannot understand the prompt, enumerate the programming
    /// languages that are supported and explain
    pub answer: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClarificationNeeded {
    pub item: Option<ProgItem>,
    pub folder: Option<String>,
    pub answer: String,
    pub user_message: String,
    pub clarification_needed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum InitialInstruction {
    GoodInstructions(GoodInstructions),
    UserError(UserError),
    ClarificationNeeded(ClarificationNeeded),
    Quit,
    TooManyTries,
}

pub fn good_instruction_example() -> Result<String> {
    let data = GoodInstructions {
        item: ProgItem::Python(PythonProgItem::Function),
        answer: "I understand that you want to document your Python functions in the folder src/"
            .to_string(),
        user_message: "Create a Python function".to_string(),
        folder: Some("src".to_string()),
    };
    serde_json::to_string_pretty(&data).map_err(|e| anyhow!(e))
}

pub fn good_instruction_example_2() -> Result<String> {
    let data = GoodInstructions {
        item: ProgItem::Python(PythonProgItem::Function),
        answer: "I understand that you want to document your Python functions in the folder using Pirate talk src/"
            .to_string(),
        user_message: "Create a Python function".to_string(),
        folder: Some("src".to_string()),
    };
    serde_json::to_string_pretty(&data).map_err(|e| anyhow!(e))
}

pub fn user_error_instruction_example() -> Result<String> {
    let data = UserError {
        user_message: "Edit functions".to_string(),
        answer: "Please provide what programming language you want to use and how you want to change the functions".to_string(),
    };
    serde_json::to_string_pretty(&data).map_err(|e| anyhow!(e))
}

pub fn clarification_needed_instruction_example() -> Result<String> {
    let data = ClarificationNeeded {
        item: None,
        folder: None,
        answer: "Can you please clarify what programming language you are referring to?"
            .to_string(),
        user_message: "Create a function".to_string(),
        clarification_needed: false,
    };
    serde_json::to_string_pretty(&data).map_err(|e| anyhow!(e))
}

pub fn all_instruction_examples() -> Result<String> {
    let good_instruction = schema_for!(GoodInstructions);
    let good_instruction_json_schema = serde_json::to_string_pretty(&good_instruction).unwrap();
    let clarification_instruction = schema_for!(ClarificationNeeded);
    let clarification_json_schema =
        serde_json::to_string_pretty(&clarification_instruction).unwrap();
    Ok(format!(
        r#"
Examples of proper answers

Good instructions
=================

JSON Schema:
{}

Example:
{}

{}

Clarification needed
====================
language, item, folder, code action can be null or filled but not all of them.
That's why we need clarification about what the user wants. The user
must specify all of the information

JSON Schema:
{}

Example:
{}"#,
        good_instruction_json_schema,
        good_instruction_example()?,
        good_instruction_example_2()?,
        clarification_json_schema,
        clarification_needed_instruction_example()?
    ))
}
