use serde::Deserialize;
use serde_json::Value;
use crate::generated::Tool;
use crate::history::History;
use crate::LLMModel;

fn default_folder_path() -> String {
	"./workdir".to_string()
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Project {
	pub name: String,
	#[serde(default)]
	pub model: LLMModel,
	pub output_token_count: u32,
	pub input_token_count: u32,
	pub input_token_cost: f32,
	pub output_token_cost: f32,
	pub todo_items: Vec<TodoItem>,
	pub history: History,
	pub instructions: String,
	pub current_msg: String,
	pub activated_tools: Vec<Tool>,
	#[serde(default = "default_folder_path")]
	pub folder_path: String,
	#[serde(default)]
	pub forbidden_files: Vec<String>,
	pub modified: bool,
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
	pub new_forbidden_file_name: String,
	pub max_conversation_turns: u32,
	pub conversation_turns: u32,
	pub max_context_size: u32,
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