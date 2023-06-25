use strum_macros::{EnumString, EnumVariantNames};

#[derive(Clone, Debug, EnumString, EnumVariantNames)]
pub enum CodeAction {
    CustomAction(String),
    CommonAction(CommonAction),
    PythonAction(PythonAction),
    RustAction(RustAction),
}

#[derive(Clone, Debug, EnumString, EnumVariantNames)]
pub enum CommonAction {
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
