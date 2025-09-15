use std::{fs::File, io::BufReader};

use color_eyre::Result;
use serde::Deserialize;
use serde_yaml::{Sequence, Value};

pub fn load_yaml<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: T = serde_yaml::from_reader(reader)?;
    Ok(data)
}

pub fn parse_unserialized_sequence(sequence: &mut Sequence) -> Result<&mut Sequence> {
    if sequence.iter().all(|item| item.is_string()) {
        sequence.iter_mut().for_each(|item| {
            *item = serde_yaml::from_str::<Value>(item.as_str().unwrap()).unwrap()
        });
    }

    Ok(sequence)
}
