use crate::EnvVariable;
use core::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::str::Split;

impl fmt::Display for EnvVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.key)
    }
}

pub struct EnvVariables(pub Vec<EnvVariable>);

impl fmt::Display for EnvVariables {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, env| {
            result.and_then(|_| writeln!(f, "{}", env.key))
        })
    }
}

pub fn write_to_file(file_path: &str, content: &str) -> Result<(), io::Error> {
    fs::OpenOptions::new()
        .append(true)
        .open(file_path)?
        .write_all(content.as_bytes())
}

pub fn read_env_file(file_path: &str) -> io::Result<String> {
    fs::read_to_string(file_path)
}

pub fn ask_select_item<T: fmt::Display>(
    message: &str,
    items: Vec<T>,
) -> Result<T, inquire::InquireError> {
    inquire::Select::new(message, items).prompt()
}

pub fn ask_select_items<T: fmt::Display>(
    message: &str,
    items: Vec<T>,
) -> Result<Vec<T>, inquire::InquireError> {
    inquire::MultiSelect::new(message, items).prompt()
}

pub fn ask_create_item(message: &str) -> String {
    inquire::Text::new(message).prompt().expect("Missing input")
}

pub fn ask_proceed(message: &str, default: bool) -> bool {
    inquire::Confirm::new(message)
        .with_default(default)
        .prompt()
        .expect("Error getting confirmation")
}

pub fn parse_env_file(file_contents: &str) -> Vec<EnvVariable> {
    let split: Split<&str> = file_contents.split("\n");

    let env_variables: Vec<EnvVariable> = split
        .filter_map(|line| {
            let env;

            if line.contains("=") {
                let mut env_iterator: Split<&str> = line.split("=");

                let key = env_iterator.next()?;
                let value = env_iterator.next()?;

                env = Some(EnvVariable {
                    key: String::from(key),
                    value: String::from(value),
                });
            } else {
                env = None;
            }

            env
        })
        .collect();

    env_variables
}
