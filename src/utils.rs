use core::fmt;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;
use std::str::Split;

#[derive(Debug)]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
    pub line: i32,
}

pub struct EnvVariables(pub Vec<EnvVariable>);

impl fmt::Display for EnvVariables {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, env| {
            result.and_then(|_| writeln!(f, "{}", env.key))
        })
    }
}

pub fn write_to_file(file_path: &String, content: String) -> Result<(), io::Error> {
    let mut file = fs::OpenOptions::new().append(true).open(file_path)?;

    file.write_all(content.as_bytes())
}

pub fn strip_trailing_newline(input: String) -> String {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(&input)
        .to_string()
}

pub fn read_env_file(file_path: &String) -> String {
    fs::read_to_string(file_path).unwrap_or_else(|_| {
        println!("Could not read environment file at {}", file_path);
        process::exit(1)
    })
}

pub fn ask_select_item<T: fmt::Display>(message: &str, items: Vec<T>) -> T {
    inquire::Select::new(message, items)
        .prompt()
        .expect("Nothing selected")
}

pub fn ask_create_item(message: &str) -> String {
    inquire::Text::new(message).prompt().expect("Missing input")
}

pub fn ask_proceed(message: String, default: bool) -> bool {
    inquire::Confirm::new(&message)
        .with_default(default)
        .prompt()
        .expect("Error getting confirmation")
}

pub fn parse_env_file(file_contents: &String) -> Vec<EnvVariable> {
    let split: Split<&str> = file_contents.split("\n");

    let mut env_variables: Vec<EnvVariable> = Vec::new();
    let mut line_number = 0;

    split
        // .filter(|env| env.contains("="))
        .for_each(|line| {
            if line.contains("=") {
                let mut env_iterator: Split<&str> = line.split("=");

                let key = env_iterator.next().unwrap();
                let value = env_iterator.next().unwrap();

                env_variables.push(EnvVariable {
                    key: String::from(key),
                    value: String::from(value),
                    line: line_number,
                });
            }

            line_number += 1;
        });

    env_variables
}

pub fn get_argument_or_default(index: usize, default: &str) -> String {
    let args: Vec<String> = env::args().collect();

    match args.get(index) {
        Some(arg) => String::from(arg),
        None => String::from(default),
    }
}

pub fn compare_env_vars<'a>(
    first_vars: &'a Vec<EnvVariable>,
    second_vars: &Vec<EnvVariable>,
) -> Vec<&'a EnvVariable> {
    // for each var in first_vars, check if var exists and is the same in second_vars
    first_vars
        .iter()
        .filter(|env_var| {
            let found_provision_var = second_vars.iter().find(|provision_var| {
                let key_matches = env_var.key == provision_var.key;
                let value_matches = env_var.value == provision_var.value;

                key_matches && value_matches
            });

            found_provision_var.is_none()

            // if found_provision_var.is_some() {
            //     println!("{:?}", found_provision_var.unwrap());
            // }
        })
        .collect()
}
