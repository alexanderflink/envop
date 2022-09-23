use crate::EnvVariable;
use core::fmt;
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::io;
use std::process;
use std::process::{Command, Output};
use std::string::FromUtf8Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OPVault {
    pub id: String,
    pub name: String,
}

impl fmt::Display for OPVault {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.name)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OPItem {
    pub id: String,
    pub title: String,
}

impl fmt::Display for OPItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.title)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OPSection {
    pub id: String,
    pub label: Option<String>,
}

impl fmt::Display for OPSection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.label.clone() {
            Some(label) => {
                write!(f, "{}", label)
            }
            None => {
                write!(f, "No label, id: {}", &self.id)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OPField {
    pub id: String,
    pub label: Option<String>,
    #[serde(rename = "type")]
    pub _type: String,
    pub purpose: Option<String>,
    pub reference: String,
    pub section: Option<OPSection>,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OPItemDetails {
    pub id: String,
    pub title: String,
    pub version: Option<u16>,
    pub vault: OPVault,
    pub category: String,
    pub last_edited_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub sections: Option<Vec<OPSection>>,
    pub fields: Vec<OPField>,
}

#[derive(Debug)]
pub enum ParseOPError {
    IOError(io::Error),
    SerializeError(serde_json::Error),
    FromUtf8Error(FromUtf8Error),
}

impl From<io::Error> for ParseOPError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<FromUtf8Error> for ParseOPError {
    fn from(e: FromUtf8Error) -> Self {
        Self::FromUtf8Error(e)
    }
}

impl From<serde_json::Error> for ParseOPError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerializeError(e)
    }
}

pub fn op_edit(item_id: &str, edits: Vec<String>) -> Result<OPItemDetails, ParseOPError> {
    let mut args = vec!["item", "edit", item_id, "--format=json"];

    let edits: Vec<&str> = edits.iter().map(|s| s.as_str()).collect();

    args.extend(edits);

    let output = Command::new("op").args(args).output()?;

    let output_string = String::from_utf8(output.stdout)?;

    let item_details: OPItemDetails = serde_json::from_str(&output_string)?;

    Ok(item_details)
}

fn op_parse_list<T: de::DeserializeOwned>(command: &mut Command) -> Result<Vec<T>, ParseOPError> {
    let output = command.output()?;
    let output_string = String::from_utf8(output.stdout)?;

    let list_items: Vec<T> = serde_json::from_str(&output_string)?;

    Ok(list_items)
}

pub fn op_get_vaults() -> Result<Vec<OPVault>, ParseOPError> {
    op_parse_list(Command::new("op").args(["vault", "list", "--format=json"]))
}

pub fn op_get_items(vault: &OPVault) -> Result<Vec<OPItem>, ParseOPError> {
    let mut vault_parameter = String::from("--vault=");
    vault_parameter.push_str(&vault.name);

    op_parse_list(Command::new("op").args(["item", "list", &vault_parameter, "--format=json"]))
}

pub fn op_get_item(id: &str) -> Result<OPItemDetails, ParseOPError> {
    let output = Command::new("op")
        .args(["item", "get", id, "--format=json"])
        .output()?;

    let output_string = String::from_utf8(output.stdout)?;

    let item_details: OPItemDetails = serde_json::from_str(&output_string)?;

    Ok(item_details)
}

pub fn op_create_item(vault: &str, title: &str) -> Result<OPItemDetails, ParseOPError> {
    let output = Command::new("op")
        .args([
            "item",
            "create",
            &format!("--title={}", title),
            &format!("--vault={}", vault),
            "--category=Secure Note",
            "--format=json",
        ])
        .output()?;

    let output_string = String::from_utf8(output.stdout)?;

    let item_details: OPItemDetails = serde_json::from_str(&output_string)?;

    Ok(item_details)
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

pub fn op_field_in_section(field: &OPField, section: &Option<OPSection>) -> bool {
    match (&field.section, section) {
        (Some(field_section), Some(section)) if field_section.label == section.label => true,
        (Some(OPSection { label: None, .. }) | None, None) => true,
        (_, _) => false,
    }
}

pub fn op_field_to_env_var_reference(field: &OPField) -> Option<EnvVariable> {
    match field {
        OPField {
            label: Some(label), ..
        } => Some(EnvVariable {
            key: label.to_string(),
            value: String::from(&field.reference),
        }),
        _ => None,
    }
}

pub fn op_field_to_env_var(field: &OPField) -> Option<EnvVariable> {
    match field {
        OPField {
            label: Some(label),
            value: Some(value),
            ..
        } => Some(EnvVariable {
            key: label.to_string(),
            value: value.to_string(),
        }),
        _ => None,
    }
}

pub fn op_inject(env_file_path: &str, provision_file_path: &str) -> io::Result<Output> {
    match Command::new("op")
        .args([
            "inject",
            format!("--in-file={}", env_file_path).as_str(),
            format!("--out-file={}", provision_file_path).as_str(),
            "--force",
        ])
        .output()
    {
        Ok(output @ Output { status, .. }) if status.success() => Ok(output),
        Ok(output @ Output { status, .. }) if !status.success() => Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8(output.stderr).expect(""),
        )),
        Err(err) => Err(err),
        output => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to inject because of an unknown error: {:?}", output),
        )),
    }
}
