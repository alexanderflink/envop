mod op_utils;
mod utils;
use inquire;
use op_utils::{op_get_vaults, op_sign_in, op_whoami};
use std::fs;
use std::process;
use utils::{compare_env_vars, get_argument_or_default, parse_env_file};

fn main() {
    let env_file_path = get_argument_or_default(1, ".env");
    let provision_file_path = get_argument_or_default(2, ".env.provision");

    let op_signed_in = op_whoami();

    if !op_signed_in {
        println!("You are not logged in to 1password CLI. Proceeding to log in...");

        op_sign_in();
    }

    let env_file_contents = fs::read_to_string(&env_file_path).unwrap_or_else(|_| {
        println!("Could not read environment file at {}", &env_file_path);
        process::exit(1)
    });

    let provision_file_contents =
        fs::read_to_string(&provision_file_path).unwrap_or(String::from(""));

    let env_vars = parse_env_file(env_file_contents);
    let provision_vars = parse_env_file(provision_file_contents);

    let unsynced_env_vars = compare_env_vars(&env_vars, &provision_vars);

    let unsynced_env_var_keys: Vec<&String> =
        unsynced_env_vars.iter().map(|env| &env.key).collect();

    let keys_to_sync = match inquire::MultiSelect::new(
        "Select environment variables to sync:",
        unsynced_env_var_keys,
    )
    .prompt()
    {
        Ok(keys) => {
            if keys.len() == 0 {
                println!("No keys to sync, exiting");
                process::exit(0);
            }

            keys
        }
        Err(_) => {
            println!("Could not read keys");
            process::exit(1);
        }
    };

    let vaults = op_get_vaults();

    let vault_names: Vec<&String> = vaults.iter().map(|vault| &vault.name).collect();

    inquire::Select::new("Select vault:", vault_names).prompt();
}
