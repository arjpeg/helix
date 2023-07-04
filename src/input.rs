use std::io::Write;

use owo_colors::OwoColorize;

/// Prints the prompt to the user.
pub fn print_prompt() {
    print!("{}", "helix ❯ ".green().bold());
    std::io::stdout().flush().unwrap();
}

/// Prints the intro message to the user.
/// (When an instance of the REPL is started)
pub fn print_intro() {
    println!("{} {}", "Helix".green().bold(), "v0.1.0 (stdin)".dimmed());
    println!("  Type {} for more information.", "#help".bold());
}

/// Gets a line of input from the user.
pub fn get_input() -> String {
    let mut input = String::new();

    print_prompt();

    std::io::stdin().read_line(&mut input).unwrap();

    input
}
