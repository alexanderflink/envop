use std::env;
use std::str::Split;

#[derive(Debug)]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
}

pub fn parse_env_file(file_contents: String) -> Vec<EnvVariable> {
    let split: Split<&str> = file_contents.split("\n");

    return split
        .map(|env| {
            let mut env_iterator: Split<&str> = env.split("=");

            let key = env_iterator.next().unwrap();
            let value = env_iterator.next().unwrap();

            EnvVariable {
                key: String::from(key),
                value: String::from(value),
            }
        })
        .collect();
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
