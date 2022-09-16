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
    let mut file = fs::OpenOptions::new().append(true).open(file_path)?;

    file.write_all(content.as_bytes())
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

    let mut line_number = 0;

    let env_variables: Vec<EnvVariable> = split
        .map(|line| {
            let env;

            if line.contains("=") {
                let mut env_iterator: Split<&str> = line.split("=");

                let key = env_iterator.next()?;
                let value = env_iterator.next()?;

                env = Some(EnvVariable {
                    key: String::from(key),
                    value: String::from(value),
                    line: line_number,
                });
            } else {
                env = None;
            }

            line_number += 1;

            env
        })
        .filter(|env| env.is_some())
        .map(|env| env.unwrap())
        .collect();

    // split.try_for_each(|line| {
    //     if line.contains("=") {
    //         let mut env_iterator: Split<&str> = line.split("=");

    //         let key = env_iterator.next()?;
    //         let value = env_iterator.next()?;

    //         env_variables.push(EnvVariable {
    //             key: String::from(key),
    //             value: String::from(value),
    //             line: line_number,
    //         });
    //     }

    //     line_number += 1;
    //     Some(())
    // });

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
