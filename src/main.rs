use color_eyre::{
    Result,
    eyre::{Report, eyre},
};
use project_manager::{
    config::parse,
    program::{args::ArgToken, *},
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let arg_tokens = args::tokenize_args(std::env::args())?;

    let initial_scope = cli::get_initial_scope_from_args(arg_tokens.clone())?;

    let mut args_iter = arg_tokens.iter().peekable();

    if !matches!(args_iter.peek(), Some(ArgToken::Constant(_))) {
        return Err(eyre!("unexpected end of command, expected command name"));
    }

    let command_name = match args_iter.next().unwrap() {
        ArgToken::Constant(name) => name,
        _ => unreachable!(),
    };

    match command_name.as_str() {
        "list" => {
            let project_config = parse::parse_project_config("./example/project.yaml")?;

            commands::list_commands::run(&project_config)?;

            Ok::<_, Report>(())
        }
        "run" => {
            let project_config = parse::parse_project_config("./example/project.yaml")?;

            let command = match args_iter.next() {
                Some(ArgToken::Constant(name)) => Ok(name),
                _ => Err(eyre!("no or invalid command name provided")),
            }?;

            commands::run_command::run(command, &initial_scope, &project_config)?;

            Ok(())
        }
        _ => {
            let project_config = parse::parse_project_config("./example/project.yaml")?;

            commands::run_command::run(command_name, &initial_scope, &project_config)?;

            Ok(())
        }
    }?;

    Ok(())
}
