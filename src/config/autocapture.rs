use std::collections::HashMap;

use color_eyre::eyre::Result;
use serde_yaml::{Mapping, Value};

use crate::util::path as path_util;

pub fn auto_capture_workspaces<'a>(
    path: &str,
    working_dir: &str,
    workspaces: &'a mut Mapping,
) -> Result<&'a mut Mapping> {
    println!("{path} - auto capturing workspaces");

    let auto_captured_workspaces = path_util::get_sub_directories(working_dir)?
        .into_iter()
        .filter(|(key, _)| !key.starts_with('.'))
        .collect::<HashMap<_, _>>();

    for (key, value) in auto_captured_workspaces {
        let key = key.replace(' ', "-");

        if workspaces.contains_key(key.clone())
            || workspaces.values().any(|v| v.as_str().unwrap() == value)
        {
            println!(
                "WARNING - conflicting key or value already present in user defined workspaces. skipping adding '{key}': '{value}'"
            );
            continue;
        }

        println!("{path} - auto captured '{key}' ({value})");
        workspaces.insert(Value::String(key), Value::String(value));
    }

    println!("{path} - finished capturing workspaces");

    Ok(workspaces)
}
