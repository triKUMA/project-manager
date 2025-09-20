use color_eyre::Result;
use project_manager::{parse::expand, util::yaml};
use serde_yaml::Mapping;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut project_config: Mapping = yaml::load_yaml("example/project.yaml")?;
    let normalized_project_config: Mapping = yaml::load_yaml("example/normalized.yaml")?;

    // println!("{:#?}\n", config);

    expand::expand_project_config(&mut project_config)?;

    println!("{:#?}\n", project_config);
    // println!("{:#?}\n", normalized_config);

    // assert_eq!(
    //     config, normalized_config,
    //     "configs are not equal\nprocessed config: {:#?}\nexpected config: {:#?}",
    //     config, normalized_config
    // );

    Ok(())
}
