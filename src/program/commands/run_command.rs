use color_eyre::eyre::{Result, eyre};
use serde_yaml::Mapping;

use crate::{config::constants::{self, SCOPE_RESERVED_KEYS}, models};

pub fn run(command: &str, mut command_scope: models::command::CommandScope, config: &Mapping) -> Result<()> {
	let mut command_parts = command.split(constants::SCOPE_SEPARATOR).peekable();

	if !config.contains_key("commands") {
		return Err(eyre!(
			"unable to run command, no commands defined in config"
		));
	}

	let commands = config.get("commands").unwrap().as_mapping().unwrap();

	let (mut prev_path, mut prev_scope) = (String::new(), commands);
	let mut last_scope_name = "";
	while let Some(curr_scope_name) = command_parts.next() {
		if !prev_scope.contains_key(curr_scope_name) {
			if command_parts.peek().is_none() {
				last_scope_name = curr_scope_name;
				continue;
			}
			
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

		command_scope.accumulate_from_mapping(curr_scope);

		last_scope_name = curr_scope_name;
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
	}

	if let Some(commands_mapping) = prev_scope.get("commands") {
		let command_name = if commands_mapping.as_mapping().unwrap().contains_key(".") {
			"."
		} else {
			last_scope_name
		};
		
		let mut filtered_commands_mapping = commands_mapping.as_mapping().unwrap().clone();
		let commands_keys = filtered_commands_mapping.keys().cloned().collect::<Vec<_>>();
		for k in commands_keys {
			let k_str = k.as_str().unwrap();
			if !SCOPE_RESERVED_KEYS.contains(&k_str) && k_str != command_name {
				filtered_commands_mapping.remove(k_str);
			}
		}
		
		command_scope.accumulate_from_mapping(&filtered_commands_mapping);
		
		println!("would execute command scope: {command_scope:#?}");
	} else {
		return Err(eyre!(
			"unable to find command '{last_scope_name}' in config{}", if prev_path.is_empty() {
				String::new()
			} else {
				format!(". could not find '{last_scope_name}' in '{prev_path}'")
			},
		));
	}

	Ok(())
}
