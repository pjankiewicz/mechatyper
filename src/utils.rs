use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

use colored::Colorize;
use openai::set_key;

pub fn find_git_directory(mut path: PathBuf) -> Option<PathBuf> {
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

pub fn load_env_variables() {
    dotenv::dotenv().expect("Failed to read .env file");
    set_key(env::var("OPENAI_KEY").expect("OPENAI_KEY not set"));
}

fn clear_screen() {
    // This is the ANSI escape code to clear the screen
    print!("\x1B[2J\x1B[1;1H");
    // Flush the output to ensure it is displayed
    stdout().flush().unwrap();
}

pub fn print_introduction() {
    clear_screen();
    println!(
        "{}",
        "Welcome to MechaTyper! Here you can interactively work with the program.\nType in your task, and get assistance!".bright_blue()
    );
}

pub fn get_user_input(prompt: &str) -> anyhow::Result<String> {
    print!("{}: ", prompt);
    stdout().flush()?;
    let mut user_input = String::new();
    stdin().read_line(&mut user_input)?;
    Ok(user_input)
}
