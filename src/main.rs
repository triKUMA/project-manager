use color_eyre::Result;
use project_manager::{config::parse, program};

fn main() -> Result<()> {
    color_eyre::install()?;

    // let project_config = parse::parse_project_config("./example/project.yaml")?;

    // println!("\n{:#?}", project_config);
    // println!("{:#?}\n", normalized_project_config);

    let arg_tokens = program::args::tokenize_args(std::env::args())?;
    for token in arg_tokens {
        println!("{token:?}");
    }

    Ok(())
}
