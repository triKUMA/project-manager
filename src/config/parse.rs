use color_eyre::{Report, Result, eyre::eyre};
use serde_yaml::Mapping;

use crate::{
	config::{autocapture, desugar, expand},
	util::{path as path_util, yaml},
};

pub fn parse_project_config(path: &str) -> Result<Mapping> {
	let project_config_path =
		path_util::try_get_path(path, None)?.ok_or_else(|| eyre!("unable to find '{path}'"))?;
	let project_config_path_str = project_config_path.clone().into_string();
	let project_config_path_dir_str = project_config_path
		.parent()
		.ok_or_else(|| {
			eyre!("unable to get directory for config file path: '{project_config_path_str}'",)
		})?
		.to_path_buf()
		.into_string();

	println!("processing config file: '{project_config_path_str}'");

	let mut project_config: Mapping = yaml::load_yaml(&project_config_path_str)?;

	expand::expand_project_config(&project_config_path_dir_str, &mut project_config)?;

	// can use unwrap for expecting key and value types after expanding, as any invalid key or value types would have thrown an error in the expansion step
	for (key, value) in project_config.iter_mut() {
		match key.as_str().unwrap() {
			"commands" => {
				let commands = value.as_mapping_mut().unwrap();

				desugar::desugar_mapping("commands", &project_config_path_dir_str, commands)?;

				Ok::<_, Report>(())
			}
			"workspaces" => {
				let workspaces = value.as_mapping_mut().unwrap();

				autocapture::auto_capture_workspaces(
					"workspaces",
					&project_config_path_dir_str,
					workspaces,
				)?;

				Ok(())
			}
			_ => Ok(()),
		}?
	}

	if let Some(commands) = project_config["commands"].as_mapping_mut() {
		desugar::desugar_mapping("commands", &project_config_path_dir_str, commands)?;
	}

	Ok(project_config)
}
