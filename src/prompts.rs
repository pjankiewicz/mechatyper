use anyhow::Result;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumVariantNames};

use crate::instructions::{all_instruction_examples, InitialInstruction};
use crate::lang::{ProgItem, ProgLanguage};

pub fn get_system_prompt() -> Result<String> {
    Ok(format!(
        r#"
Hi ChatGPT. I will paste a user prompt for a code assistant tool. The tool works by iterating through some folder,
find the items to be changed and applies the changes.

Your answer should be a JSON using one of those variants

{}

Requirements:
- answer only with a proper JSON that can be parsed into one of those variants
- please don't guess the programming language if it is not mentioned, ask for clarification
  using ClarificationNeeded variant
- users cannot select spefific classes
- don't guess the folder name, leave empty if it is not mentioned

SUPPORTED_ITEMS = {{"Rust": ["Struct", "Enum", "Function"], "Python": ["Function", "Class"]}}

if the user uses a different combination mention the ones that can be used and tell that
we are working on more."#,
        all_instruction_examples()?
    ))
}

pub fn wrap_user_message(user_message: &str) -> Result<String> {
    let prompt = format!(
        r#"
Hi ChatGPT. I will paste a user prompt for a code assistant tool. The tool works by iterating through some folder,
find the items to be changed and applies the changes.

Your answer should be one of these JSON structures

{}

Requirements:
- answer only with a proper JSON that can be parsed into one of those variants
- please don't guess the programming language if it is not mentioned, ask for clarification
  using ClarificationNeeded variant
- users cannot select spefific classes
- Currently only some combinations of language and items are supported (others are coming soon).

SUPPORTED_ITEMS = {{"rust": ["struct", "enum", "function"], "python": ["function", "class"]}}

if the user uses a different combination mention the ones that can be used and tell that
we are working on more.

USER_MESSAGE = "{}"

Parse this message into one of: ClarificationNeeded, GoodInstructions, UserError.
"#,
        all_instruction_examples()?,
        user_message
    );
    Ok(prompt)
}

pub fn chatgpt_wrong_answer(
    chatgpt_answer: &str,
    original_question: &str,
    error_message: &str,
) -> Result<String> {
    Ok(format!(
        r#"
Hi ChatGPT. The answer you provided:

{}

Doesn't match the schemas:

{}

Original question was:

{}

Error:

{}

Requirements:
- answer only with a proper JSON that can be parsed into one of those variants
- please don't guess the programming language if it is not mentioned, ask for clarification
  using ClarificationNeeded variant
- users cannot select spefific classes
- don't guess the folder name, leave empty if it is not mentioned

SUPPORTED_ITEMS = {{"Rust": ["Struct", "Enum", "Function"], "Python": ["Function", "Class"]}}

~~~~~~~~~~

Please fix the issue and rewrite the answer so it matches the schema."#,
        chatgpt_answer,
        all_instruction_examples()?,
        original_question,
        error_message
    ))
}

pub fn user_action_to_chatgpt_prompt(prog_item: &ProgItem, user_message: &str) -> String {
    format!(
        r#"
Please {}:

<CODE>

Requirements:
Ensure the code remains functionally equivalent.
Return only the transformed code and do not include any explanations, comments, or additional text.
The output should be only code, ready to be used as a replacement for the original code.
Don't add special characters at the beginning or end.

Code:"#,
        user_message
    )
}

pub fn chatgpt_wrong_code_proposal(
    old_code: &str,
    new_code: &str,
    error_message: &str,
) -> Result<String> {
    Ok(format!(
        r#"
Hi ChatGPT. The code you provided:

{}

Cannot be parsed as programming code

{}

The error is

{}

Requirements:
- answer only with a proper code
- dont add comments"#,
        old_code, new_code, error_message
    ))
}
