#[allow(unused_imports)]
use std::io::{self, Write, Read};
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use is_executable::IsExecutable;

const BUILTINS: [&str;5] = ["exit", "echo", "type", "pwd", "cd"];

fn main() {
    // Begin looping
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read the user's input into a buffer
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        // Trim the white space from the end
        let buffer = buffer.trim_end();

        // Split the command and arguments from the input
        // First get just the command
        let split = buffer.split(" ").collect::<Vec<&str>>();
        let command = split[0];
        // Remove the command from the front and pass the rest as a vec of chars to the parser
        let args_vec = parse_args(buffer.strip_prefix(command).unwrap().trim_start().chars().collect());
        // Convert the Strings to &str
        let args_str: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();
        // Finally collect the args into an array of str
        let args: &[&str] = &args_str;

        match command {
            // If the command is exit, break the outer loop
            "exit" => break,
            // The other builtin commands
            "echo" => handle_echo(args),
            "type" => handle_type(args),
            "pwd" => handle_pwd(args),
            "cd" => handle_cd(args),
            // If the command is not recognized
            _ => {
                // First check if it is an executable
                // If it is
                if !is_path_executable(&command).is_empty() {
                    // Create a child process with the input and output connected to this parent
                    // This will run the process and then return the status after it finishes (Which isn't used)
                    Command::new(command)
                        .args(args)
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .status()
                        .expect("Failed to execute command");

                    // Once the child has finished, get a new command from the user
                    continue
                }

                // If it isn't
                // Print it back out in the error message formated as -> {command}: command not found
                println!("{}: command not found", buffer);
            }
        }
    }
}

/// Parses the arguments from the input string
/// Fills the provided vector with the arguments
fn parse_args(input: Vec<char>) -> Vec<String> {
    // Instantiate the output vec
    let mut output = Vec::new();

    // Iterate over all the characters and add them to a buffer
    // Track whether we are inside of double or single quotes or escaping a char
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut escaped_char = false;
    // When a space is reached, if the buffer has chars, add them to the output
    let mut buf = String::new();
    for &c in input.iter() {
        // If the char was escaped, add it to the buffer no matter what the flip the flag back
        if escaped_char {
            buf.push(c);
            escaped_char = false;
            continue
        }
        // If a backslash is reached, set the escape flag
        if c == '\\' && !in_single_quotes {
            escaped_char = true;
            continue
        }
        // If a double quote is reached, flip the flag
        if c == '"' && !in_single_quotes{
            in_double_quotes = !in_double_quotes;
            continue
        }

        // If a single quote is reached, flip the flag if we're not inside double quotes
        if c == '\'' && !in_double_quotes{
            in_single_quotes = !in_single_quotes;
            continue
        }
        // If a space is not reached and or we are in single quotes or double quotes, just push the char to the buffer
        if c != ' ' || in_single_quotes || in_double_quotes {
            buf.push(c);
            continue
        }
        // If a space is reached outside single quotes and the buffer is not empty, push it to the output and clear it
        if !buf.is_empty() {
            output.push(buf);
            buf = String::new()
        }
    }
    // At the end, if the buffer isn't empty, push it to the output as well
    if !buf.is_empty() { output.push(buf) }
    // Return output
    output
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
        println!("One argument expected. Correct usage: type command");
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

/// Handles the pwd command with the passed arguments
fn handle_pwd(args: &[&str]) {
    // If any arguments were passed, print the correct usage
    if args.len() > 0 {
        println!("No arguments expected. Correct usage: pwd")
    }

    // Print the current directory out
    println!("{}", get_working_dir())
}

/// Handles the cd command with the passed arguments
fn handle_cd(args: &[&str]) {
    // If no or too many arguments are provided, print the correct usage
    if args.len() != 1 {
        println!("One argument expected. Correct usage: cd path");
        return
    }

    // Get the first argument
    let mut arg = args[0].to_string();

    // If the path starts with '~', replace it with the home directory
    if arg.starts_with("~") {
        arg = env::home_dir().expect("Could not get home directory").to_str().unwrap().to_owned()
            + arg.strip_prefix("~").unwrap()
    }

    // Set the current directory to the passed path
    match env::set_current_dir(&arg) {
        Ok(_) => (),
        // If the directory failed to change, print the error out
        Err(_) => {
            println!("cd: {}: No such file or directory", arg)
        }
    }
}

/// Checks if the provided argument is an executable in the environment PATH
fn is_path_executable(arg: &str) -> String {
    // Get the actual name of the executable being looked for
    // This depends on the system type, but can be gotten by appending the exe suffix to the arg
    let arg = arg.to_owned() + env::consts::EXE_SUFFIX;

    // Get the environmental PATH variable
    let paths = env::var_os("PATH").unwrap();
    let paths = env::split_paths(&paths);
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
            // TODO: Figure out why this happens and handle it properly. Seems to work fine however
            Err(_error) => ()
        }
    }
    // If no match was found, return an empty string
    String::new()
}

/// Gets the current directory of the process
fn get_working_dir() -> String {
    // Get the path buf of the current directory
    let dir = env::current_dir().unwrap();
    // Return the string form of it
    dir.into_os_string().into_string().unwrap()
}