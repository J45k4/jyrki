use std::io::Seek;
use std::io::Write;
use std::path::Path;

use anyhow::Ok;
// use codegen::ToolCodegen;
use serde::Deserialize;
use serde::Serialize;
use tokio::fs;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;

use crate::generated::ToolCallParameters;

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

	pub async fn execute(&self, tool: ToolCallParameters) -> anyhow::Result<String> {
		let res = match tool {
			ToolCallParameters::WriteFile(w) => {
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
			ToolCallParameters::ReadFile(r) => {
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
			ToolCallParameters::RemoveFile(r) => {
				let path = Path::new(&self.base_path).join(&r.path);
				if !path.exists() {
					return Ok("File does not exist".to_string());
				}
				tokio::fs::remove_file(&path).await?;
				"File removed".to_string()
			}
			ToolCallParameters::ListFolderContent(args) => {
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
			ToolCallParameters::AddNewTodo(args) => {
				todo!()
			},
			ToolCallParameters::CompleteTodo(args) => {
				todo!()
			},
			ToolCallParameters::FindInFile(args) => {
				todo!()
			},
			_ => todo!(),
		};

		Ok(res)
	}

	pub async fn get_result(&mut self) {

	} 
}