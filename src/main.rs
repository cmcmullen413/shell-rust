#[allow(unused_imports)]
use std::io::{self, Write};

const BUILT_INS: [&str;3] = ["exit", "echo", "type"];

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
        let buffer:String = buffer.trim_end().into();

        // If the command is "exit", break the loop
        if buffer == "exit" {
            break;
        }

        // If the command starts with "echo", echo the input arguments
        // If there are no input args, print the correct usage
        if buffer.starts_with("echo") {
            // Trim "echo " from the front of the buffer
            let args = buffer.trim_start_matches("echo").trim_start();

            // If args is empty, print the correct usage of echo
            if args.is_empty() {
                println!("No arguments provided. Correct usage: echo args");
                continue;
            }

            // Print the input back out with a new line
            println!("{}", args);
            continue;
        }

        // If the command starts with "type ", return what type of command the argument is
        if buffer.starts_with("type") {
            // Trim "type " from the front
            let arg = buffer.trim_start_matches("type").trim_start();

            // If no argument is provided, print the correct usage
            if arg.is_empty() {
                println!("No argument provided. Correct usage: type arg");
                continue;
            }

            // If the argument is in the built ins list, print that it is a built in
            if BUILT_INS.contains(&arg) {
                println!("{} is a shell builtin", arg);
                continue
            }
            // Otherwise, print that the command is not valid
            println!("{}: not found", arg);
            continue;
        }

        // If the command is not recognized:
        // Print it back out in the error message
        // Should be in the format {command}: command not found
        println!("{}: command not found", buffer);
    }
}
