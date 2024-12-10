use std::process::Command;
use serde_json::json;
use shell_words;
use clap::{Command as ClapCommand, Arg, ArgAction};
use serde_yaml;

const COMMAND_DELIMITER: &str = ",";

fn main() {
    let matches = ClapCommand::new("Command Executor")
        .version("1.0")
        .author("Divan")
        .about("Executes commands and formats output")
        .arg(
            Arg::new("commands")
                .help("Comma-separated commands to execute")
                .required(true),
        )
        .arg(
            Arg::new("yaml")
                .long("yaml")
                .help("Output results in YAML format")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let raw_commands = matches.get_one::<String>("commands").expect("Commands required");
    let use_yaml = matches.get_flag("yaml");

    let commands: Vec<String> = raw_commands
        .split(COMMAND_DELIMITER)
        .map(|cmd| cmd.trim().to_string())
        .filter(|cmd| !cmd.is_empty())
        .collect();

    if commands.is_empty() {
        eprintln!("Error: No commands provided.");
        return;
    }

    let mut messages = Vec::new();

    for command in commands {
        match execute_command(&command) {
            Ok(output) => {
                let message = json!({
                    "role": "system",
                    "content": format!(
                        "Command executed: '{}'.\nOutput:\n{}",
                        command, output.trim()
                    )
                });
                messages.push(message);
            }
            Err(e) => {
                let error_message = json!({
                    "role": "system",
                    "content": format!(
                        "Command execution failed: '{}'.\nError: {}",
                        command, e
                    )
                });
                messages.push(error_message);
            }
        }
    }

    if use_yaml {
        match serde_yaml::to_string(&messages) {
            Ok(yaml_output) => println!("{}", yaml_output),
            Err(e) => eprintln!("Error serializing YAML: {}", e),
        }
    } else {
        match serde_json::to_string_pretty(&json!(messages)) {
            Ok(chat_context) => println!("{}", chat_context),
            Err(e) => eprintln!("Error serializing JSON: {}", e),
        }
    }
}

fn execute_command(command: &str) -> Result<String, String> {
    let parts: Vec<String> = shell_words::split(command).map_err(|e| e.to_string())?;
    if parts.is_empty() {
        return Err("Command is empty".into());
    }

    let cmd = &parts[0];
    let args = &parts[1..];
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into())
    }
}
