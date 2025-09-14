use std::collections::BTreeMap;

use color_eyre::Result;
use project_manager::util::yaml::load_yaml;
use serde_yaml::Value;

fn main() -> Result<()> {
    color_eyre::install()?;

    let config: BTreeMap<String, Value> = load_yaml("example/no-syntactic-sugar.yaml")?;

    println!("{:#?}", config);

    Ok(())
}
