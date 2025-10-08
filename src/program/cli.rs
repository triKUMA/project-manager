use color_eyre::eyre::Result;
use serde_yaml::Mapping;

use crate::program::args::{self, ArgToken};

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
