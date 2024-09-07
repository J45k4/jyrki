use std::io::Seek;
use std::io::Write;
use std::path::Path;

use anyhow::Ok;
use serde::Deserialize;
use serde::Serialize;
use tokio::fs;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WriteFile {
	pub path: String,
	pub content: String,
	pub linenumber: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReadFile {
	pub path: String,
	pub start_line_number: u32,
	pub linenumber_count: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ListFolderContents {
	pub path: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateTodoItem {
	pub name: String,
	pub text: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CompleteTodoItem {
	pub name: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FindText {
	pub text: String,
	pub path: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "name", content = "arguments")] 
pub enum Tool {
	#[serde(rename = "write_file")]
	WriteFile(WriteFile),
	#[serde(rename = "read_file")]
	ReadFile(ReadFile),
	#[serde(rename = "list_folder_contents")]
	ListFolderContents(ListFolderContents),
	#[serde(rename = "create_folder")]
	CreateTodoItem(CreateTodoItem),
	#[serde(rename = "delete_folder")]
	CompleteTodoItem(CompleteTodoItem),
	#[serde(rename = "find_text")]
	FindText(FindText),
}

impl Tool {
    pub fn parse(name: &str, args: &str) -> anyhow::Result<Tool> {
        println!("Parsing tool: {} with args: {}", name, args);

        // First, parse the args from a JSON string into a serde_json::Value
        let args_value: serde_json::Value = serde_json::from_str(args)?;

        // Now, match the tool name and deserialize the args_value into the appropriate type
        let tool = match name {
            "write_file" => Tool::WriteFile(serde_json::from_value(args_value)?),
            "read_file" => Tool::ReadFile(serde_json::from_value(args_value)?),
            "list_folder_contents" => Tool::ListFolderContents(serde_json::from_value(args_value)?),
            "create_folder" => Tool::CreateTodoItem(serde_json::from_value(args_value)?),
            "delete_folder" => Tool::CompleteTodoItem(serde_json::from_value(args_value)?),
            "find_text" => Tool::FindText(serde_json::from_value(args_value)?),
            _ => return Err(anyhow::anyhow!("Unknown tool name: {}", name)),
        };

        Ok(tool)
    }
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

	pub async fn execute(&self, tool: Tool) -> anyhow::Result<String> {
		let res = match tool {
			Tool::WriteFile(w) => {
				let path = Path::new(&self.base_path).join(&w.path);
				let file_name = path.file_name().unwrap().to_str().unwrap();
				if self.forbidden_files.contains(&file_name.to_string()) {
					println!("File {} is forbidden", file_name);
					return Ok("You are not allowed to write this file".to_string());
				}

				let lines_to_write: Vec<&str> = w.content.lines().collect();
				let mut file = OpenOptions::new()
					.read(true)
					.write(true)
					.create(true)
					.open(&path)
					.await?;
				let mut content = String::new();
				file.read_to_string(&mut content).await?;
				let mut lines: Vec<&str> = content.lines().collect();
				for l in w.linenumber..lines_to_write.len().min(lines.len()) as u32 {
					lines[l as usize] = lines_to_write[(l - w.linenumber) as usize];
				}

				if lines_to_write.len() + w.linenumber as usize > lines.len() {
					lines.extend(lines_to_write.iter().skip(lines.len() - w.linenumber as usize));
				}

				let content = lines.join("\n");
				file.seek(tokio::io::SeekFrom::Start(0)).await?;
				file.write(&content.as_bytes()).await?;

				"File written".to_string()
			},
			Tool::ReadFile(r) => {
				let path = Path::new(&self.base_path).join(&r.path);
                let mut file = File::open(&path).await?;
                let mut content = String::new();
                file.read_to_string(&mut content).await?;

                let lines: Vec<&str> = content.lines().collect();
                let start = r.start_line_number as usize;
                let end = std::cmp::min(start + r.linenumber_count as usize, lines.len());
                let selected_lines = &lines[start..end];

                selected_lines.join("\n")
			},
			Tool::ListFolderContents(args) => {
				let path = Path::new(&self.base_path).join(&args.path);

				if !path.exists() {
					return Ok("Path does not exist".to_string());
				}

				let mut paths = fs::read_dir(&path).await?;
                let mut contents = vec![];
				while let Some(path) = paths.next_entry().await? {
					let entry = path.path();
					contents.push(entry.to_string_lossy().to_string());
				}
                contents.join("\n")
			},
			Tool::CreateTodoItem(args) => {
				todo!()
			},
			Tool::CompleteTodoItem(args) => {
				todo!()
			},
			Tool::FindText(args) => {
				todo!()
			},
		};

		Ok(res)
	}

	pub async fn get_result(&mut self) {

	} 
}