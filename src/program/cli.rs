use color_eyre::eyre::Result;
use serde_yaml::Mapping;

use crate::{models, program::args::{self, ArgToken}};

pub fn get_initial_scope_from_args(args: Vec<ArgToken>) -> Result<models::command::CommandScope> {
	let mut args_iter = args.iter().peekable();

	let mut args_mapping = Mapping::new();
	while let Some(arg) = args_iter.peek()
		&& matches!(arg, args::ArgToken::Flag(_) | args::ArgToken::Param(_, _))
	{
		match args_iter.next() {
			Some(args::ArgToken::Flag(flag)) => {
				args_mapping.insert(
					serde_yaml::Value::String(flag.clone()),
					serde_yaml::Value::Bool(true),
				);
			}
			Some(args::ArgToken::Param(flag, value)) => {
				args_mapping.insert(
					serde_yaml::Value::String(flag.clone()),
					serde_yaml::Value::String(value.clone()),
				);
			}
			_ => unreachable!(),
		}
	}

	let mut initial_scope = models::command::CommandScope::default();
	initial_scope.accumulate_from_mapping(&args_mapping);

	Ok(initial_scope)
}
