use std::{collections::HashMap};

use serde_yaml::{Mapping, Value};

#[derive(Default, Debug)]
pub struct TaskCollection {
	pub working_dir: Option<String>,
	pub tasks: Vec<String>,
	pub background: bool,
	pub parallel: bool
}

pub type TaskGrouping = Vec<TaskCollection>;

#[derive(Default, Debug)]
pub struct CommandScope {
	pub variables: HashMap<String, Value>,
	pub working_dir: Option<String>,
	pub pre_tasks: Option<TaskGrouping>,
	pub command: TaskCollection,
	pub post_tasks: Option<TaskGrouping>
}

impl CommandScope {
	pub fn accumulate_from_mapping(&mut self, mapping: &Mapping) {
		for (k, v) in mapping {
			match k.as_str().unwrap() {
				"variables" => {
					let variables_mapping = v.as_mapping().unwrap();
					for (k, v) in variables_mapping.iter() {
						self.variables.insert(
							k.as_str().unwrap().to_string(),
							v.as_mapping().unwrap().get("value").unwrap().clone()
						);
					}
				}
				"-in" => {
					self.working_dir = None;
				}
				"in" => {
					self.working_dir = Some(v.as_str().unwrap().to_string());
				},
				"-pre" => {
					self.pre_tasks = None;
				}
				"-post" => {
					self.post_tasks = None;
				}
				"pre" | "post" => {
					let tasks_val = match k.as_str().unwrap() {
						"pre" => &mut self.pre_tasks,
						"post" => &mut self.post_tasks,
						_ => unreachable!()
					};
					
					if tasks_val.is_none() {
						*tasks_val = Some(TaskGrouping::default());
					}

					if let Some(tasks) = tasks_val {
						let task_mapping = v.as_mapping().unwrap();
						let task_collection = TaskCollection {
							working_dir: task_mapping.get("in").map(|in_val| in_val.as_str().unwrap().to_string()),
							tasks: task_mapping
								.get("tasks")
								.unwrap()
								.as_sequence()
								.unwrap()
								.iter()
								.map(|i| i
									.as_str()
									.unwrap()
									.to_string()
								).collect(),
							background: if let Some(background) = task_mapping.get("background") {
								background.as_bool().unwrap()
							} else {
								false
							},
							parallel: if let Some(parallel) = task_mapping.get("parallel") {
								parallel.as_bool().unwrap()
							} else {
								false
							}
						};
						
						tasks.push(task_collection);
					}
				},
				_ if v.is_mapping() && v.as_mapping().unwrap().contains_key("tasks") => {
					let task_mapping = v.as_mapping().unwrap();

					if task_mapping.contains_key("-in") {
						self.command.working_dir = None;
					} else if task_mapping.contains_key("in") {
						self.command.working_dir = Some(task_mapping.get("in").unwrap().as_str().unwrap().to_string());
					}

					self.command.tasks = task_mapping
						.get("tasks")
						.unwrap()
						.as_sequence()
						.unwrap()
						.iter()
						.map(|i| i
							.as_str()
							.unwrap()
							.to_string()
						).collect();

					if task_mapping.contains_key("-background") {
						self.command.background = false;
					} else if task_mapping.contains_key("background") {
						self.command.background = task_mapping.get("background").unwrap().as_bool().unwrap()
					}

					if task_mapping.contains_key("-parallel") {
						self.command.parallel = false;
					} else if task_mapping.contains_key("parallel") {
						self.command.parallel = task_mapping.get("parallel").unwrap().as_bool().unwrap()
					}
				},
				_ => {},
			};
		}
	}
}