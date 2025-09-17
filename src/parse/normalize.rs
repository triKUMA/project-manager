use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::util::yaml::*;

const SCOPE_RESERVED_KEYS: [&str; 6] = ["in", "variables", "pre", "post", "run", "commands"];

// TODO:
// - need to auto capture workspaces from root directory
// - need to build reserved variables
//   - $env built from current environment variables
//   - $opts built from command line arguments

pub fn normalize_yaml<'a>(yaml: &'a mut Mapping) -> Result<&'a mut Mapping> {
    normalize_mapping("", yaml, |key: &str, value: &mut Value| {
        if let Some(value_mapping) = value.as_mapping_mut() {
            match key {
                "workspaces" => {
                    normalize_workspaces("workspaces", value_mapping)?;
                }
                "state" => {
                    normalize_state("state", value_mapping)?;
                }
                "commands" => {
                    normalize_scope("commands", value_mapping)?;
                }
                "runners" => {
                    normalize_runners("runners", value_mapping)?;
                }
                _ => (),
            };
        }

        Ok(())
    })?;

    Ok(yaml)
}

pub fn normalize_workspaces<'a>(path: &str, yaml: &'a mut Mapping) -> Result<&'a mut Mapping> {
    Ok(yaml)
}

pub fn normalize_state<'a>(path: &str, yaml: &'a mut Mapping) -> Result<&'a mut Mapping> {
    Ok(yaml)
}

pub fn normalize_scope<'a>(path: &str, yaml: &'a mut Mapping) -> Result<&'a mut Mapping> {
    /*
    scope can contain:
    - sub scopes (determined if all scope mapping key values are mappings)
    - pre, post, run, commands, variables (determined if all scope mapping keys are in SCOPE_RESERVED_KEYS)
    - implicit commands map (determined if all scope mapping key values are strings)
     */

    // extract any keys in scope that are variables
    let variable_keys = yaml
        .keys()
        .cloned()
        .filter_map(|k| {
            if let Some(k) = k.as_str() {
                k.strip_prefix('$').map(|k| k.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // process variables and normalize
    if !variable_keys.is_empty() {
        if !yaml.contains_key("variables") {
            yaml.insert(
                Value::String("variables".to_string()),
                Value::Mapping(Mapping::new()),
            );
        }

        for var_key in variable_keys {
            let var_key_value = yaml.remove(format!("${}", var_key)).unwrap();
            yaml["variables"]
                .as_mapping_mut()
                .unwrap()
                .insert(Value::String(var_key), var_key_value);
        }
    }

    // get list of implicit command keys
    let implicit_command_keys = yaml
        .keys()
        .cloned()
        .filter_map(|k| {
            if let Some(k) = k.as_str()
                && !SCOPE_RESERVED_KEYS.contains(&k)
            {
                Some(k.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // add commands key if it doesnt exist yet but will be needed
    if (yaml.contains_key("run") || !implicit_command_keys.is_empty())
        && !yaml.contains_key("commands")
    {
        yaml.insert(
            Value::String("commands".to_string()),
            Value::Mapping(Mapping::new()),
        );
    }

    if let Some(run_val) = yaml.remove("run") {
        yaml["commands"]
            .as_mapping_mut()
            .unwrap()
            .insert(Value::String(".".to_string()), run_val);
    }

    for implicit_command_key in implicit_command_keys {
        let implicit_command_key_value = yaml.remove(implicit_command_key.clone()).unwrap();
        yaml["commands"].as_mapping_mut().unwrap().insert(
            Value::String(implicit_command_key),
            implicit_command_key_value,
        );
    }

    normalize_mapping(path, yaml, |key, value| match key {
        "in" => Ok(()),
        "pre" => Ok(()),
        "post" => Ok(()),
        "commands" => Ok(()),
        _ if SCOPE_RESERVED_KEYS.contains(&key) => Ok(()),
        _ if value.is_mapping() => {
            normalize_scope(
                format!("{}.{}", path, key).as_str(),
                value.as_mapping_mut().unwrap(),
            )?;

            Ok(())
        }
        _ => Ok(()),
    })?;

    Ok(yaml)
}

pub fn normalize_runners<'a>(path: &str, yaml: &'a mut Mapping) -> Result<&'a mut Mapping> {
    Ok(yaml)
}

// pub fn normalize_commands(yaml: &mut Mapping) -> Result<()> {
//     // if mapping
//     // - if has commands key, process sub commands
//     // if key starts with '!'
//     // - update scope to include 'in: {scope_name}'
//     // DONT expand glob patterns in this step - should be done in canonizing step when scopes have been extracted
//     // ensure implicit commands map doesn't contain any reserved keywords as keys before normalizing
//     // reserved keywords: run, commands, in, pre, post, background
//     // ensure variables are not counted in implicit commands map - should require explicit commands map if using variables
//     // process leaf commands to expand into anonymous/unnamed tasks

//     normalize_mapping(yaml, |key, value| {
//         if let Some(value_mapping) = value.as_mapping_mut() {
//             if value_mapping.contains_key("run") {
//                 if !value_mapping.contains_key("commands") {
//                     value_mapping.insert(
//                         Value::String("commands".to_string()),
//                         Value::Mapping(Mapping::new()),
//                     );
//                 }

//                 let run = value_mapping.remove("run").unwrap();

//                 let scope_commands_mapping = value_mapping["commands"].as_mapping_mut().unwrap();

//                 if scope_commands_mapping.contains_key(".") {
//                     return Err(eyre!("duplicate root command in yaml: {}", key));
//                 }

//                 scope_commands_mapping.insert(Value::String(".".to_string()), run);
//             } else if !value_mapping.contains_key("commands") {
//                 if value_mapping.iter().all(|(k, v)| v.is_string()) {
//                     let keys = value_mapping.keys().cloned().collect::<Vec<_>>();
//                     let mut commands_mapping = Mapping::new();
//                     commands_mapping.extend(keys.into_iter().map(|k| {
//                         (
//                             k.clone(),
//                             value_mapping.remove(k.as_str().unwrap()).unwrap(),
//                         )
//                     }));
//                     value_mapping.insert(
//                         Value::String("commands".to_string()),
//                         Value::Mapping(commands_mapping),
//                     );
//                 } else {
//                     normalize_commands(value_mapping)?;
//                 }
//             }
//         }

//         Ok(())
//     })?;

//     Ok(())
// }

pub fn normalize_key<'a>(path: &str, key: &'a str) -> Result<(&'a str, Mapping)> {
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

        for (_, v) in query.iter_mut() {
            if let Some(str_val) = v.as_str() {
                if str_val.is_empty() {
                    *v = Value::Bool(true);
                } else {
                    *v = serde_yaml::from_str::<Value>(str_val).map_err(|e| eyre!(e))?;
                }
            } else if let Some(sequence) = v.as_sequence_mut() {
                parse_unserialized_sequence(sequence).map_err(|e| eyre!(e))?;
            }
        }
    }

    if implicit_scope && !query.contains_key("in") {
        query.insert(
            Value::String("in".to_string()),
            Value::String(key_parts[0].to_string()),
        );
    }

    Ok((key_parts[0], query))
}

pub fn normalize_mapping<'a, F>(
    path: &str,
    yaml: &'a mut Mapping,
    mut value_processor: F,
) -> Result<&'a mut Mapping>
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
        let (base_key, shorthand_props) = normalize_key(path, key)?;

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
