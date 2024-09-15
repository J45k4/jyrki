use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;

use crate::generated::Tool;
use crate::history::History;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
	pub name: String,
	pub output_token_count: u32,
	pub input_token_count: u32,
	pub input_token_cost: f32,
	pub output_token_cost: f32,
	pub todo_items: Vec<TodoItem>,
	// items: Vec<ConversationItem>,
	pub history: History,
	pub instructions: String,
	pub current_msg: String,
	pub disallowed_files: Vec<String>,
	pub activated_tools: Vec<Tool>,
	pub folder_path: String,
	pub modified: bool,
}

impl Default for Project {
	fn default() -> Project {
		Project {
			name: "New Project".to_string(),
			output_token_count: 0,
			input_token_count: 0,
			input_token_cost: 0.0,
			output_token_cost: 0.0,
			todo_items: Vec::new(),
			history: History::new(),
			instructions: "".to_string(),
			current_msg: "".to_string(),
			disallowed_files: Vec::new(),
			activated_tools: Vec::new(),
			folder_path: "".to_string(),
			modified: true,
		}
	}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TodoItem {
	pub text: String,
	pub done: bool,
}

#[derive(Debug, Default)]
pub struct State {
	pub projects: Vec<Project>,
	pub active_project: Option<usize>,
	pub current_msg: String,
}

impl State {
	pub fn new() -> State {
		State::default()
	}
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolDef {
	Function {
		name: String,
		description: String,
		parameters: Value,
	}
}