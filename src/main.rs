mod op_utils;
mod utils;
use inquire;
use op_utils::{
    op_create_item, op_edit, op_get_item, op_get_items, op_get_sections, op_get_vaults, op_read,
    op_sign_in, op_whoami, OPField, OPItem, OPSection,
};
use std::fs;
use std::process;
use utils::{
    ask_create_item, ask_proceed, ask_select_item, compare_env_vars, get_argument_or_default,
    parse_env_file, read_env_file, write_to_file, EnvVariable, EnvVariables,
};

fn main() {
    let env_file_path = get_argument_or_default(1, ".env");
    let provision_file_path = get_argument_or_default(2, ".env.provision");
    let op_signed_in = op_whoami();

    if !op_signed_in {
        println!("You are not logged in to 1password CLI. Proceeding to log in...");

        op_sign_in();
    }

    let env_file_contents = read_env_file(&env_file_path);
    let provision_file_contents = read_env_file(&provision_file_path);

    let env_vars = parse_env_file(&env_file_contents);
    let provision_vars = parse_env_file(&provision_file_contents);

    let vaults = op_get_vaults();

    let selected_vault = ask_select_item("Select vault: ", vaults);

    let mut items = op_get_items(&selected_vault);

    items.push(OPItem {
        title: String::from("Create new"),
        id: String::from("create-new"),
    });

    let selected_item = ask_select_item("Select file: ", items);

    let item_details = match selected_item.id.as_str() {
        "create-new" => {
            let new_item_title = ask_create_item("Enter a name: ");

            op_create_item(&selected_vault.name, new_item_title.as_str())
        }
        _ => op_get_item(&selected_item.id),
    };

    let mut item_sections = item_details.sections.unwrap_or(Vec::new());

    item_sections.push(OPSection {
        label: Some(String::from("Create new")),
        id: String::from("create-new"),
    });

    let item_fields = item_details.fields;

    let mut selected_section = ask_select_item("Select environment (section): ", item_sections);

    if selected_section.id == "create-new" {
        let new_section_label = ask_create_item("Enter a name: ");

        selected_section = OPSection {
            label: Some(new_section_label.clone()),
            id: new_section_label.clone(),
        };
    }

    let unsynced_env_vars: Vec<&EnvVariable> = env_vars
        .iter()
        .filter(|env| {
            item_fields
                .iter()
                .filter(|field| {
                    let field_section_label = match field.section.clone() {
                        Some(section) => section.label.unwrap_or(String::from("")),
                        None => String::from(""),
                    };

                    field_section_label
                        == selected_section.label.clone().unwrap_or(String::from(""))
                })
                .all(|field| {
                    let field_label = field.label.clone().unwrap_or(String::from(""));
                    let field_value = field.value.clone().unwrap_or(String::from(""));

                    // check if variable is not synced
                    (field_label == env.key && field_value != env.value) || field_label != env.key
                })
        })
        .collect();

    println!(
        "Found {} unsynced variables: {:?}",
        unsynced_env_vars.len(),
        unsynced_env_vars
    );

    let mut updated_env_vars: Vec<&EnvVariable> = Vec::new();

    unsynced_env_vars.iter().for_each(|env| {
        if !ask_proceed(format!("Do you want to sync {} ?", env.key), true) {
            println!("Skipping {}", env.key);

            return;
        }

        if op_edit(
            item_details.id.as_str(),
            format!("{}.{}[text]={}", selected_section.id, env.key, env.value),
        )
        .success()
        {
            println!("Synced {} successfully!", env.key);
            updated_env_vars.push(env);
        } else {
            println!("Failed to sync {}!", env.key);
        }
    });

    /*
        PROVISION FILE
    */

    if ask_proceed(
        String::from(
            "Do you want to sync variables from vault to provision file (.env.provision)?",
        ),
        false,
    ) {
        let use_env_for_section = ask_proceed(
            String::from("Use environment variable for section ($OP_ENV)?"),
            true,
        );

        let item_details = op_get_item(item_details.id.as_str());

        let item_fields = item_details.fields;

        let unsynced_fields: Vec<&OPField> = item_fields
            .iter()
            .filter(|field| match field.section.clone() {
                Some(field_section) => {
                    field_section.label == selected_section.label
                        && provision_vars.iter().all(|provision_var| {
                            provision_var.key != field.label.clone().unwrap_or(String::from(""))
                        })
                }
                None => false,
            })
            .collect();

        unsynced_fields.iter().for_each(|field| {
            let field_label = field.clone().label.clone();

            let selected_section_label = if use_env_for_section {
                Some(String::from("$OP_ENV"))
            } else {
                selected_section.label.clone()
            };

            match (field_label, selected_section_label) {
                (Some(field_label), Some(selected_section_label)) => {
                    write_to_file(
                        &provision_file_path,
                        format!(
                            "{}=op://{}/{}/{}/{}\n",
                            field_label,
                            selected_vault.name,
                            item_details.title,
                            selected_section_label,
                            field_label
                        ),
                    )
                    .expect("Failed to write to provision file.");
                }
                (Some(field_label), None) => {
                    write_to_file(
                        &provision_file_path,
                        format!(
                            "{}=op://{}/{}/{}/{}\n",
                            field_label,
                            selected_vault.name,
                            item_details.title,
                            selected_section.id,
                            field_label
                        ),
                    )
                    .expect("Failed to write to provision file.");
                }
                _ => (),
            }
        });
    }

    println!("Done!");
}
