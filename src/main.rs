#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::fs;
use is_executable::IsExecutable;

const BUILTINS: [&str;3] = ["exit", "echo", "type"];

fn main() {
    // Begin looping
    'outer: loop {
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

            // If the argument is in the builtins list, print that it is a built in
            if BUILTINS.contains(&arg) {
                println!("{} is a shell builtin", arg);
                continue
            }

            // If the arg is not a builtin, check if it is an executable in the path
            //
            // If this is a windows system, the file being looked for will be appended with .exe to search for an executable
            // Create a new variable that will hold the actual file name on any system
            let arg_executable = arg.to_owned() + env::consts::EXE_SUFFIX;
            //
            // Get the environment PATH var
            let paths = env::var_os("PATH").unwrap();
            for path in env::split_paths(&paths) {
                // Get the contents of the directory
                match fs::read_dir(path) {
                    Ok(dir) => {
                        // Check each file in the dir
                        for file in dir {
                            let file_name = file.as_ref().unwrap().file_name();
                            // If the name matches the arg and it is executable, print that out
                            if file_name.into_string().unwrap() == arg_executable && file.as_ref().unwrap().path().is_executable() {
                                println!("{} is {}", arg, file.unwrap().path().display());
                                // Continue the outer loop
                                continue 'outer;
                            }
                            // Otherwise keep looking
                        }
                    }
                    // If there is an error accessing the directory, just silence it
                    Err(_error) => ()
                }
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
