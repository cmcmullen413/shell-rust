#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Begin looping
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read the user's input into a buffer
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        // Remove the tailing new line char  (or chars in case of windows)
        let buffer = buffer.trim_end();

        // If the command is "exit", break the loop
        if buffer == "exit" {
            break;
        }

        // If the command is not recognized:
        // Print it back out in the error message
        // Should be in the format {command}: command not found
        println!("{}: command not found", buffer);
    }
}
