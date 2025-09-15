use std::collections::HashMap;

use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::util::yaml::*;

pub fn desugar_yaml(yaml: &mut Mapping) -> Result<&mut Mapping> {
    let keys: Vec<Value> = yaml.keys().cloned().collect();
    for key in keys {
        if !key.is_string() {
            return Err(eyre!(
                "invalid root key in yaml: {:#?}\nroot key must be a string",
                key
            ));
        }

        let key = key.as_str().unwrap();
        let (base_key, shorthand_props) = desugar_property_shorthand(key)?;
        if base_key != key
            && let Some(mut value) = yaml.remove(key)
        {
            // normalize value to final format
            
            if let Some(value_mapping) = value.as_mapping_mut() {
                soft_merge_mappings(value_mapping, &shorthand_props);
            }

            yaml.insert(Value::String(base_key.to_string()), value);
        }
    }

    Ok(yaml)
}

pub fn desugar_workspaces(yaml: &mut Value) -> Result<&mut Value> {
    Ok(yaml)
}

pub fn desugar_state(yaml: &mut Value) -> Result<&mut Value> {
    Ok(yaml)
}

pub fn desugar_commands(yaml: &mut Value) -> Result<&mut Value> {
    // if mapping
    // - if has run key, update syntax to include 'commands: ".": {run_value}'
    //   - if already has "." command then should throw error for duplicate commands
    // - if has commands key, process sub commands
    // if key has query shorthand syntax
    // - parse query shorthand syntax to key value pairs (no matching value == boolean with value true)
    // - add properties to key's matching object
    // - example: 'test?foo=bar&baz: { greeting: "hello" }' -> 'test: { greeting: "hello", foo: "bar", baz: true }'
    // - query shorthand is lower priority than normal syntax - if property already exists in matching object then shorthand value is ignored
    // if key starts with '!'
    // - update scope to include 'in: {scope_name}'
    // DONT expand glob patterns in this step - should be done in canonizing step when scopes have been extracted

    Ok(yaml)
}

pub fn desugar_property_shorthand(key: &str) -> Result<(&str, Mapping)> {
    let key_parts = key.split('?').collect::<Vec<_>>();

    if key_parts.len() == 1 {
        return Ok((key_parts[0], Mapping::new()));
    }

    if key_parts.len() > 2 {
        return Err(eyre!(
            "invalid property shorthand syntax: {}",
            key_parts[1..].join("?")
        ));
    }

    // TODO: handle the unwraps used here - should return readable errors instead
    let mut query: Mapping = serde_qs::from_str(key_parts[1])?;
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

    Ok((key_parts[0], query))
}
