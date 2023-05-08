//! A simple module for user input.

use std::io::{stdin, Write};

use owo_colors::OwoColorize;

/// Draw a prompt arrow
fn prompt_indicator() {
    print!("{}", "helix ❯ ".green().bold());

    // Flush the buffer
    std::io::stdout()
        .flush()
        .expect("failed to write to standard output");
}

/// Simple function to get user input
fn read_user_input() -> String {
    let mut input = String::new();

    stdin()
        .read_line(&mut input)
        .expect("failed to read from standard input");

    input
}

/// Draws a prompt arrow and returns the user input
pub fn prompt() -> String {
    prompt_indicator();
    read_user_input()
}
