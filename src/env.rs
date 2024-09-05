use std::path::Path;

pub fn load_envs() {
	let path = Path::new(".env");
	if !path.exists() {
		return;
	}
	let content = std::fs::read_to_string(path).unwrap();
	for line in content.lines() {
		let parts: Vec<&str> = line.split('=').collect();
		if parts.len() != 2 {
			continue;
		}

		let key = parts[0];
		let value = parts[1];
		std::env::set_var(key, value);
	}
}