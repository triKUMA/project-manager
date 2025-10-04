use color_eyre::eyre::{Result, eyre};
use serde_yaml::Mapping;

use crate::config::constants;

pub fn run(command: &str, initial_scope: &Mapping, config: &Mapping) -> Result<()> {
    let mut command_parts = command.split(constants::SCOPE_SEPARATOR).peekable();

    if !config.contains_key("commands") {
        return Err(eyre!(
            "unable to run command, no commands defined in config"
        ));
    }

    let commands = config.get("commands").unwrap().as_mapping().unwrap();
    let command_scope = initial_scope.clone();

    let (mut prev_path, mut prev_scope) = (String::new(), commands);
    while let Some(curr_scope_name) = command_parts.next() {
        if command_parts.peek().is_some() {
            // not at end of command parts yet, just accumulate scope
            if !prev_scope.contains_key(curr_scope_name) {
                return Err(eyre!(
                    "unable to find command scope '{curr_scope_name}' in config{}",
                    if prev_path.is_empty() {
                        String::new()
                    } else {
                        format!(". scope not found in '{prev_path}'",)
                    }
                ));
            }

            let curr_scope = prev_scope
                .get(curr_scope_name)
                .unwrap()
                .as_mapping()
                .unwrap();

            // add current scope to command scope, overriding any existing keys - need to exclude certain keys like "commands"

            (prev_path, prev_scope) = (
                format!(
                    "{}{}{}",
                    prev_path,
                    if prev_path.is_empty() {
                        ""
                    } else {
                        constants::SCOPE_SEPARATOR
                    },
                    curr_scope_name
                ),
                curr_scope,
            );
        } else {
            // at end of command parts, this is the command to try and execute
            if let Some(Some(task_collection)) =
                prev_scope.get("commands").map(|v| v.get(curr_scope_name))
            {
                let command_to_run = task_collection.get("tasks").unwrap();
                println!("would run command: {command_to_run:?} with scope: {command_scope:?}");
            } else if let Some(Some(curr_scope)) =
                prev_scope.get(curr_scope_name).map(|v| v.as_mapping())
                && let Some(Some(task_collection)) = curr_scope.get("commands").map(|v| v.get("."))
            {
                // add current scope to command scope, overriding any existing keys - need to exclude certain keys like "commands"

                let command_to_run = task_collection.get("tasks").unwrap();
                println!("would run command: {command_to_run:?} with scope: {command_scope:?}");
            } else {
                return Err(eyre!(
                    "unable to find command '{curr_scope_name}' in config. could not find '{curr_scope_name}' in '{prev_path}'",
                ));
            }
        }
    }

    // loop through command parts
    // - build up current scope as we go
    // - try to run command if we have reached the end of the command parts

    Ok(())
}
