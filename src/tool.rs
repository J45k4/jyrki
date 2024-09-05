use std::io::Seek;
use std::io::Write;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WriteFile {
	pub path: String,
	pub content: String,
	pub linenumber: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "name", content = "arguments")] 
pub enum Tool {
	#[serde(rename = "write_file")]
	WriteFile(WriteFile),
	#[serde(rename = "read_file")]
	ReadFile,
	#[serde(rename = "list_folder_contents")]
	ListFolderContents,
	#[serde(rename = "create_folder")]
	CreateTodoItem { text: String },
	#[serde(rename = "delete_folder")]
	CompleteTodoItem,
	#[serde(rename = "find_text")]
	FindText { text: String, path: String },
}

pub struct ToolExecutor {
	forbidden_files: Vec<String>,
	base_path: String,
}

impl ToolExecutor {
	pub fn new() -> ToolExecutor {
		ToolExecutor {
			forbidden_files: vec![],
			base_path: "./workdir".to_string(),
		}
	}

	pub fn execute(&self, id: String, tool: Tool) {
		match tool {
			Tool::WriteFile(w) => {
				let path = Path::new(&self.base_path).join(&w.path);
				let file_name = path.file_name().unwrap().to_str().unwrap();
				if self.forbidden_files.contains(&file_name.to_string()) {
					println!("File {} is forbidden", file_name);
					return;
				}

				let file = std::fs::OpenOptions::new()
					.write(true)
					.create(true)
					.open(&path).unwrap();


				println!("Write file {} with content {}", w.path, w.content);
			},
			Tool::ReadFile => {
				println!("Read file");
			},
			Tool::ListFolderContents => {
				println!("List folder contents");
			},
			Tool::CreateTodoItem { text } => {
				println!("Create todo item {}", text);
			},
			Tool::CompleteTodoItem => {
				println!("Complete todo item");
			},
			Tool::FindText { text, path } => {
				println!("Find text {} in file {}", text, path);
			},
		}
	}

	pub async fn get_result(&mut self) {

	} 
}