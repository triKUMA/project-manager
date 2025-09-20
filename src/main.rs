use color_eyre::{Result, eyre::eyre};
use project_manager::{
    parse::expand,
    util::{path::try_get_path, yaml},
};
use serde_yaml::Mapping;

fn main() -> Result<()> {
    color_eyre::install()?;

    let project_config_path = try_get_path("example/project.yaml", None)?
        .ok_or_else(|| eyre!("unable to find project.yaml"))?;
    let project_config_path_str = project_config_path.clone().into_string();
    let project_config_path_dir_str = project_config_path
        .parent()
        .ok_or_else(|| {
            eyre!(
                "unable to get directory for config file path: '{}'",
                project_config_path_str
            )
        })?
        .to_path_buf()
        .into_string();

    println!("config path: {project_config_path_str}");
    println!("config dir: {project_config_path_dir_str}");

    let mut project_config: Mapping = yaml::load_yaml(&project_config_path_str)?;
    // let normalized_project_config: Mapping = yaml::load_yaml("example/normalized.yaml")?;

    // println!("{:#?}\n", config);

    expand::expand_project_config(&project_config_path_dir_str, &mut project_config)?;

    println!("{:#?}\n", project_config);
    // println!("{:#?}\n", normalized_config);

    // assert_eq!(
    //     config, normalized_config,
    //     "configs are not equal\nprocessed config: {:#?}\nexpected config: {:#?}",
    //     config, normalized_config
    // );

    Ok(())
}
