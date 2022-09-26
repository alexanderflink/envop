# envop
This is a CLI for syncing environment variables using 1password and provisioning files. Each environment (e.g staging or production) is stored in a separate section of an item in the 1password vault. Uses .provision files as templates for the environemnt files.

## Usage
### Uploading variables to 1password
1. Fill out your environment file with your secrets (envop looks for .env as default).
2. Run `envop up`, if your environment file is named something other than .env, you can pass it as an argument: `--env FILE_PATH`
3. Select appropriate 1password vault, item and section (or use the prompt to create a new item and section)
4. Select which variables you want to upload. Only new or updated variables will show up.
5. You will be asked if you also want to write to a provision file. The provision file will be named whatever the name of the section you chose was.

### Downloading variables from 1password
1. Make sure you have at least one .provision file
2. Run `envop down`, if your environment file is named something other than .env, you can pass it as an argument: `--env FILE_PATH`
3. Select the provision file you'd like to use.

## Requirements
- [1password CLI](https://1password.com/downloads/command-line/)

## Installation
### Using homebrew
`brew install alexanderflink/envop/envop`

### Using npm
`npm install envop --save-dev`

### Using cargo
`cargo install envop`

### Manual installation
Download the [latest release](https://github.com/alexanderflink/envop/releases), unpack the appropriate binary for your system and point to it in your $PATH.