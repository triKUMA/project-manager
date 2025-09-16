use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::util::yaml::*;

// TODO:
// - need to auto capture workspaces from root directory
// - need to build reserved variables
//   - $env built from current environment variables
//   - $opts built from command line arguments

pub fn normalize_yaml(yaml: &mut Mapping) -> Result<()> {
    normalize_mapping(yaml, |key, value| {
        if let Some(value_mapping) = value.as_mapping_mut() {
            match key {
                "workspaces" => normalize_workspaces(value_mapping)?,
                "state" => normalize_state(value_mapping)?,
                "commands" => normalize_commands(value_mapping)?,
                "runners" => normalize_runners(value_mapping)?,
                _ => (),
            };
        }

        Ok(())
    })?;

    Ok(())
}

pub fn normalize_workspaces(yaml: &mut Mapping) -> Result<()> {
    Ok(())
}

pub fn normalize_state(yaml: &mut Mapping) -> Result<()> {
    Ok(())
}

pub fn normalize_commands(yaml: &mut Mapping) -> Result<()> {
    // if mapping
    // - if has commands key, process sub commands
    // if key starts with '!'
    // - update scope to include 'in: {scope_name}'
    // DONT expand glob patterns in this step - should be done in canonizing step when scopes have been extracted
    // ensure implicit commands map doesn't contain any reserved keywords as keys before normalizing
    // reserved keywords: run, commands, in, pre, post, background
    // ensure variables are not counted in implicit commands map - should require explicit commands map if using variables
    // process leaf commands to expand into anonymous/unnamed tasks

    normalize_mapping(yaml, |key, value| {
        if let Some(value_mapping) = value.as_mapping_mut() {
            if value_mapping.contains_key("run") {
                if !value_mapping.contains_key("commands") {
                    value_mapping.insert(
                        Value::String("commands".to_string()),
                        Value::Mapping(Mapping::new()),
                    );
                }

                let run = value_mapping.remove("run").unwrap();

                let scope_commands_mapping = value_mapping["commands"].as_mapping_mut().unwrap();

                if scope_commands_mapping.contains_key(".") {
                    return Err(eyre!("duplicate root command in yaml: {}", key));
                }

                scope_commands_mapping.insert(Value::String(".".to_string()), run);
            } else if !value_mapping.contains_key("commands") {
                if value_mapping.iter().all(|(k, v)| v.is_string()) {
                    let keys = value_mapping.keys().cloned().collect::<Vec<_>>();
                    let mut commands_mapping = Mapping::new();
                    commands_mapping.extend(keys.into_iter().map(|k| {
                        (
                            k.clone(),
                            value_mapping.remove(k.as_str().unwrap()).unwrap(),
                        )
                    }));
                    value_mapping.insert(
                        Value::String("commands".to_string()),
                        Value::Mapping(commands_mapping),
                    );
                } else {
                    normalize_commands(value_mapping)?;
                }
            }
        }

        Ok(())
    })?;

    Ok(())
}

pub fn normalize_runners(yaml: &mut Mapping) -> Result<()> {
    Ok(())
}

pub fn noramlize_key(key: &str) -> Result<(&str, Mapping)> {
    let implicit_scope = key.starts_with('!');

    let key_parts = key
        .strip_prefix('!')
        .unwrap_or(key)
        .split('?')
        .collect::<Vec<_>>();

    if key_parts.len() == 1 && !implicit_scope {
        return Ok((key_parts[0], Mapping::new()));
    }

    if key_parts.len() > 2 {
        return Err(eyre!(
            "invalid property shorthand syntax: {}",
            key_parts[1..].join("?")
        ));
    }

    let mut query = Mapping::new();

    // Only parse query string if there are query parameters
    if key_parts.len() == 2 {
        // TODO: handle the unwraps used here - should return readable errors instead
        query = serde_qs::from_str(key_parts[1])?;

        query.iter_mut().for_each(|(_, v)| {
            if let Some(str_val) = v.as_str() {
                if str_val.is_empty() {
                    *v = Value::Bool(true);
                } else {
                    *v = serde_yaml::from_str::<Value>(str_val).unwrap();
                }
            } else if let Some(sequence) = v.as_sequence_mut() {
                parse_unserialized_sequence(sequence).unwrap();
            }
        });
    }

    if implicit_scope && !query.contains_key("in") {
        query.insert(
            Value::String("in".to_string()),
            Value::String(key_parts[0].to_string()),
        );
    }

    Ok((key_parts[0], query))
}

pub fn normalize_mapping<F>(yaml: &mut Mapping, mut value_processor: F) -> Result<&mut Mapping>
where
    F: FnMut(&str, &mut Value) -> Result<()>,
{
    let keys: Vec<Value> = yaml.keys().cloned().collect();
    for key in keys {
        if !key.is_string() {
            return Err(eyre!(
                "invalid root key in yaml: {:#?}\nroot key must be a string",
                key
            ));
        }

        let key = key.as_str().unwrap();
        let (base_key, shorthand_props) = noramlize_key(key)?;

        if let Some(mut value) = yaml.remove(key) {
            // Process the value using the provided closure
            value_processor(base_key, &mut value)?;

            if let Some(value_mapping) = value.as_mapping_mut() {
                soft_merge_mappings(value_mapping, &shorthand_props);
            }

            yaml.insert(Value::String(base_key.to_string()), value);
        }
    }

    Ok(yaml)
}
