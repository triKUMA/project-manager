use std::{fs::File, io::BufReader};

use color_eyre::Result;
use serde::Deserialize;
use serde_yaml::{Mapping, Sequence, Value};

pub fn load_yaml<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: T = serde_yaml::from_reader(reader)?;
    Ok(data)
}

pub fn parse_unserialized_sequence(sequence: &mut Sequence) -> Result<&mut Sequence> {
    if sequence.iter().all(|item| item.is_string()) {
        for item in sequence.iter_mut() {
            *item = serde_yaml::from_str::<Value>(item.as_str().unwrap())?
        }
    }

    Ok(sequence)
}

pub fn soft_merge_mappings(base: &mut Mapping, merger: &Mapping) {
    for (k, v) in merger.iter() {
        if !base.contains_key(k) {
            base.insert(k.clone(), v.clone());
        } else if let Some(merger_child_mapping) = v.as_mapping()
            && let Some(base_child_mapping) = base[k].as_mapping_mut()
        {
            soft_merge_mappings(base_child_mapping, merger_child_mapping);
        }
    }
}
