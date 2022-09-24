mod op_utils;
mod utils;
use argh::FromArgs;
use glob::glob;
use op_utils::{
    op_create_item, op_edit, op_field_in_section, op_field_to_env_var,
    op_field_to_env_var_reference, op_get_item, op_get_items, op_get_vaults, op_inject, op_sign_in,
    op_whoami, OPItem, OPSection,
};
use std::fs;
use std::io::Write;
use std::path;
use std::process;
use utils::{
    ask_create_item, ask_proceed, ask_select_item, ask_select_items, parse_env_file, read_env_file,
    write_to_file,
};

#[derive(Debug)]
pub struct EnvVariable {
    pub key: String,
    pub value: String,
}

#[derive(FromArgs)]
/// Sync environment variables using 1password. Requires the 1password CLI to be installed: https://1password.com/downloads/command-line/. This CLI will not delete any items from 1password, it will only add and update their values. Deletion has to be done manually. Syncing is done using provisioning files which point to a secret in a 1password vault. Different environments such as "staging" and "production" are best handled using sections in 1password.

struct Args {
    #[argh(subcommand)]
    subcommand: SubCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommands {
    Up(SyncUpOptions),
    Down(SyncDownOptions),
}

#[derive(FromArgs, PartialEq, Debug)]
/// sync variables from .env file to 1password vault
#[argh(subcommand, name = "up")]
struct SyncUpOptions {
    #[argh(option, default = "String::from(\".env\")")]
    /// path to .env file
    env: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// sync variables from 1password vault to .env file
#[argh(subcommand, name = "down")]
struct SyncDownOptions {
    #[argh(option, default = "String::from(\".env\")")]
    /// path to .env file
    env: String,
}

fn main() {
    let args: Args = argh::from_env();

    match args.subcommand {
        SubCommands::Up(options) => sync_up(options),
        SubCommands::Down(options) => sync_down(options),
    }

    println!("Done!");
}

fn sync_up(options: SyncUpOptions) {
    let op_signed_in = op_whoami();

    if !op_signed_in {
        println!("You are not logged in to 1password CLI. Proceeding to log in...");

        op_sign_in();
    }

    let env_file_path = options.env;

    let env_file_contents = match read_env_file(&env_file_path) {
        Ok(contents) => contents,
        Err(_) => {
            println!("Failed to read environment file at {}", &env_file_path);
            process::exit(1);
        }
    };

    let env_vars = parse_env_file(&env_file_contents);

    let vaults = op_get_vaults().expect("Failed to get vaults.");

    let selected_vault =
        ask_select_item("Select vault: ", vaults).expect("Failed to select vault.");

    let mut items = op_get_items(&selected_vault).expect("Failed to get items.");

    // add a "Create new" item to the list of items
    items.push(OPItem {
        title: String::from("(Create new)"),
        id: String::from("create-new"),
    });

    // get item details, or create new item if user has chosen that
    let mut item_details = match ask_select_item("Select item, or create new: ", items)
        .expect("Failed to select item.")
    {
        OPItem { id, .. } if id == String::from("create-new") => {
            let new_item_title = ask_create_item("Enter a name: ");

            op_create_item(&selected_vault.name, new_item_title.as_str())
                .expect("Failed to create item.")
        }
        item => op_get_item(item.id.as_str()).expect("Failed to create item."),
    };

    // get item sections with a label (or an empty Vec)
    let mut item_sections: Vec<OPSection> = item_details
        .sections
        .unwrap_or(Vec::new())
        .iter()
        .cloned()
        .filter(|section| section.label.is_some())
        .collect();

    // add "Create new" and "None" sections to the list of sections
    item_sections.push(OPSection {
        label: Some(String::from("(Create new)")),
        id: String::from("create-new"),
    });

    item_sections.push(OPSection {
        label: Some(String::from("(None)")),
        id: String::from("none"),
    });

    // get selected section, or create new one
    let selected_section = match ask_select_item(
        "Select environment (e.g staging / production), or create new: ",
        item_sections,
    )
    .expect("Failed to select section.")
    {
        OPSection { id, .. } if id == String::from("create-new") => {
            let new_section_label = ask_create_item("Enter a name: ");

            Some(OPSection {
                label: Some(new_section_label.clone()),
                id: new_section_label.clone(),
            })
        }
        OPSection { id, .. } if id == String::from("none") => None,
        section => Some(section),
    };

    // get fields from 1password item and convert to EnvVariables
    let item_vars: Vec<EnvVariable> = item_details
        .fields
        .iter()
        .filter(|field| op_field_in_section(field, &selected_section))
        .filter_map(op_field_to_env_var)
        .collect();

    let unsynced_env_vars: Vec<&EnvVariable> = env_vars
        .iter()
        .filter(|env| {
            item_vars.iter().all(|field_env| {
                // check if variable is not synced
                (field_env.key == env.key && field_env.value != env.value)
                    || field_env.key != env.key
            })
        })
        .collect();

    let env_vars_to_sync =
        match ask_select_items("Which variables do you want to sync?", unsynced_env_vars) {
            Ok(env_vars) => env_vars,
            Err(_) => {
                println!("No new variables to upload!");
                Vec::new()
            }
        };

    let confirmation_string = env_vars_to_sync.iter().fold(String::from(""), |acc, env| {
        format!("{}{} -> {}\n", acc, env.key, env.value)
    });

    // update fields in 1password
    if env_vars_to_sync.len() > 0
        && ask_proceed(
            format!(
                "Are you sure you want to sync these variables? \n{}",
                confirmation_string
            )
            .as_str(),
            false,
        )
    {
        let field_edit_command: Vec<String> = env_vars_to_sync
            .iter()
            .map(|env| match selected_section.clone() {
                Some(selected_section) => {
                    format!("{}.{}[text]={}", selected_section.id, env.key, env.value)
                }
                None => format!("{}[text]={}", env.key, env.value),
            })
            .collect();

        match op_edit(item_details.id.as_str(), field_edit_command) {
            Ok(result) => {
                item_details = result;
                println!("Synced variables successfully!");
            }
            Err(_) => {
                println!("Failed to sync variables!");
            }
        }
    }

    let provision_file_path = match selected_section.clone() {
        Some(OPSection {
            label: Some(label), ..
        }) => format!(".env.provision.{}", label),
        _ => String::from(".env.provision"),
    };

    // write to provision file
    if ask_proceed(
        format!("Do you want to write to {}?", &provision_file_path).as_str(),
        true,
    ) {
        let mut item_provision_vars: Vec<EnvVariable> = item_details
            .fields
            .iter()
            .filter(|field| op_field_in_section(field, &selected_section))
            .filter_map(op_field_to_env_var_reference)
            .collect();

        if path::Path::new(&provision_file_path).is_file() {
            let provision_file_contents =
                fs::read_to_string(&provision_file_path).expect("Failed to read provision file.");

            let provision_vars = parse_env_file(&provision_file_contents);

            item_provision_vars = item_provision_vars
                .into_iter()
                .filter(|item_var| {
                    provision_vars
                        .iter()
                        .all(|provision_var| provision_var.key != item_var.key)
                })
                .collect();
        } else {
            println!("Didn't find provision file, creating a new one!");
            fs::File::create(&provision_file_path)
                .expect("Failed to create provision file.")
                .write_all("# This provision file was auto-generated by envop. You can remove this comment and modify the file as you like.\n# Only missing variables will be appended when generating again.\n".as_bytes()).expect("Failed to write to provision file.");
        }

        let string_to_write = item_provision_vars
            .iter()
            .fold(String::from(""), |acc, env| {
                format!("{}\n{}={}", acc, env.key, env.value)
            });

        write_to_file(&provision_file_path, &string_to_write)
            .expect("Failed to write to provision file.");
    }
}

fn sync_down(options: SyncDownOptions) {
    let env_file_path = options.env;

    // find all provision files
    let provision_files: Vec<String> = glob("./.env.provision*")
        .expect("Failed to read glob pattern")
        .filter_map(|glob_result| match glob_result {
            Ok(result) => Some(result.into_os_string().into_string().ok()?),
            Err(_) => None,
        })
        .collect();

    let selected_provision_file =
        ask_select_item("Which provision file do you want to use?", provision_files)
            .expect("Failed to select provision file.");

    match op_inject(&selected_provision_file, &env_file_path) {
        Ok(_) => {
            println!("Successfully synced to {} !", &env_file_path);
        }
        Err(err) => {
            println!("Error syncing variables: {}", err);
        }
    }
}
