use crate::instructions::{all_instruction_examples, InitialInstruction};
use crate::lang::{LanguageItem, ProgItem, ProgLanguage};
use anyhow::Result;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, EnumVariantNames};

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

#[derive(Clone, Debug, EnumString, EnumVariantNames)]
pub enum CodeAction {
    CustomAction(String),
    CommonAction(CommonAction),
    PythonAction(PythonAction),
    RustAction(RustAction),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, EnumString, EnumVariantNames)]
pub enum SimpleAction {
    Other(String),
    Refactor,
    Document,
    AddDocStrings,
    SplitLongFunctions,
    RemoveDeadCode,
    AddErrorHandling,
}

#[derive(Clone, Debug, EnumString, EnumVariantNames)]
pub enum CommonAction {
    Other(String),
    Refactor,
    Document,
    AddDocStrings,
    SplitLongFunctions,
    RemoveDeadCode,
    AddErrorHandling,
    EncapsulateFields,
    ApplyFunctionalStyle,
    GeneralizeTypes,
    ValidateParameters,
    SimplifyConditionalStatements,
}

impl Default for CommonAction {
    fn default() -> Self {
        CommonAction::Refactor
    }
}

impl SimpleAction {
    pub fn to_chat_gpt_prompt(&self) -> String {
        match self {
            SimpleAction::Refactor =>
                "Please refactor the following code to improve readability and maintainability: <CODE>. Ensure the code remains functionally equivalent. Return only the transformed code.".to_string(),
            SimpleAction::Document =>
                "Please document the following code by adding appropriate comments: <CODE>. Explain the purpose and functionality of the code. Return only the documented code.".to_string(),
            SimpleAction::AddDocStrings =>
                "Please add docstrings to the following code: <CODE>. Provide detailed explanations for functions and classes. Return only the code with added docstrings.".to_string(),
            SimpleAction::SplitLongFunctions =>
                "Please split any long functions in the following code into smaller, more manageable functions: <CODE>. Ensure that the functionality remains the same. Return only the transformed code.".to_string(),
            SimpleAction::RemoveDeadCode =>
                "Please remove any dead or unreachable code in the following code: <CODE>. Ensure that the remaining code is functional and clean. Return only the cleaned code.".to_string(),
            SimpleAction::AddErrorHandling =>
                "Please add error handling to the following code: <CODE>. Ensure that the code handles potential errors gracefully and provides informative error messages. Return only the code with error handling.".to_string(),
            SimpleAction::Other(action) =>
                format!("Please help me to transform the code: <CODE>. The desired custom action is '{}'. Return the transformed code.", action),
        }
    }
}

#[derive(Clone, Debug, EnumString, EnumVariantNames)]
pub enum PythonAction {
    CustomFunctionAction(String),
    CustomClassAction(String),
    AddTypeAnnotations,
    ConvertPrintToLogging,
    ConvertOldFormatStrings,
    UseListComprehensions,
    ConvertToGenerator,
    ReplaceManualExceptions,
    UseUnderScoresInNumericLiterals,
    UseFormattedStringLiterals,
    ConvertToDataClass,
    ReplaceExplicitLoopsWithItertools,
    UseStaticMethods,
    RefactorNestedFunctions,
}

impl Default for PythonAction {
    fn default() -> Self {
        PythonAction::CustomClassAction("Refactor".into())
    }
}

#[derive(Clone, Debug, EnumString, EnumVariantNames)]
pub enum RustAction {
    CustomStructAction(String),
    CustomFunctionAction(String),
    CustomEnumAction(String),
    ConvertEnumToStruct,
    AddErrorHandling,
    UseSerdeForSerialization,
    ImplementDisplayTrait,
    ImplementFromTrait,
    RefactorWithPatternMatching,
    OptimizeLifetimeAnnotations,
    ReplacePanicWithResult,
    UseMacrosForCodeReuse,
    UseBorrowingEffectively,
    UtilizeIteratorMethods,
    SimplifyMatchStatements,
}

impl Default for RustAction {
    fn default() -> Self {
        RustAction::CustomFunctionAction("Refactor".into())
    }
}

impl CodeAction {
    pub fn to_chat_gpt_prompt(&self) -> String {
        match self {
            CodeAction::CustomAction(action) => format!("Please help me to customly transform the code: <CODE>. The desired custom action is '{}'. Return the transformed code.", action),
            CodeAction::CommonAction(common_action) => {
                match common_action {
                    CommonAction::Refactor =>
                        "Please refactor the following code to improve readability and maintainability: <CODE>. Ensure the code remains functionally equivalent. Return only the transformed code.".to_string(),
                    CommonAction::Document =>
                        "Please document the following code by adding appropriate comments: <CODE>. Explain the purpose and functionality of the code. Return only the documented code.".to_string(),
                    CommonAction::AddDocStrings =>
                        "Please add docstrings to the following code: <CODE>. Provide detailed explanations for functions and classes. Return only the code with added docstrings.".to_string(),
                    CommonAction::SplitLongFunctions =>
                        "Please split any long functions in the following code into smaller, more manageable functions: <CODE>. Ensure that the functionality remains the same. Return only the transformed code.".to_string(),
                    CommonAction::RemoveDeadCode =>
                        "Please remove any dead or unreachable code in the following code: <CODE>. Ensure that the remaining code is functional and clean. Return only the cleaned code.".to_string(),
                    CommonAction::AddErrorHandling =>
                        "Please add error handling to the following code: <CODE>. Ensure that the code handles potential errors gracefully and provides informative error messages. Return only the code with error handling.".to_string(),
                    CommonAction::EncapsulateFields =>
                        "Please encapsulate the fields in the following code: <CODE>. Make sure to provide proper getters and setters where necessary. Return only the encapsulated code.".to_string(),
                    CommonAction::ApplyFunctionalStyle =>
                        "Please refactor the following code: <CODE>, to use a functional programming style. Replace loops with map and reduce operations where possible. Return only the transformed code.".to_string(),
                    CommonAction::GeneralizeTypes =>
                        "Please refactor the following code: <CODE>, to use more generic types. This might involve replacing concrete types with interfaces or generics. Return only the transformed code.".to_string(),
                    CommonAction::ValidateParameters =>
                        "Please add parameter validation to the functions in the following code: <CODE>. Ensure that the functions check for valid input before proceeding. Return only the code with parameter validation.".to_string(),
                    CommonAction::SimplifyConditionalStatements =>
                        "Please simplify the conditional statements in the following code: <CODE>. Reduce complexity and improve readability. Return only the simplified code.".to_string(),
                    CommonAction::Other(other) =>
                        format!("Please apply this change '{}' to the following code: <CODE>. Return only the simplified code.", other).to_string(),
                }
            }
            CodeAction::PythonAction(python_action) => {
                match python_action {
                    PythonAction::AddTypeAnnotations =>
                        "Please add type annotations to the functions and variables in the following Python code: <CODE>. Return only the code with type annotations.".to_string(),
                    PythonAction::ConvertPrintToLogging =>
                        "Please convert any print statements in the following Python code: <CODE>, to use the logging module. This will allow for more flexible control over log output. Return only the code using the logging module.".to_string(),
                    PythonAction::ConvertOldFormatStrings =>
                        "Please convert any old-style formatted strings (e.g. %s) in the following Python code: <CODE>, to use f-strings. Return only the code with converted formatted strings.".to_string(),
                    PythonAction::UseListComprehensions =>
                        "Please refactor the following Python code: <CODE>, to use list comprehensions instead of explicit loops for creating lists. Return only the code with list comprehensions.".to_string(),
                    PythonAction::ConvertToGenerator =>
                        "Please convert any applicable functions in the following Python code: <CODE>, into generator functions using the 'yield' keyword. Return only the code with generator functions.".to_string(),
                    PythonAction::ReplaceManualExceptions =>
                        "Please replace manual exception handling in the following Python code: <CODE>, with appropriate built-in exceptions. Return only the code with built-in exceptions.".to_string(),
                    PythonAction::UseUnderScoresInNumericLiterals =>
                        "Please improve the readability of large numbers in the following Python code: <CODE>, by using underscores as thousand separators. Return only the code with underscores in numeric literals.".to_string(),
                    PythonAction::UseFormattedStringLiterals =>
                        "Please refactor the following Python code: <CODE>, to use formatted string literals (f-strings) for string formatting. Return only the code with formatted string literals.".to_string(),
                    PythonAction::ConvertToDataClass =>
                        "Please convert the classes in the following Python code: <CODE>, to data classes using the '@dataclass' decorator from the 'dataclasses' module. Return only the code with data classes.".to_string(),
                    PythonAction::ReplaceExplicitLoopsWithItertools =>
                        "Please refactor the following Python code: <CODE>, to replace explicit loops with functions from the 'itertools' module where possible. Return only the code with itertools functions.".to_string(),
                    PythonAction::UseStaticMethods =>
                        "Please refactor the following Python code: <CODE>, by converting methods that don't use instance variables to static methods. Return only the code with static methods.".to_string(),
                    PythonAction::RefactorNestedFunctions =>
                        "Please refactor the following Python code: <CODE>, by moving deeply nested functions to the top level, and passing necessary data as parameters. Return only the refactored code.".to_string(),
                    PythonAction::CustomFunctionAction(prompt) =>
                        format!("Please apply the following custom action to the given Python function: <CODE>. Custom action: {} Return only the modified code.", prompt),
                    PythonAction::CustomClassAction(prompt) =>
                        format!("Please apply the following custom action to the given Python class: <CODE>. Custom action: {} Return only the modified code.", prompt)
                }
            }
            CodeAction::RustAction(rust_action) => {
                match rust_action {
                    RustAction::ConvertEnumToStruct =>
                        "Please convert any enums in the following Rust code: <CODE>, to structs. Provide implementations for any necessary functions that were part of the enum. Return only the code with enums converted to structs.".to_string(),
                    RustAction::AddErrorHandling =>
                        "Please refactor the following Rust code: <CODE>, to include error handling using the Result type. Replace unwraps and expects with proper error handling. Return only the Rust code with error handling added.".to_string(),
                    RustAction::UseSerdeForSerialization =>
                        "Please refactor the following Rust code: <CODE>, to use the Serde library for serialization and deserialization of structs and enums. Ensure all necessary attributes and imports are included. Return only the Rust code with Serde integration.".to_string(),
                    RustAction::ImplementDisplayTrait =>
                        "For the following Rust code: <CODE>, please implement the Display trait for any structs or enums that could benefit from custom string representation. Prepend the old code to the answer and ensure that all required imports are included. Return only the Rust code with the Display trait implemented.".to_string(),
                    RustAction::ImplementFromTrait =>
                        "Please implement the From trait for appropriate type conversions in the following Rust code: <CODE>. Prepend the old code to the answer and ensure that all required imports are included. Return only the Rust code with the From trait implemented.".to_string(),
                    RustAction::RefactorWithPatternMatching =>
                        "Please refactor the following Rust code: <CODE>, to use pattern matching for more concise and readable control flow. Return only the Rust code with pattern matching.".to_string(),
                    RustAction::OptimizeLifetimeAnnotations =>
                        "Please optimize the lifetime annotations in the following Rust code: <CODE>. Remove unnecessary annotations and ensure that the code is efficient and readable. Return only the optimized Rust code.".to_string(),
                    RustAction::ReplacePanicWithResult =>
                        "Please refactor the following Rust code: <CODE>, to replace any panic! calls with returning an Err from the function. This should improve the error handling of the code. Return only the Rust code with panics replaced with Result.".to_string(),
                    RustAction::UseMacrosForCodeReuse =>
                        "Please refactor the following Rust code: <CODE>, to use macros where repetitive code patterns can be abstracted for reuse. Return only the Rust code with macros for code reuse.".to_string(),
                    RustAction::UseBorrowingEffectively =>
                        "Please refactor the following Rust code: <CODE>, to use borrowing effectively, avoiding unnecessary cloning and ownership transfer where references can be used. Return only the Rust code optimized with effective borrowing.".to_string(),
                    RustAction::UtilizeIteratorMethods =>
                        "Please refactor the following Rust code: <CODE>, to utilize iterator methods for more concise and efficient processing of collections. Return only the Rust code with iterator methods.".to_string(),
                    RustAction::SimplifyMatchStatements =>
                        "Please simplify any complex match statements in the following Rust code: <CODE>, by using patterns and combining cases where possible. Return only the simplified Rust code.".to_string(),
                    RustAction::CustomStructAction(prompt) =>
                        format!("Please apply the following custom action to the given Rust struct: <CODE>. Custom action: {} Return only the modified code.", prompt),
                    RustAction::CustomFunctionAction(prompt) =>
                        format!("Please apply the following custom action to the given Rust function: <CODE>. Custom action: {} Return only the modified code.", prompt),
                    RustAction::CustomEnumAction(prompt) =>
                        format!("Please apply the following custom action to the given Rust enum: <CODE>. Custom action: {} Return only the modified code.", prompt)
                }
            }
        }
    }
}
