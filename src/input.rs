use std::io::{self, Read, Write};

use owo_colors::OwoColorize;
use termios::Termios;

/// A wrapper around getting input from the user.
pub struct Input {
    /// The history of inputs
    history: Vec<String>,
    /// The current index in the history
    history_index: usize,
}

impl Input {
    const LEFT_ARROW: u8 = 68;
    const RIGHT_ARROW: u8 = 67;
    const UP_ARROW: u8 = 65;
    const DOWN_ARROW: u8 = 66;

    /// Creates a new input-wrapper.
    pub fn new() -> Self {
        let mut settings = Termios::from_fd(0).unwrap();
        settings.c_lflag &= !(termios::ICANON | termios::ECHO);

        termios::tcsetattr(0, termios::TCSANOW, &settings).unwrap();

        Self {
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Gets a line of input from the user.
    pub fn get_line(&mut self) -> String {
        print!("{} ", "helix ❯".green().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();

        // get one single character from the user
        let mut reader = io::stdin();
        let mut buffer = [0; 1];

        let mut cursor = 0;

        loop {
            // read a single byte from stdin
            reader.read_exact(&mut buffer).unwrap();

            let char = buffer[0] as char;

            // if the byte is a newline, break
            if char == '\n' {
                break;
            }

            // if the byte is an arrow, move the cursor
            if buffer[0] == 27 {
                reader.read_exact(&mut buffer).unwrap();
                reader.read_exact(&mut buffer).unwrap();

                if buffer[0] == Self::RIGHT_ARROW {
                    // right arrow
                    if cursor < input.len() {
                        cursor += 1;
                        print!("\x1B[1C");
                        io::stdout().flush().unwrap();
                    }
                } else if buffer[0] == Self::LEFT_ARROW {
                    // left arrow
                    if cursor > 0 {
                        cursor -= 1;
                        print!("\x1B[1D");
                        io::stdout().flush().unwrap();
                    }
                } else if buffer[0] == Self::UP_ARROW {
                    self.history_index = self.history_index.saturating_sub(1);

                    if let Some(last) = self.history.get(self.history_index) {
                        print!("\x1B[2K");
                        print!("\r");
                        print!("{} ", "helix ❯".green().bold());
                        print!("{}", last);

                        input = last.clone();
                        cursor = input.len();

                        io::stdout().flush().unwrap();
                    }
                } else if buffer[0] == Self::DOWN_ARROW {
                    // down arrow
                    self.history_index = usize::min(self.history_index + 1, self.history.len() + 1);
                    let next = self.history.get(self.history_index).unwrap_or(&input);

                    print!("\x1B[2K");
                    print!("\r");
                    print!("{} ", "helix ❯".green().bold());
                    print!("{}", next);

                    input = next.clone();
                    cursor = input.len();

                    io::stdout().flush().unwrap();
                }
            }
            // if the byte is a backspace, remove the last character
            else if buffer[0] == 127 {
                if !input.is_empty() {
                    input.remove(cursor - 1);
                    print!("\x08 \x08");
                    io::stdout().flush().unwrap();

                    cursor -= 1;
                }
            } else {
                // otherwise, push the character to the input at the cursor
                input.insert(cursor, char);

                // clear the line
                print!("\x1B[1K");
                print!("\r");
                print!("{} ", "helix ❯".green().bold());
                print!("{}", input);

                io::stdout().flush().unwrap();
                cursor += 1;
            }
        }

        // add the input to the history
        self.history.push(input.clone());
        self.history_index = self.history.len();

        // print a newline
        println!();

        input
    }
}

/// Prints the intro message to the user.
/// (When an instance of the REPL is started)
pub fn print_intro() {
    let version = format!("v{} (stdin)", env!("CARGO_PKG_VERSION"));

    println!("{} {}", "Helix".green().bold(), version.dimmed());
    println!("  Type {} for more information.", "#help".cyan().bold());
}
