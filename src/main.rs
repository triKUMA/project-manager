use color_eyre::Result;
use project_manager::config::parse;

fn main() -> Result<()> {
    color_eyre::install()?;

    let project_config = parse::parse_project_config("./example/project.yaml")?;

    println!("\n{:#?}", project_config);
    // println!("{:#?}\n", normalized_project_config);

    Ok(())
}
