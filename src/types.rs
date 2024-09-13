use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;

use crate::generated::Tool;
use crate::history::History;

#[derive(Debug)]
pub struct Project {
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
}

#[derive(Debug)]
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