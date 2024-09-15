use std::path::PathBuf;
use homedir::my_home;

pub fn get_app_dir() -> PathBuf {
	let homedir = my_home().unwrap().unwrap();
	let company_dir = homedir.join(".puppycorp");
	let project_dir = company_dir.join("puppycoder");
	if !project_dir.exists() {
		std::fs::create_dir_all(&project_dir).unwrap();
	}
	project_dir
}

pub fn get_projects_dir() -> PathBuf {
	let projects_dir = get_app_dir().join("projects");
	if !projects_dir.exists() {
		std::fs::create_dir_all(&projects_dir).unwrap();
	}
	projects_dir
}