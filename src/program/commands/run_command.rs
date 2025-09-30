use color_eyre::eyre::{Result, eyre};
use serde_yaml::Mapping;

use crate::config::constants;

pub fn run(command: &str, initial_scope: &Mapping, config: &Mapping) -> Result<()> {
    let command_parts = command.split(constants::SCOPE_SEPARATOR);

    if !config.contains_key("commands") {
        return Err(eyre!(
            "unable to run command, no commands defined in config"
        ));
    }

    let commands = config.get("commands").unwrap().as_mapping().unwrap();
    let scope = initial_scope.clone();

    for (i, cmd_curr_scope) in command_parts.enumerate() {}

    // loop through command parts
    // - build up current scope as we go
    // - try to run command if we have reached the end of the command parts

    Ok(())
}
