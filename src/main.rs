use color_eyre::Result;
use project_manager::config::parse;

fn main() -> Result<()> {
    color_eyre::install()?;

    let project_config = parse::parse_project_config("example/project.yaml");
    // let normalized_project_config = parse_project_config("example/normalized.yaml");

    println!("{:#?}\n", project_config);
    // println!("{:#?}\n", normalized_project_config);

    // assert_eq!(
    //     config, normalized_project_config,
    //     "configs are not equal\nprocessed config: {:#?}\nexpected config: {:#?}",
    //     config, normalized_project_config
    // );

    Ok(())
}
