#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::env::split_paths;
use std::fs;
use is_executable::IsExecutable;

const BUILTINS: [&str;3] = ["exit", "echo", "type"];

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

        // Split the command and arguments
        let splits = buffer.split(" ").collect::<Vec<&str>>();
        let command = splits[0];
        let args = &splits[1 ..];

        match command {
            // If the command is exit, break the outer loop
            "exit" => break,
            // The other builtin commands
            "echo" => handle_echo(args),
            "type" => handle_type(args),
            // If the command is not recognized:
            _ => {
                // Print it back out in the error message formated as -> {command}: command not found
                println!("{}: command not found", buffer);
            }
        }
    }
}

/// Handles the echo command with the passed arguments
fn handle_echo(args: &[&str]) {
    // If there are no arguments, print the correct usage
    if args.len() == 0 {
        println!("No arguments provided. Correct usage: echo arg1 arg2 ...");
        return
    }
    // Otherwise, print out all the arguments
    // Add the first one outside the loop so the spaces can be added before the rest
    print!("{}", args[0]);
    for arg in args.iter().skip(1) {
        print!(" {}", arg)
    }
    // Print the new line. This also flushes the buffer
    println!()
}

/// Handles the type command with the passed arguments
fn handle_type(args: &[&str]) {
    // If no or too many arguments are provided, print the correct usage
    if args.len() != 1 {
        println!("None or too many arguments provided. Correct usage: type arg");
        return
    }
    // Redefine the arg as just one string
    let arg = args[0];
    // Check if the arg is in the builtins list. If it is, print that message out
    if BUILTINS.contains(&arg) {
        println!("{} is a shell builtin", arg);
        return
    }
    // Check if the arg is an executable in the environment PATH
    let path = is_path_executable(&arg);
    if !path.is_empty() {
        println!("{} is {}", arg, path);
        return
    }
    // Otherwise, print that the command wasn't found
    println!("{}: not found", arg);
}

/// Checks if the provided argument is an executable in the environment PATH
fn is_path_executable(arg: &str) -> String {
    // Get the actual name of the executable being looked for
    // This depends on the system type, but can be gotten by appending the exe suffix to the arg
    let arg = arg.to_owned() + env::consts::EXE_SUFFIX;

    // Get the environmental PATH variable
    let paths = env::var_os("PATH").unwrap();
    let paths = split_paths(&paths);
    // For each path in the PATH, check each of it's subfiles for the arg
    for path in paths {
        // Get the contents of the directory
        match fs::read_dir(path) {
            Ok(dir) => {
                for file in dir {
                    let path = file.as_ref().unwrap().path();
                    // Check if the file name matches and is executable
                    if file.unwrap().file_name().to_str().unwrap() == arg && path.is_executable() {
                        // If they are, return the path to the file
                        return path.to_str().unwrap().to_owned()
                    }
                }
            },
            // If the dir can't be read (usually because of permissions) just skip it
            Err(_error) => ()
        }
    }
    // If no match was found, return an empty string
    String::new()
}