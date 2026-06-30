#[allow(unused_imports)]
use std::io::{self, Write, Read};
use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};
use std::ptr::null;
use is_executable::IsExecutable;

const BUILTINS: [&str;5] = ["exit", "echo", "type", "pwd", "cd"];

// Struct for storing information about redirecting the terminal output
struct RedirectInfo {
    stdout: bool,
    stderr: bool,
    destination: String,
    append: bool
}


fn main() {
    // Begin looping
    loop {
        // Print the prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Get the users input and parse it into a command and args
        let mut command_string = String::new();
        let mut args_string = Vec::new();
        // Also gets whether the output should be redirected or not and in what way
        let mut redirect_info = RedirectInfo {
            stdout: false,
            stderr: false,
            destination: String::new(),
            append: false
        };

        parse_input(&mut command_string, &mut args_string, &mut redirect_info);
        // Convert the Strings into &str to pass to the rest of the functions
        let command = command_string.as_str();
        let args_str: Vec<&str> = args_string.iter().map(|s| s.as_str()).collect();
        let args: &[&str] = &args_str;

        match command {
            // If the command is exit, break the outer loop
            "exit" => break,
            // The other builtin commands
            "echo" => handle_echo(&args, redirect_info),
            "type" => handle_type(&args, redirect_info),
            "pwd" => handle_pwd(&args, redirect_info),
            "cd" => handle_cd(&args, redirect_info),
            // If the command is not recognized
            _ => {
                // First check if it is an executable
                match !is_path_executable(&command).is_empty() {
                    true => {
                        // If it is call the method to create and handle the child process
                        handle_non_builtin(&command, &args, redirect_info);
                        // Once the child has finished, get a new command from the user
                        continue
                    },
                    false => {
                        // If it isn't
                        // Print it back out in the error message formated as -> {command}: command not found
                        println!("{}: command not found", command);
                    }
                }
            }
        }
    }
}

/// Parses user input and fills the provided command and args outputs
/// Returns whether the parsing was successful
fn parse_input(command_out: &mut String, args_out: &mut Vec<String>, redirect_info: &mut RedirectInfo) -> bool{
    // Read the user's input into a buffer
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    // Trim the white space from the ends
    let buffer= buffer.trim().to_string();

    // Flag to track the success of the parsing
    let mut success = true;

    // Figure out if the output needs to be redirected in any way
    let mut buffer = parse_redirect(buffer, redirect_info);

    // Get the command from the front of the string and put it into command_out
    let args = parse_command(&mut buffer, &mut success);
    command_out.push_str(&buffer);
    // Parse the remaining argument string and put it into args_out
    let mut args_vec = parse_args(args.chars().collect());
    args_out.append(&mut args_vec);

    success
}

/// Parses any redirects in the input
fn parse_redirect(input: String, redirect_info: &mut RedirectInfo) -> String {
    // Loop through looking for a '>' char
    // If one is found, change the values of redirect info accordingly
    //  then return the input with the redirection part cut off

    // Convert the input into a vec of chars to be iterated over
    let input_vec:Vec<char> = input.chars().collect();

    // What output goes to the file can depend on this
    for (i, &c) in input_vec.iter().enumerate() {
        if c == '>' {
            // Keep a variable of where the end needs to be cut off to pass back out to the other parsers
            let mut start_index = i;
            // Also keep where the end of the redirection characters are to cut off and get the output file
            let mut end_index = i + 1;

            // If the last char was 2 or &, redirect stderr
            if input_vec[i-1] == '2' || input_vec[i-1] == '&' {
                // Move the start index
                start_index -= 1;

                // Set the stderr redirect flag in redirect info
                redirect_info.stderr = true;
            }
            // If the last char was anything but 2, redirect stdout
            if input_vec[i-1] != '2' {
                // Move the start index back one if the char was one of the recognized ones (1 or &)
                if input_vec[i-1] == '1' || input_vec[i-1] == '&' { start_index -= 1; }

                // Set the stdout redirection flag in redirection info
                redirect_info.stdout = true;
            }
            // If the next char is also '>' set the append mode
            if input_vec[i+1] == '>' {
                redirect_info.append = true;
                // Move the end index back by one
                end_index += 1;
            }

            // Create a copy of input to mutate into the output string
            let mut output = String::clone(&input);

            // Get the file to redirect the output to
            let file = output.split_off(end_index);
            // Set the file handle in the redirection info
            redirect_info.destination = file.trim().to_string();

            // Remove the rest of the redirection chars and return the rest of the input string
            let _ = output.split_off(start_index);
            return output
        }
    }

    // If there is no redirection, just return input
    input
}

/// Parses the command from the front of the input string
/// Modifies the passed in string to contain just the command and returns the args as a new String
fn parse_command(input: &mut String, success: &mut bool) -> String {
    // If the command starts with a single quote, the command is everything between it and the next single quote
    //  nothing is escaped and all characters are treated as literals
    // If the command starts with a double quote, the command is everything between in and the next
    //  double quote, escaping characters the same way as when parsing args
    if input.starts_with(&['\'', '"']) {
        // Remove the first char from the string
        let prefix = input.remove(0);

        // Find the next occurrence of the prefix
        let prefix_index = match input.find(prefix) {
            // If one isn't found, clear the input and return an empty string
            //  also print an error message out
            None => {
                println!("Incorrect use of single quotes in command. To use quotes, command must be preceded and succeeded by the same type of quote (' or \")");
                *success = false;
                input.clear();
                return String::new()
            },
            Some(index) => index + 1
        };

        // Split the string at the found index
        // The returned value will be the right half (args) and the String left behind will be the left half (command + ')
        let args = input.split_off(prefix_index);
        // Trim the quote left behind on the command
        input.pop();

        // If the prefix was a double quote, find all the backslashes to remove them so the following chars are escaped
        if prefix == '"' {
            // Start a loop to find each occurrence of the backslash char
            // If the following char is another backslash, set a flag to skip it
            // Either way add the index to a list of chars to be removed at the end of the loop
            // Each index is added to the front so when they are removed the indices are preserved for the ones to be removed after
            let mut skip_flag = false;
            let mut removals = Vec::new();
            for (i, c) in input.chars().enumerate() {
                if skip_flag {
                    skip_flag = false;
                    continue
                }
                if c == '\\' {
                    skip_flag = true;
                    removals.push(i)
                }
            }
            // Now remove all the chars marked for removal
            for i in removals { input.remove(i); }
        }

        // Return the args
        return args
    }

    // If the command doesn't start with a quote, split the command with a space like normal
    // Split the string at the first space. The returned value will be the right half (args
    //  and the String left behind will be the left half (command)

    // Get the index of the first space
    let space_index = match input.find(" ") {
        // If there isn't one leave the string alone and return an empty string as the args
        None => {
            return String::new()
        },
        Some(index) => index + 1
    };

    // Split off the command
    let args =input.split_off(space_index);
    // Trim the space left behind on the command
    input.pop();

    // Return the command
    args
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
fn handle_echo(args: &[&str], redirect_info: RedirectInfo) {
    // If there are no arguments, print the correct usage
    if args.len() == 0 {
        print_error("No arguments provided. Correct usage: echo arg1 arg2 ...\n", redirect_info);
        return
    }
    // Otherwise, print out all the arguments
    let mut output = String::new();
    // Add the first one outside the loop so the spaces can be added before the rest
    output.push_str(&args[0]);
    for arg in args.iter().skip(1) {
        output.push_str(&format!(" {}", arg));
    }
    // Append a newline char
    output.push('\n');

    print_out(output.as_str(), redirect_info);
}

/// Handles the type command with the passed arguments
fn handle_type(args: &[&str], redirect_info: RedirectInfo) {
    // If no or too many arguments are provided, print the correct usage
    if args.len() != 1 {
        print_error("One argument expected. Correct usage: type command\n", redirect_info);
        return
    }
    // Redefine the arg as just one string
    let arg = args[0];
    // Check if the arg is in the builtins list. If it is, print that message out
    if BUILTINS.contains(&arg) {
        print_out(&format!("{} is a shell builtin\n", arg), redirect_info);
        return
    }
    // Check if the arg is an executable in the environment PATH
    let path = is_path_executable(&arg);
    if !path.is_empty() {
        print_out(&format!("{} is {}\n", arg, path), redirect_info);
        return
    }
    // Otherwise, print that the command wasn't found
    print_out(&format!("{}: not found\n", arg), redirect_info);
}

/// Handles the pwd command with the passed arguments
fn handle_pwd(args: &[&str], redirect_info: RedirectInfo) {
    // If any arguments were passed, print the correct usage
    if args.len() > 0 {
        print_error("No arguments expected. Correct usage: pwd\n", redirect_info);
        return
    }

    // Print the current directory out
    print_out(&format!("{}\n", get_working_dir()), redirect_info);

}

/// Handles the cd command with the passed arguments
fn handle_cd(args: &[&str], redirect_info: RedirectInfo) {
    // If no or too many arguments are provided, print the correct usage
    if args.len() != 1 {
        print_error("One argument expected. Correct usage: cd path\n", redirect_info);
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
            print_out(&format!("cd: {}: No such file or directory\n", arg), redirect_info);
        }
    }
}

/// Handles non builtin command calls with the passed arguments
fn handle_non_builtin(command: &str, args: &[&str], redirect_info: RedirectInfo) {
    // Create a child process with the input connected to this parent and the out and err connected to this or a file depending
    // This will run the process and then return the status after it finishes (Which isn't used)

    // If either stdout or stderr are redirecting, open or create the file
    let mut stdout = Stdio::inherit();
    let mut stderr = Stdio::inherit();

    match redirect_info.stdout || redirect_info.stderr {
        true => {
            // Try to open the file in the correct append/overwrite mode
            // If it doesn't exist, create a new file instead
            let file = match OpenOptions::new().write(true).append(redirect_info.append).open(&redirect_info.destination) {
                Ok(file) => file,
                Err(_) => File::create(&redirect_info.destination).unwrap()
            };

            if redirect_info.stdout {
                stdout = Stdio::from(file.try_clone().unwrap())
            }
            if redirect_info.stderr {
                stderr = Stdio::from(file)
            }
        },
        false => ()
    }

    Command::new(command)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .expect("Failed to execute command");
}

fn print_error(message: &str, redirect_info: RedirectInfo) {
    // If stderr is redirected, pass the message along
    if redirect_info.stderr {
        print_to_file(message, redirect_info.destination.as_str(), redirect_info.append);
        return
    }
    // Otherwise, print it to stdout
    print!("{}", message)
}

fn print_out(message: &str, redirect_info: RedirectInfo) {
    // If stdout is redirected, pass the message along
    if redirect_info.stdout {
        print_to_file(message, redirect_info.destination.as_str(), redirect_info.append);
        return
    }
    // Otherwise, print it to stdout
    print!("{}", message)
}

fn print_to_file(message: &str, file_path: &str, append: bool) {
    // Get the actual path of the passed in file path string
    let path = Path::new(file_path);

    // Try to open the file in the correct append/overwrite mode
    // If it doesn't exist, create a new file instead
    let mut file = match OpenOptions::new().write(true).append(append).open(path) {
        Ok(file) => file,
        Err(_) => File::create(path).unwrap()
    };

    if let Err(e) = write!(file, "{}", message) {
        println!("Couldn't write to file: {}", e);
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