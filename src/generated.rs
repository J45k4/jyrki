
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Tool {
	ReadFile,
	WriteFile,
	RemoveFile,
	AddNewTodo,
	CompleteTodo,
	AddMemory,
	ForgetMemory,
	ListFolderContent,
	FindInFile
}

impl ToString for Tool {
	fn to_string(&self) -> String {
		match self {
			Tool::ReadFile => "Read File".to_string(),
			Tool::WriteFile => "Write File".to_string(),
			Tool::RemoveFile => "Remove File".to_string(),
			Tool::AddNewTodo => "Add New Todo".to_string(),
			Tool::CompleteTodo => "Complete Todo".to_string(),
			Tool::AddMemory => "Add Memory".to_string(),
			Tool::ForgetMemory => "Forget Memory".to_string(),
			Tool::ListFolderContent => "List Folder Content".to_string(),
			Tool::FindInFile => "Find In File".to_string(),
		}
	}
}

impl Tool {
	pub fn get_name(&self) -> &str {
		match self {
			Tool::ReadFile => "read_file",
			Tool::WriteFile => "write_file",
			Tool::RemoveFile => "remove_file",
			Tool::AddNewTodo => "add_new_todo",
			Tool::CompleteTodo => "complete_todo",
			Tool::AddMemory => "add_memory",
			Tool::ForgetMemory => "forget_memory",
			Tool::ListFolderContent => "list_folder_content",
			Tool::FindInFile => "find_in_file",
		}
	}

	pub fn get_description(&self) -> &str {
		match self {
			Tool::ReadFile => "Read file contents",
			Tool::WriteFile => "Write file contents",
			Tool::RemoveFile => "Remove file",
			Tool::AddNewTodo => "Add new todo item for your self",
			Tool::CompleteTodo => "Complete todo item",
			Tool::AddMemory => "Add which is always available for you however you can only keep 20 memories at a time",
			Tool::ForgetMemory => "You can forget memories with this tool to free up space",
			Tool::ListFolderContent => "List folder content",
			Tool::FindInFile => "Find content in file",
		}
	}

	pub fn get_parameters(&self) -> serde_json::Value {
		match self {
			Tool::ReadFile => serde_json::json!({"properties":{"linenumber_count":{"description":"Length of the content you want to read. Default is full file","type":"integer"},"path":{"description":"Path of file you want to read","type":"string"},"start_line_number":{"description":"Offset from which you want to read the file. Default is 0","type":"integer"}},"required":["linenumber_count","path","start_line_number"],"type":"object"}),
			Tool::WriteFile => serde_json::json!({"properties":{"content":{"description":"Content you want to write in file","type":"string"},"linenumber":{"description":"Linenumber from which you want to write to the file.","type":"integer"},"path":{"description":"Path of file you want to write","type":"string"}},"required":["content","linenumber","path"],"type":"object"}),
			Tool::RemoveFile => serde_json::json!({"properties":{"path":{"description":"Path of file you want to delete","type":"string"}},"required":["path"],"type":"object"}),
			Tool::AddNewTodo => serde_json::json!({"properties":{"content":{"description":"Content of the todo item","type":"string"},"name":{"description":"Name of the todo item","type":"string"}},"required":["content"],"type":"object"}),
			Tool::CompleteTodo => serde_json::json!({"properties":{"name":{"description":"Name of the todo item you want to complete","type":"string"}},"required":["name"],"type":"object"}),
			Tool::AddMemory => serde_json::json!({"properties":{"content":{"description":"Content you want to remember","type":"string"},"name":{"description":"Name of the memory","type":"string"}},"required":["content"],"type":"object"}),
			Tool::ForgetMemory => serde_json::json!({"properties":{"name":{"description":"Name of the memory you want to forget","type":"string"}},"required":["name"],"type":"object"}),
			Tool::ListFolderContent => serde_json::json!({"properties":{"path":{"description":"Path of the folder you want to list","type":"string"}},"required":["path"],"type":"object"}),
			Tool::FindInFile => serde_json::json!({"properties":{"path":{"description":"Path of the file in which you want to search","type":"string"},"pattern":{"description":"Pattern you want to search","type":"string"}},"required":["path","pattern"],"type":"object"}),
		}
	}
}

pub const TOOLS: [Tool; 9] = [
	Tool::ReadFile,
	Tool::WriteFile,
	Tool::RemoveFile,
	Tool::AddNewTodo,
	Tool::CompleteTodo,
	Tool::AddMemory,
	Tool::ForgetMemory,
	Tool::ListFolderContent,
	Tool::FindInFile,
];
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadFile {
	pub linenumber_count: u32,
	pub path: String,
	pub start_line_number: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WriteFile {
	pub content: String,
	pub linenumber: u32,
	pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoveFile {
	pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddNewTodo {
	pub content: String,
	pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompleteTodo {
	pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddMemory {
	pub content: String,
	pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ForgetMemory {
	pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ListFolderContent {
	pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FindInFile {
	pub path: String,
	pub pattern: String,
}
#[derive(Debug, Clone)]
pub enum ToolCallParameters {
	ReadFile(ReadFile),
	WriteFile(WriteFile),
	RemoveFile(RemoveFile),
	AddNewTodo(AddNewTodo),
	CompleteTodo(CompleteTodo),
	AddMemory(AddMemory),
	ForgetMemory(ForgetMemory),
	ListFolderContent(ListFolderContent),
	FindInFile(FindInFile)
}

impl ToolCallParameters {
	pub fn get_name(&self) -> &str {
		match self {
			ToolCallParameters::ReadFile(_) => "read_file",
			ToolCallParameters::WriteFile(_) => "write_file",
			ToolCallParameters::RemoveFile(_) => "remove_file",
			ToolCallParameters::AddNewTodo(_) => "add_new_todo",
			ToolCallParameters::CompleteTodo(_) => "complete_todo",
			ToolCallParameters::AddMemory(_) => "add_memory",
			ToolCallParameters::ForgetMemory(_) => "forget_memory",
			ToolCallParameters::ListFolderContent(_) => "list_folder_content",
			ToolCallParameters::FindInFile(_) => "find_in_file",
		}
	}

	pub fn get_args(&self) -> String {
		match self {
			ToolCallParameters::ReadFile(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::WriteFile(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::RemoveFile(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::AddNewTodo(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::CompleteTodo(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::AddMemory(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::ForgetMemory(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::ListFolderContent(args) => serde_json::to_string(args).unwrap(),
			ToolCallParameters::FindInFile(args) => serde_json::to_string(args).unwrap(),
		}
	}

	pub fn parse(name: &str, args: &str) -> anyhow::Result<ToolCallParameters> {
		match name {
			"read_file" => Ok(ToolCallParameters::ReadFile(serde_json::from_str(args)?)),
			"write_file" => Ok(ToolCallParameters::WriteFile(serde_json::from_str(args)?)),
			"remove_file" => Ok(ToolCallParameters::RemoveFile(serde_json::from_str(args)?)),
			"add_new_todo" => Ok(ToolCallParameters::AddNewTodo(serde_json::from_str(args)?)),
			"complete_todo" => Ok(ToolCallParameters::CompleteTodo(serde_json::from_str(args)?)),
			"add_memory" => Ok(ToolCallParameters::AddMemory(serde_json::from_str(args)?)),
			"forget_memory" => Ok(ToolCallParameters::ForgetMemory(serde_json::from_str(args)?)),
			"list_folder_content" => Ok(ToolCallParameters::ListFolderContent(serde_json::from_str(args)?)),
			"find_in_file" => Ok(ToolCallParameters::FindInFile(serde_json::from_str(args)?)),
			_ => anyhow::bail!("Unknown tool: {}", name),
		}
	}
}