use std::env::Args;

use color_eyre::eyre::{Result, eyre};
use serde_yaml::Mapping;

use crate::{
    program::args::{self, ArgToken},
    program::commands,
};

pub fn run(args: Args, config: &Mapping) -> Result<()> {
    let arg_tokens = args::tokenize_args(args)?;

    let initial_scope = get_initial_scope_from_args(arg_tokens.clone())?;

    // process through initial flags/params to build initial scope

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
            commands::list::list_commands(config)?;

            Ok(())
        }
        _ => Err(eyre!("unknown command: {}", command_name)),
    }?;

    Ok(())
}

pub fn get_initial_scope_from_args(args: Vec<ArgToken>) -> Result<Mapping> {
    let mut args_iter = args.iter().peekable();

    let mut initial_scope = Mapping::new();
    while let Some(arg) = args_iter.peek()
        && matches!(arg, args::ArgToken::Flag(_) | args::ArgToken::Param(_, _))
    {
        match args_iter.next() {
            Some(args::ArgToken::Flag(flag)) => {
                initial_scope.insert(
                    serde_yaml::Value::String(flag.clone()),
                    serde_yaml::Value::Bool(true),
                );
            }
            Some(args::ArgToken::Param(flag, value)) => {
                initial_scope.insert(
                    serde_yaml::Value::String(flag.clone()),
                    serde_yaml::Value::String(value.clone()),
                );
            }
            _ => unreachable!(),
        }
    }

    Ok(initial_scope)
}
