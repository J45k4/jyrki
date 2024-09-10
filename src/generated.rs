
#[derive(debug)]
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
	pub fn to_value(&self) -> Value {
		match self {
			Tool::ReadFile => Value::String("read_file".to_string()),
			Tool::WriteFile => Value::String("write_file".to_string()),
			Tool::RemoveFile => Value::String("remove_file".to_string()),
			Tool::AddNewTodo => Value::String("add_new_todo".to_string()),
			Tool::CompleteTodo => Value::String("complete_todo".to_string()),
			Tool::AddMemory => Value::String("add_memory".to_string()),
			Tool::ForgetMemory => Value::String("forget_memory".to_string()),
			Tool::ListFolderContent => Value::String("list_folder_content".to_string()),
			Tool::FindInFile => Value::String("find_in_file".to_string()),
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
#[derive(Debug, Clone)]
pub struct ReadFile {
	pub linenumber_count: u32,
	pub path: String,
	pub start_line_number: u32,
}

#[derive(Debug, Clone)]
pub struct WriteFile {
	pub content: String,
	pub linenumber: u32,
	pub path: String,
}

#[derive(Debug, Clone)]
pub struct RemoveFile {
	pub path: String,
}

#[derive(Debug, Clone)]
pub struct AddNewTodo {
	pub content: String,
	pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompleteTodo {
	pub name: String,
}

#[derive(Debug, Clone)]
pub struct AddMemory {
	pub content: String,
	pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ForgetMemory {
	pub name: String,
}

#[derive(Debug, Clone)]
pub struct ListFolderContent {
	pub path: String,
}

#[derive(Debug, Clone)]
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