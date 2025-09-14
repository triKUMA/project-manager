use std::{fs::File, io::BufReader};

use color_eyre::Result;
use serde::Deserialize;

pub fn load_yaml<T: for<'de> Deserialize<'de>>(path: &str) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let data: T = serde_yaml::from_reader(reader)?;
    Ok(data)
}
