use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::process;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub struct OPVault {
    pub id: String,
    pub name: String,
}

pub fn op_get_vaults() -> Vec<OPVault> {
    match Command::new("op")
        .args(["vault", "list", "--format=json"])
        .output()
    {
        Ok(output) => {
            let output_string =
                String::from_utf8(output.stdout).expect("Error reading op vault list");

            let vaults: Vec<OPVault> = serde_json::from_str(&output_string).unwrap();

            vaults
        }
        Err(_) => {
            println!("Could not run 1password CLI. Please make sure it is installed.");

            process::exit(1);
        }
    }
}

pub fn op_sign_in() -> bool {
    match Command::new("op").args(["signin"]).output() {
        Ok(output) => {
            if output.status.code().unwrap_or(1) == 1 {
                println!("Could not log in to 1password. Please try again.");

                process::exit(1);
            }

            true
        }
        Err(_) => {
            println!("Could not run 1password CLI. Please make sure it is installed.");

            process::exit(1);
        }
    }
}

pub fn op_whoami() -> bool {
    match Command::new("op").args(["whoami"]).output() {
        Ok(output) => output.status.code().unwrap_or(1) == 0,
        Err(_) => {
            println!("Could not run 1password CLI. Please make sure it is installed.");

            process::exit(1);
        }
    }
}
