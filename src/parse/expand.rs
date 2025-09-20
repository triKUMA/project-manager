use std::path::Path;

use camino::Utf8Path;
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

pub fn expand_project_config(yaml: &mut Mapping) -> Result<&mut Mapping> {
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

                    Ok(())
                }
                "state" => {
                    expand_state("state", value_mapping)?;

                    Ok(())
                }
                "commands" => {
                    expand_scope("commands", value_mapping)?;

                    Ok(())
                }
                _ => Err(eyre!("unable to process unknown key: {key}")),
            }?;
        }

        Ok(())
    })?;

    Ok(yaml)
}

pub fn expand_workspaces<'a>(path: &str, workspaces: &'a mut Mapping) -> Result<&'a mut Mapping> {
    println!("{path} - expanding workspaces");

    println!("{path} - workspaces expanded");

    Ok(workspaces)
}

pub fn expand_state<'a>(path: &str, state: &'a mut Mapping) -> Result<&'a mut Mapping> {
    println!("{path} - expanding state");

    let shorthand_variable_keys = get_shorthand_variable_keys(state);
    if !shorthand_variable_keys.is_empty() {
        expand_shorthand_variables(path, state, shorthand_variable_keys)?;
    }

    yaml::map_mapping(
        state,
        |child_key, child_value| match desugar::get_base_key(child_key, true) {
            "variables" => {
                println!(
                    "{path} - processing '{child_key}' (variables): {:?}",
                    child_value
                );

                expand_variables(
                    format!("{path}.{}", desugar::get_base_key(child_key, true)).as_str(),
                    child_value.as_mapping_mut().unwrap(),
                )?;

                Ok(())
            }
            _ if constants::STATE_RESERVED_KEYS.contains(&child_key) => {
                println!(
                    "{path} - processing '{child_key}' (unhandled reserved): {:?}",
                    child_value
                );

                Err(eyre!(
                    "processing a reserved key that should have been explicitly handled: {path}.{child_key}"
                ))
            }
            _ => {
                println!(
                    "{path} - processing '{child_key}' (unknown): {:?}",
                    child_value
                );

                Err(eyre!("unable to process unknown key: {path}.{child_key}"))
            }
        },
    )?;

    println!("{path} - state expanded");

    Ok(state)
}

pub fn expand_scope<'a>(path: &str, scope: &'a mut Mapping) -> Result<&'a mut Mapping> {
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

    yaml::map_mapping(
        scope,
        |child_key, child_value| match desugar::get_base_key(child_key, true) {
            "variables" => {
                println!(
                    "{path} - processing '{child_key}' (variables): {:?}",
                    child_value
                );

                expand_variables(
                    format!("{path}.{}", desugar::get_base_key(child_key, true)).as_str(),
                    child_value.as_mapping_mut().unwrap(),
                )?;

                Ok(())
            }
            "pre" => {
                println!("{path} - processing '{child_key}' (pre): {:?}", child_value);

                expand_task_collection(
                    format!("{path}.{}", desugar::get_base_key(child_key, true)).as_str(),
                    child_value,
                )?;

                Ok(())
            }
            "post" => {
                println!(
                    "{path} - processing '{child_key}' (post): {:?}",
                    child_value
                );

                expand_task_collection(
                    format!("{path}.{}", desugar::get_base_key(child_key, true)).as_str(),
                    child_value,
                )?;

                Ok(())
            }
            "commands" => {
                println!(
                    "{path} - processing '{child_key}' (commands): {:?}",
                    child_value
                );

                expand_commands(
                    format!("{path}.{}", desugar::get_base_key(child_key, true)).as_str(),
                    child_value.as_mapping_mut().unwrap(),
                )?;

                Ok(())
            }
            "in" => {
                println!("{path} - processing '{child_key}' (in): {:?}", child_value);

                expand_potential_path(format!("{path}.{child_key}").as_str(), child_value)?;

                Ok(())
            }
            _ if constants::SCOPE_RESERVED_KEYS.contains(&child_key) => {
                println!(
                    "{path} - processing '{child_key}' (unhandled reserved): {:?}",
                    child_value
                );

                Err(eyre!(
                    "processing a reserved key that should have been explicitly handled: {path}.{child_key}"
                ))
            }
            _ if child_value.is_mapping() => {
                println!(
                    "{path} - processing '{child_key}' (sub scope): {:?}",
                    child_value
                );

                expand_scope(
                    format!("{path}.{}", desugar::get_base_key(child_key, true)).as_str(),
                    child_value.as_mapping_mut().unwrap(),
                )?;

                Ok(())
            }
            _ => {
                println!(
                    "{path} - processing '{child_key}' (unknown): {:?}",
                    child_value
                );

                Err(eyre!("unable to process unknown key: {path}.{child_key}"))
            }
        },
    )?;

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

        println!(
            "{scope_path} - processing '${var_key}' (variable): {:?}",
            var_key_value
        );

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
        println!("{path} - processing '{key}' (variable): {:?}", value);

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

    println!("{scope_path} - processing '{run_key}' (run): {:?}", run_val);

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
            "{scope_path} - processing '{implicit_command_key}' (implicit command): {:?}",
            implicit_command_key_value
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
        println!("{path} - processing '{key}' (command): {:?}", value);

        expand_task_collection(
            format!("{path}.{}", if key == "." { "\".\"" } else { key }).as_str(),
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

    if !task_collection.is_sequence() {
        return Err(eyre!(
            "invalid command format in yaml: {:#?}\ncommand must be a string or array of strings",
            task_collection
        ));
    }

    println!("{path} - task collection expanded");

    Ok(implicit_task_collection)
}

pub fn expand_potential_path<'a>(path: &str, _in: &'a mut Value) -> Result<&'a mut Value> {
    if !_in.is_string() {
        return Err(eyre!(
            "key is invalid type in mapping: {:#?}\nkey must be a string",
            path
        ));
    }

    let in_str = _in.as_str().unwrap();

    if let Some(in_path) = try_get_path(in_str) {
        *_in = Value::String(
            in_path
                .canonicalize()?
                .into_os_string()
                .into_string()
                .map_err(|_| eyre!("unable to get path string from {:?}", in_str))?,
        )
    }
    // if path doesn't exist

    Ok(_in)
}

pub fn try_get_path(value: &str) -> Option<&Utf8Path> {
    // Cross-platform “pathy” heuristics:
    // 1) absolute (Unix)   -> "/..."
    // 2) absolute (Windows)-> "C:\..." or "C:/..."
    // 3) UNC (Windows)     -> "\\server\share"
    // 4) relative markers  -> "./", "../", ".\", "..\"
    // 5) home-ish          -> "~/" or "~\"
    // 6) contains a path separator ('/' or '\')

    let path = Some(Utf8Path::new(value));

    if value.starts_with('/') {
        return path;
    }
    if value.starts_with("./")
        || value.starts_with(".\\")
        || value.starts_with("../")
        || value.starts_with("..\\")
    {
        return path;
    }
    if value.starts_with("~/") || value.starts_with("~\\") {
        return path;
    }
    if value.contains('/') || value.contains('\\') {
        return path;
    }
    // Windows drive letter "C:\..." or "C:/..."
    let bytes = value.as_bytes();
    if bytes.len() >= 3
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
        && bytes[0].is_ascii_alphabetic()
    {
        return path;
    }
    // UNC path
    if value.starts_with(r"\\") {
        return path;
    }

    None
}
