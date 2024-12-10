use std::process::Command;
use serde_json::json;
use shell_words;

const COMMAND_DELIMITER: &str = ",";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Error: No commands provided.");
        return;
    }

    let mut messages = Vec::new();

    // Split arguments into separate commands by a delimiter (e.g., `,`)
    let joined_args = args.join(" ");
    let commands: Vec<String> = joined_args.split(COMMAND_DELIMITER).map(|cmd| cmd.trim().to_string()).filter(|cmd| !cmd.is_empty()).collect();

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

    match serde_json::to_string_pretty(&json!(messages)) {
        Ok(chat_context) => println!("{}", chat_context),
        Err(e) => eprintln!("Error serializing JSON: {}", e),
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
