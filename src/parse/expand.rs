use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::{
    parse::{constants, desugar},
    util::yaml,
};

// TODO:
// - need to auto capture workspaces from root directory
// - need to build reserved variables
//   - $env built from current environment variables
//   - $opts built from command line arguments

pub fn expand_yaml(yaml: &mut Mapping) -> Result<&mut Mapping> {
    let keys: Vec<Value> = yaml.keys().cloned().collect();
    for key in keys {
        if !key.is_string() {
            return Err(eyre!(
                "invalid root key in yaml: {:#?}\nroot key must be a string",
                key
            ));
        }
    }

    yaml::map_mapping(yaml, |key: &str, value: &mut Value| {
        if let Some(value_mapping) = value.as_mapping_mut() {
            match key {
                "workspaces" => {
                    expand_workspaces("workspaces", value_mapping)?;
                }
                "state" => {
                    expand_state("state", value_mapping)?;
                }
                "commands" => {
                    expand_scope("commands", value_mapping)?;
                }
                _ => (),
            };
        }

        Ok(())
    })?;

    Ok(yaml)
}

pub fn expand_workspaces<'a>(key: &str, value: &'a mut Mapping) -> Result<&'a mut Mapping> {
    Ok(value)
}

pub fn expand_state<'a>(key: &str, value: &'a mut Mapping) -> Result<&'a mut Mapping> {
    Ok(value)
}

pub fn expand_scope<'a>(key: &str, value: &'a mut Mapping) -> Result<&'a mut Mapping> {
    println!("{key} - expanding scope");

    /*
    scope can contain:
    - sub scopes (determined if all scope mapping key values are mappings)
    - pre, post, run, commands, variables (determined if all scope mapping keys are in SCOPE_RESERVED_KEYS)
    - implicit commands map (determined if all scope mapping key values are strings)
     */

    // extract any keys in scope that are variables
    let variable_keys = value
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
        if !value.contains_key("variables") {
            value.insert(
                Value::String("variables".to_string()),
                Value::Mapping(Mapping::new()),
            );
        }

        for var_key in variable_keys {
            let var_key_value = value.remove(format!("${}", var_key)).unwrap();

            println!(
                "{key} - processing '${var_key}' (variable): {:?}",
                var_key_value
            );

            // TODO: update variable format from key: string to key: { value: string }. this will allow setting properties on variables (more future proof)
            value["variables"]
                .as_mapping_mut()
                .unwrap()
                .insert(Value::String(var_key), var_key_value);
        }
    }

    // get run key and list of implicit command keys
    let run_key = value
        .iter()
        .find(|(k, _)| {
            if let Some(k) = k.as_str() {
                desugar::get_base_key(k, false) == "run"
            } else {
                false
            }
        })
        .map(|(k, _)| k.as_str().unwrap().to_string());

    let implicit_command_keys = value
        .keys()
        .cloned()
        .filter_map(|k| {
            if let Some(key) = k.as_str()
                && !constants::SCOPE_RESERVED_KEYS.contains(&desugar::get_base_key(key, true))
                && value[key].is_string()
            {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // add commands key if it doesnt exist yet but will be needed
    if (run_key.is_some() || !implicit_command_keys.is_empty()) && !value.contains_key("commands") {
        value.insert(
            Value::String("commands".to_string()),
            Value::Mapping(Mapping::new()),
        );
    }

    if let Some(run_key) = run_key
        && let Some(run_val) = value.remove(run_key.clone())
    {
        println!("{key} - processing '{run_key}' (run): {:?}", run_val);
        value["commands"].as_mapping_mut().unwrap().insert(
            Value::String(run_key.replace("run", ".").to_string()),
            run_val,
        );
    }

    for implicit_command_key in implicit_command_keys {
        let implicit_command_key_value = value.remove(implicit_command_key.clone()).unwrap();

        println!(
            "{key} - processing '{implicit_command_key}' (implicit command): {:?}",
            implicit_command_key_value
        );

        value["commands"].as_mapping_mut().unwrap().insert(
            Value::String(implicit_command_key),
            implicit_command_key_value,
        );
    }

    // TODO: could move a lot of the logic above down into the map_mapping below
    yaml::map_mapping(
        value,
        |child_key, child_value| match desugar::get_base_key(child_key, true) {
            "in" => {
                println!("{key} - processing '{child_key}' (in): {:?}", child_value);
                Ok(())
            }
            "pre" => {
                println!("{key} - processing '{child_key}' (pre): {:?}", child_value);
                Ok(())
            }
            "post" => {
                println!("{key} - processing '{child_key}' (post): {:?}", child_value);
                Ok(())
            }
            "commands" => {
                println!(
                    "{key} - processing '{child_key}' (commands): {:?}",
                    child_value
                );
                Ok(())
            }
            _ if constants::SCOPE_RESERVED_KEYS.contains(&child_key) => {
                println!(
                    "{key} - processing '{child_key}' (reserved): {:?}",
                    child_value
                );
                Ok(())
            }
            _ if child_value.is_mapping() => {
                println!(
                    "{key} - processing '{child_key}' (sub scope): {:?}",
                    child_value
                );
                expand_scope(
                    format!("{}.{}", key, desugar::get_base_key(child_key, true)).as_str(),
                    child_value.as_mapping_mut().unwrap(),
                )?;

                Ok(())
            }
            _ => {
                println!(
                    "{key} - processing '{child_key}' (custom): {:?}",
                    child_value
                );
                Ok(())
            }
        },
    )?;

    println!("{key} - scope expanded");

    Ok(value)
}
