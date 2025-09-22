use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::{config::expand, util::yaml};

pub fn desugar_mapping<'a>(
    path: &str,
    config_dir: &str,
    mapping: &'a mut Mapping,
) -> Result<&'a mut Mapping> {
    println!("{path} - desugaring mapping");

    let keys: Vec<Value> = mapping.keys().cloned().collect();
    for key in keys {
        if !key.is_string() {
            return Err(eyre!(
                "key is invalid type in mapping: {key:#?}\nkey must be a string",
            ));
        }

        let key = key.as_str().unwrap();

        println!("{path} - desugaring '{key}'");

        let (base_key, mut shorthand_props) = normalize_key(key)?;
        expand::expand_scope(
            format!("{path}.{base_key}[shorthand_props]").as_str(),
            config_dir,
            &mut shorthand_props,
            false,
        )?;

        if mapping.get(key).is_some() {
            if key == base_key {
                if let Some(value) = mapping.get_mut(key) {
                    match value {
                        Value::Mapping(value_mapping) => {
                            desugar_mapping(
                                format!("{path}.{base_key}").as_str(),
                                config_dir,
                                value_mapping,
                            )?;
                        }
                        Value::Sequence(value_sequence) => {
                            for (i, item) in value_sequence.iter_mut().enumerate() {
                                if let Some(item_mapping) = item.as_mapping_mut() {
                                    desugar_mapping(
                                        format!("{path}.{base_key}[{i}]").as_str(),
                                        config_dir,
                                        item_mapping,
                                    )?;
                                }
                            }
                        }
                        _ => { /* nothing to do for scalars when no shorthand */ }
                    }
                }
            } else if let Some(mut value) = mapping.remove(key) {
                match &mut value {
                    Value::Mapping(value_mapping) => {
                        yaml::soft_merge_mappings(value_mapping, &shorthand_props);
                        desugar_mapping(
                            format!("{path}.{base_key}").as_str(),
                            config_dir,
                            value_mapping,
                        )?;
                    }
                    Value::Sequence(value_sequence) => {
                        for (i, item) in value_sequence.iter_mut().enumerate() {
                            if let Some(item_mapping) = item.as_mapping_mut() {
                                yaml::soft_merge_mappings(item_mapping, &shorthand_props);
                                desugar_mapping(
                                    format!("{path}.{base_key}[{i}]").as_str(),
                                    config_dir,
                                    item_mapping,
                                )?;
                            } else {
                                return Err(eyre!(
                                    "unable to merge shorthand props into key value for key '{path}.{base_key}'. value is invalid type (must be mapping or sequence)"
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(eyre!(
                            "unable to merge shorthand props into key value for key '{path}.{base_key}'. value is invalid type (must be mapping or sequence)"
                        ));
                    }
                }

                mapping.insert(Value::String(base_key.to_string()), value);
            }
        }
    }

    println!("{path} - mapping desugared");

    Ok(mapping)
}

fn normalize_key(key: &str) -> Result<(&str, Mapping)> {
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
        query = serde_qs::from_str(key_parts[1])?;

        for (_, v) in query.iter_mut() {
            match v {
                Value::String(v_str) => {
                    if v_str.is_empty() {
                        *v = Value::Bool(true);
                    } else {
                        *v = serde_yaml::from_str::<Value>(v_str).map_err(|e| eyre!(e))?;
                    }
                }
                Value::Sequence(v_seq) => {
                    yaml::parse_unserialized_sequence(v_seq).map_err(|e| eyre!(e))?;
                }
                _ => {}
            }
        }
    }

    // only include 'in' if shorthand props dont already contain it
    if implicit_scope && !query.contains_key("in") {
        query.insert(
            Value::String("in".to_string()),
            Value::String(key_parts[0].to_string()),
        );
    }

    Ok((key_parts[0], query))
}

pub fn get_base_key(key: &str, allow_implicit_scope: bool) -> &str {
    let key = if allow_implicit_scope {
        key.strip_prefix('!').unwrap_or(key)
    } else {
        key
    };

    key.split('?').next().unwrap()
}
