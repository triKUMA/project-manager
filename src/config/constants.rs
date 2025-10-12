pub const SCOPE_RESERVED_KEYS: [&str; 10] =
	["in", "-in", "variables", "pre", "-pre", "post", "-post", "run", "commands", "tasks"];

pub const STATE_RESERVED_KEYS: [&str; 1] = ["variables"];

// TODO: this should be replaced with the config item in the global config file later on
pub const SCOPE_SEPARATOR: &str = ":";
