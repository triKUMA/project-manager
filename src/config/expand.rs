use camino::Utf8PathBuf;
use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::{
    config::{constants, desugar},
    util::{path as path_util, yaml},
};

// TODO:
// - need to auto capture workspaces from root directory
// - need to build reserved variables
//   - $env built from current environment variables
//   - $opts built from command line arguments

pub fn expand_project_config<'a>(
    config_dir: &str,
    config: &'a mut Mapping,
) -> Result<&'a mut Mapping> {
    let keys: Vec<Value> = config.keys().cloned().collect();
    for key in keys {
        if !key.is_string() {
            return Err(eyre!(
                "invalid root key in yaml: {key:#?}\nroot key must be a string"
            ));
        }
    }

    yaml::map_mapping(config, |key: &str, value: &mut Value| {
        if let Some(value_mapping) = value.as_mapping_mut() {
            match key {
                "workspaces" => {
                    expand_workspaces("workspaces", config_dir, value_mapping)?;

                    Ok(())
                }
                "state" => {
                    expand_state("state", value_mapping)?;

                    Ok(())
                }
                "commands" => {
                    expand_scope("commands", config_dir, value_mapping, true)?;

                    Ok(())
                }
                _ => Err(eyre!("unable to process unknown key: {key}")),
            }?;
        }

        Ok(())
    })?;

    Ok(config)
}

pub fn expand_internal_config(config: &mut Mapping) -> Result<&mut Mapping> {
    Ok(config)
}

pub fn expand_workspaces<'a>(
    path: &str,
    config_dir: &str,
    workspaces: &'a mut Mapping,
) -> Result<&'a mut Mapping> {
    println!("{path} - expanding workspaces");

    yaml::map_mapping(workspaces, |key, value| {
        println!("{path} - processing '{key}' (workspace): {value:?}");

        if !value.is_string() {
            return Err(eyre!(
                "key value is invalid type in mapping: {key:#?}\nkey value must be a string"
            ));
        }

        let value_str = value.as_str().unwrap();

        if let Some(path) = path_util::try_get_path(value_str, Some(config_dir.to_string()))? {
            process_path(value, path)?;

            Ok(())
        } else {
            Err(eyre!("unable to get path for '{value_str}'"))
        }
    })?;

    println!("{path} - workspaces expanded");

    Ok(workspaces)
}

pub fn expand_state<'a>(path: &str, state: &'a mut Mapping) -> Result<&'a mut Mapping> {
    println!("{path} - expanding state");

    let shorthand_variable_keys = get_shorthand_variable_keys(state);
    if !shorthand_variable_keys.is_empty() {
        expand_shorthand_variables(path, state, shorthand_variable_keys)?;
    }

    yaml::map_mapping(state, |key, value| match desugar::get_base_key(key, true) {
        "variables" => {
            println!("{path} - processing '{key}' (variables): {value:?}");

            expand_variables(
                format!("{path}.{}", desugar::get_base_key(key, true)).as_str(),
                value.as_mapping_mut().unwrap(),
            )?;

            Ok(())
        }
        _ if constants::STATE_RESERVED_KEYS.contains(&key) => {
            println!("{path} - processing '{key}' (unhandled reserved): {value:?}");

            Err(eyre!(
                "processing a reserved key that should have been explicitly handled: {path}.{key}"
            ))
        }
        _ => {
            println!("{path} - processing '{key}' (unknown): {value:?}");

            Err(eyre!("unable to process unknown key: {path}.{key}"))
        }
    })?;

    println!("{path} - state expanded");

    Ok(state)
}

pub fn expand_scope<'a>(
    path: &str,
    config_dir: &str,
    scope: &'a mut Mapping,
    strict: bool,
) -> Result<&'a mut Mapping> {
    println!("{path} - expanding scope");

    // process shorthand variables if any exist
    let shorthand_variable_keys = get_shorthand_variable_keys(scope);
    if !shorthand_variable_keys.is_empty() {
        expand_shorthand_variables(path, scope, shorthand_variable_keys)?;
    }

    // process run key if it exists
    if has_key(scope, "run") {
        expand_run(path, scope, "run")?;
    }

    // process implicit command keys if any exist
    let implicit_command_keys = get_implicit_command_keys(scope);
    if !implicit_command_keys.is_empty() {
        expand_implicit_commands(path, scope, implicit_command_keys)?;
    }

    yaml::map_mapping(scope, |key, value| match desugar::get_base_key(key, true) {
        "variables" => {
            println!("{path} - processing '{key}' (variables): {value:?}");

            expand_variables(
                format!("{path}.{}", desugar::get_base_key(key, true)).as_str(),
                value.as_mapping_mut().unwrap(),
            )?;

            Ok(())
        }
        "pre" => {
            println!("{path} - processing '{key}' (pre): {value:?}");

            expand_task_collection(
                format!("{path}.{}", desugar::get_base_key(key, true)).as_str(),
                value,
            )?;

            Ok(())
        }
        "post" => {
            println!("{path} - processing '{key}' (post): {value:?}");

            expand_task_collection(
                format!("{path}.{}", desugar::get_base_key(key, true)).as_str(),
                value,
            )?;

            Ok(())
        }
        "commands" => {
            println!("{path} - processing '{key}' (commands): {value:?}");

            expand_commands(
                format!("{path}.{}", desugar::get_base_key(key, true)).as_str(),
                value.as_mapping_mut().unwrap(),
            )?;

            Ok(())
        }
        "in" => {
            println!("{path} - processing '{key}' (in): {value:?}");

            expand_potential_path(format!("{path}.{key}").as_str(), config_dir, value)?;

            Ok(())
        }
        _ if constants::SCOPE_RESERVED_KEYS.contains(&key) => {
            println!("{path} - processing '{key}' (unhandled reserved): {value:?}");

            Err(eyre!(
                "processing a reserved key that should have been explicitly handled: {path}.{key}"
            ))
        }
        _ if value.is_mapping() => {
            println!("{path} - processing '{key}' (sub scope): {value:?}");

            expand_scope(
                format!("{path}.{}", desugar::get_base_key(key, true)).as_str(),
                config_dir,
                value.as_mapping_mut().unwrap(),
                strict,
            )?;

            Ok(())
        }
        _ => {
            println!("{path} - processing '{key}' (unknown): {value:?}");

            if strict {
                Err(eyre!("unable to process unknown key: {path}.{key}"))
            } else {
                Ok(())
            }
        }
    })?;

    println!("{path} - scope expanded");

    Ok(scope)
}

pub fn get_shorthand_variable_keys(scope: &mut Mapping) -> Vec<String> {
    scope
        .keys()
        .cloned()
        .filter_map(|k| {
            if let Some(k) = k.as_str() {
                k.strip_prefix('$').map(|k| k.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn expand_shorthand_variables<'a>(
    scope_path: &str,
    scope: &'a mut Mapping,
    shorthand_variable_keys: Vec<String>,
) -> Result<&'a mut Mapping> {
    if !scope.contains_key("variables") {
        scope.insert(
            Value::String("variables".to_string()),
            Value::Mapping(Mapping::new()),
        );
    }

    for var_key in shorthand_variable_keys {
        let var_key_value = scope.remove(format!("${}", var_key)).unwrap();

        println!("{scope_path} - processing '${var_key}' (variable): {var_key_value:?}");

        scope["variables"]
            .as_mapping_mut()
            .unwrap()
            .insert(Value::String(var_key), var_key_value);
    }

    Ok(scope)
}

pub fn expand_variables<'a>(path: &str, variables: &'a mut Mapping) -> Result<&'a mut Mapping> {
    println!("{path} - expanding variables");

    let shorthand_variable_keys = get_shorthand_variable_keys(variables);
    if !shorthand_variable_keys.is_empty() {
        for var_key in shorthand_variable_keys {
            let var_value = variables.remove(format!("${var_key}")).unwrap();

            variables.insert(Value::String(var_key.clone()), var_value);
        }
    }

    yaml::map_mapping(variables, |key, value| {
        println!("{path} - processing '{key}' (variable): {value:?}");

        let mut var_mapping = Mapping::new();
        var_mapping.insert(Value::String("value".to_string()), value.clone());

        *value = Value::Mapping(var_mapping);

        Ok(())
    })?;

    println!("{path} - variables expanded");

    Ok(variables)
}

pub fn has_key(scope: &mut Mapping, key: &str) -> bool {
    scope.iter().any(|(k, _)| {
        if let Some(k) = k.as_str() {
            desugar::get_base_key(k, false) == key
        } else {
            false
        }
    })
}

pub fn expand_run<'a>(
    scope_path: &str,
    scope: &'a mut Mapping,
    run_key: &str,
) -> Result<&'a mut Mapping> {
    if !scope.contains_key("commands") {
        scope.insert(
            Value::String("commands".to_string()),
            Value::Mapping(Mapping::new()),
        );
    }

    let run_val = scope.remove(run_key).unwrap();

    println!("{scope_path} - processing '{run_key}' (run): {run_val:?}");

    scope["commands"].as_mapping_mut().unwrap().insert(
        Value::String(run_key.replace("run", ".").to_string()),
        run_val,
    );

    Ok(scope)
}

pub fn get_implicit_command_keys(scope: &mut Mapping) -> Vec<String> {
    scope
        .keys()
        .cloned()
        .filter_map(|k| {
            if let Some(key) = k.as_str()
                && !constants::SCOPE_RESERVED_KEYS.contains(&desugar::get_base_key(key, true))
                && scope[key].is_string()
            {
                Some(key.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn expand_implicit_commands<'a>(
    scope_path: &str,
    scope: &'a mut Mapping,
    implicit_command_keys: Vec<String>,
) -> Result<&'a mut Mapping> {
    if !scope.contains_key("commands") {
        scope.insert(
            Value::String("commands".to_string()),
            Value::Mapping(Mapping::new()),
        );
    }

    for implicit_command_key in implicit_command_keys {
        let implicit_command_key_value = scope.remove(implicit_command_key.clone()).unwrap();

        println!(
            "{scope_path} - processing '{implicit_command_key}' (implicit command): {implicit_command_key_value:?}",
        );

        scope["commands"].as_mapping_mut().unwrap().insert(
            Value::String(implicit_command_key),
            implicit_command_key_value,
        );
    }

    Ok(scope)
}

pub fn expand_commands<'a>(path: &str, commands: &'a mut Mapping) -> Result<&'a mut Mapping> {
    println!("{path} - expanding commands");

    yaml::map_mapping(commands, |key, value| {
        println!("{path} - processing '{key}' (command): {value:?}");

        expand_task_collection(
            format!(
                "{path}.{}",
                if key == "." {
                    "\".\""
                } else {
                    desugar::get_base_key(key, true)
                }
            )
            .as_str(),
            value,
        )?;

        Ok(())
    })?;

    println!("{path} - commands expanded");

    Ok(commands)
}

pub fn expand_task_collection<'a>(
    path: &str,
    implicit_task_collection: &'a mut Value,
) -> Result<&'a mut Value> {
    println!("{path} - expanding task collection");

    if !implicit_task_collection.is_mapping() {
        let mut tasks_mapping = Mapping::new();
        tasks_mapping.insert(
            Value::String("tasks".to_string()),
            implicit_task_collection.clone(),
        );

        *implicit_task_collection = Value::Mapping(tasks_mapping);
    }

    let task_collection = implicit_task_collection.get_mut("tasks").unwrap();

    if task_collection.is_string() {
        *task_collection = Value::Sequence(vec![task_collection.clone()]);
    }

    if !task_collection.is_sequence()
        || !task_collection
            .as_sequence()
            .unwrap()
            .iter()
            .all(|i| i.is_string())
    {
        return Err(eyre!(
            "invalid command format in yaml: {task_collection:#?}\ncommand must be a string or array of strings",
        ));
    }

    let task_sequence = task_collection.as_sequence_mut().unwrap();
    *task_sequence = task_sequence
        .iter_mut()
        .flat_map(|i| {
            i.as_str()
                .unwrap()
                .split("&&")
                .map(|s| Value::String(s.trim().to_string()))
        })
        .collect();

    println!("{path} - task collection expanded");

    Ok(implicit_task_collection)
}

pub fn expand_potential_path<'a>(
    key_path: &str,
    config_dir: &str,
    value: &'a mut Value,
) -> Result<&'a mut Value> {
    println!("{key_path} - expanding path/workspace");

    if !value.is_string() {
        return Err(eyre!(
            "key value is invalid type in mapping: {key_path:#?}\nkey value must be a string"
        ));
    }

    let value_str = value.as_str().unwrap();

    if value_str.starts_with("ws:") {
        return Ok(value);
    }

    if let Some(path) = path_util::try_get_path(value_str, Some(config_dir.to_string()))? {
        process_path(value, path)?;
    } else {
        *value = Value::String(format!("ws:{value_str}"));
    }

    println!("{key_path} - path/workspace expanded");

    Ok(value)
}

pub fn process_path(value: &mut Value, path: Utf8PathBuf) -> Result<&mut Value> {
    let path_str = path.clone().into_string();

    if !path.is_dir() {
        return Err(eyre!(
            "invalid working directory path: {path_str}\npath must be to a directory"
        ));
    }

    *value = Value::String(path_str);

    Ok(value)
}
