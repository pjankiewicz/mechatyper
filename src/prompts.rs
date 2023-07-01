use anyhow::Result;
use schemars::{schema_for, JsonSchema};
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

pub fn quickcheck_prompt(task: &str) -> String {
    format!(
        r#"
Write a Python function that detects if a code snippet, representing a single function or class, is a good candidate for applying a specific change. The function should take the code snippet as input and return a boolean value indicating whether the code snippet meets the criteria for the change. Ensure that the function only checks the contents of the code snippet and does not actually modify it. The function should adhere to the following signature:

```python
def detect(code: str) -> bool:
    # Your code here
    pass
```

Please specify the specific change or condition you want the function to check for, and I will provide you with the corresponding Python code. Please note that the response should be provided as a code snippet only, without any additional comments or explanations.

Example 1:
Task: "remove unwrap from functions, convert them to use Result from anyhow crate"

should return a Python function:
```python
def detect(code: str) -> bool:
    return "unwrap" in code
```

Example 2:
Task: "split long functions (above 50 lines of code) to smaller functions"

should return a Python function:
```python
def detect(code: str) -> bool:
    return len(code.splitlines()) > 50
```

Example 3:
Task: "Refactor all Python functions"

no condition can be applied so the function should always return True:
```python
def detect(code: str) -> bool:
    return True
```

Requirements:
- return only Python code without any additional comments or explanations
- don't include any special characters before or after the code
- the function can be only applied to a specific code fragment like a function or a class
- you can use only standard library
- the function can return false positives so you can use many different conditions connected with "or"

~~~~~~~~~~~~

TASK: {}

```python
"#,
        task
    )
}
