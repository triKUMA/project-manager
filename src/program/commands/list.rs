use color_eyre::eyre::Result;
use serde_yaml::{Mapping, Value};

use crate::config::constants;

pub fn list_commands(config: &Mapping) -> Result<()> {
    // TODO: this should be replaced with the config item in the global config file later on
    let scope_separator = "::";

    let mut commands = Vec::new();

    let mut scopes_to_process: Vec<(String, &Mapping)> = Vec::new();

    if let Some(Value::Mapping(root_scope)) = config.get("commands") {
        scopes_to_process.push(("root".to_string(), root_scope));
    }

    while let Some(scope) = scopes_to_process.pop() {
        if let Some(Value::Mapping(commands_mapping)) = scope.1.get("commands") {
            for key in commands_mapping.keys() {
                let key = key.as_str().unwrap();
                commands.push(if scope.0 == "root" {
                    key.to_string()
                } else if key == "." {
                    scope.0.clone()
                } else {
                    format!("{}{scope_separator}{}", scope.0, key)
                });
            }
        }

        for key in scope.1.keys() {
            let key = key.as_str().unwrap();

            if constants::SCOPE_RESERVED_KEYS.contains(&key) {
                continue;
            }

            let value = scope.1.get(key).unwrap();
            if value.is_mapping() {
                scopes_to_process.push((
                    if scope.0 == "root" {
                        key.to_string()
                    } else {
                        format!("{}{scope_separator}{}", scope.0, key)
                    },
                    value.as_mapping().unwrap(),
                ));
            }
        }
    }

    commands.sort_unstable();

    println!("\nAvailable commands:");
    for command in commands {
        println!("- {}", command);
    }

    Ok(())
}
