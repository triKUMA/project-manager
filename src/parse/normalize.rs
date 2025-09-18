use color_eyre::{Result, eyre::eyre};
use serde_yaml::{Mapping, Value};

use crate::util::yaml;

pub fn normalize_key(key: &str) -> Result<(&str, Mapping)> {
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
                yaml::parse_unserialized_sequence(sequence).map_err(|e| eyre!(e))?;
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

pub fn get_base_key(key: &str) -> &str {
    key.strip_prefix('!')
        .unwrap_or(key)
        .split('?')
        .next()
        .unwrap()
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
        let (base_key, shorthand_props) = normalize_key(key)?;

        if let Some(mut value) = yaml.remove(key) {
            let mut props_pre_merged = false;

            if let Some(value_mapping) = value.as_mapping_mut() {
                yaml::soft_merge_mappings(value_mapping, &shorthand_props);
                props_pre_merged = true;
            } else if let Some(value_sequence) = value.as_sequence_mut() {
                for item in value_sequence.iter_mut() {
                    if let Some(item_mapping) = item.as_mapping_mut() {
                        yaml::soft_merge_mappings(item_mapping, &shorthand_props);
                        props_pre_merged = true;
                    }
                }
            }

            // Process the value using the provided closure
            value_processor(base_key, &mut value)?;

            if !props_pre_merged {
                if let Some(value_mapping) = value.as_mapping_mut() {
                    yaml::soft_merge_mappings(value_mapping, &shorthand_props);
                } else if let Some(value_sequence) = value.as_sequence_mut() {
                    for item in value_sequence.iter_mut() {
                        if let Some(item_mapping) = item.as_mapping_mut() {
                            yaml::soft_merge_mappings(item_mapping, &shorthand_props);
                        }
                    }
                }
            }

            yaml.insert(Value::String(base_key.to_string()), value);
        }
    }

    Ok(yaml)
}
