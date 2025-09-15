use color_eyre::Result;
use project_manager::{parse::desugar::*, util::yaml::*};
use serde_yaml::Mapping;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut config: Mapping = load_yaml("example/project.yaml")?;

    println!("{:#?}\n", config);

    desugar_yaml(&mut config)?;

    println!("{:#?}\n", config);

    Ok(())
}
