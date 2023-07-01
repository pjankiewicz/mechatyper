use crate::code_cleaning::extract_python_code;
use crate::lang::{ProgItem, PythonProgItem};
use crate::prompts::quickcheck_prompt;
use crate::search::parse_code;
use dotenv::dotenv;
use openai::chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole};
use openai::set_key;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use tokio::runtime::Runtime;

pub fn load_env_variables() {
    dotenv().expect("Failed to read .env file");
    set_key(env::var("OPENAI_KEY").expect("OPENAI_KEY not set"));
}

async fn process_chat_prompt(
    prompt: &str,
    default_output: String,
) -> Result<String, Box<dyn Error>> {
    load_env_variables();

    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: Some("You are a code assists that writes Python code without any additional comments or explanations".to_string()),
        name: None,
        function_call: None,
    }];

    messages.push(ChatCompletionMessage {
        role: ChatCompletionMessageRole::User,
        content: Some(prompt.to_string()),
        name: None,
        function_call: None,
    });

    let mut attempt_count = 0;
    let max_attempts = 3;

    loop {
        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo-16k-0613", messages.clone())
            .create()
            .await?;

        let returned_message = chat_completion.choices.first().unwrap().message.clone();
        let content = returned_message.content.clone().unwrap().trim().to_string();

        println!("Answer: {}", content);

        match parse_code(&content, &ProgItem::Python(PythonProgItem::Function)) {
            Ok(_) => return Ok(content),
            Err(_) => {
                if let Some(python_code) = extract_python_code(&content) {
                    match parse_code(&python_code, &ProgItem::Python(PythonProgItem::Function)) {
                        Ok(_) => return Ok(python_code),
                        Err(_) if attempt_count < max_attempts => {
                            messages.push(ChatCompletionMessage {
                                role: ChatCompletionMessageRole::User,
                                content: Some("Please fix the Python code.".to_string()),
                                name: None,
                                function_call: None,
                            });
                            attempt_count += 1;
                        }
                        Err(_) => return Ok(default_output),
                    }
                } else if attempt_count < max_attempts {
                    messages.push(ChatCompletionMessage {
                        role: ChatCompletionMessageRole::User,
                        content: Some("Please provide Python code.".to_string()),
                        name: None,
                        function_call: None,
                    });
                    attempt_count += 1;
                } else {
                    return Ok(default_output);
                }
            }
        }
    }
}

// ... include the other functions `extract_python_code`, `parse_code`, and the necessary enum definitions ...

#[test]
fn main_test() {
    // Create a new runtime
    let rt = Runtime::new().unwrap();

    // Use the runtime to block on the async function
    rt.block_on(async {
        let prompt = quickcheck_prompt(
            "replace all .unwrap calls to .expect with a proper message in Rust functions",
        );
        let default_output = "Unable to retrieve Python code.".to_string();

        match process_chat_prompt(&prompt, default_output).await {
            Ok(result) => println!("Result:\n{}", result),
            Err(e) => println!("An error occurred: {}", e),
        }
    });
}
