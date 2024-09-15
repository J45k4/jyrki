use wgui::Item;
use wgui::*;
use crate::generated::ToolCallParameters;
use crate::generated::TOOLS;
use crate::types::Project;
use crate::types::State;
use crate::types::TodoItem;
use crate::LLMMessage;
use crate::ToolCall;

pub const SELECT_PROJECT_LINK: u32 = 1;
pub const SEND_MESSAGE_BUTTON: u32 = 2;
pub const MESSAGE_INPUT: u32 = 3;
pub const TOOL_CHECKBOX: u32 = 4;
pub const SELECT_PROJECT_FOLDER: u32 = 5;
pub const NEW_PROJECT_BUTTON: u32 = 6;
pub const PROJECT_NAME_INPUT: u32 = 7;
pub const SAVE_PRJECT_BUTTON: u32 = 8;
pub const INSTRUCTIONS_TEXT_INPUT: u32 = 9;
pub const MODEL_SELECT: u32 = 10;

fn todo_item_view(todo_item: &TodoItem) -> Item {
	hstack([
		text(&todo_item.text),
		text(if todo_item.done { "done" } else { "not done" }),
	])
	.spacing(10)
	.border("1px solid black")
	.padding(5)
}

fn format_cost(cost: f32) -> String {
	format!("{}€", cost as f32 / 100.0)
}

fn tokens_view(project: &Project) -> Item {
	hstack([
		vstack([
			text("Input"),
			text(&format!("{}", project.input_token_count)),
			text("Output"),
			text(&format!("{}", project.output_token_count)),
			text("Total"),
			text(&format!(
				"{}",
				project.input_token_count + project.output_token_count
			)),
		])
		.grow(1),
		vstack([
			text("Cost"),
			text(&format_cost(project.input_token_cost)),
			text("Cost"),
			text(&format_cost(project.output_token_cost)),
			text("Cost"),
			text(&format_cost(
				project.input_token_cost + project.output_token_cost,
			)),
		]),
	])
	.border("1px solid black")
	.padding(5)
	.spacing(15)
}

fn todo_list_view(todo_items: &Vec<TodoItem>) -> Item {
	vstack(todo_items.iter().map(todo_item_view)).spacing(10)
}

fn tools_list_view(project: &Project) -> Item {
	vstack([
		text("Tools"),
		vstack(TOOLS.iter().enumerate().map(|(inx, tool)| {
			hstack([
				checkbox()
					.checked(project.activated_tools.contains(tool))
					.id(TOOL_CHECKBOX)
					.inx(inx as u32),
				text(&tool.get_name()),
			])
		})),
	])
	.border("1px solid black")
	.spacing(10)
	.padding(5)
}

fn forbidden_files(project: &Project) -> Item {
	vstack([
		text("Forbidden files"),
		vstack(project.disallowed_files.iter().map(|file| text(file))),
	])
	.border("1px solid black")
	.spacing(10)
}

fn send_message_view(msg: &str) -> Item {
	hstack([
		text_input().placeholder("Message").grow(1).id(MESSAGE_INPUT).svalue(msg),
		button("Send").id(SEND_MESSAGE_BUTTON),
	])
	.spacing(5)
	.height(35)
}

fn multile_text(t: &str) -> Item {
	vstack(t.split("\n").map(|line| text(line))).spacing(5)
}

fn tool_call_view(tool_call: &ToolCall) -> Item {
	vstack([
		text(&tool_call.id),
		match &tool_call.tool {
			ToolCallParameters::WriteFile(w) => {
				vstack([
					text("WriteFile"),
					text(&format!("path: {}", w.path)),
					text(&format!("line number: {}", w.linenumber)),
					multile_text(&w.content),
				])
			},
			ToolCallParameters::ReadFile(r) => {
				vstack([
					text("ReadFile"),
					text(&format!("path: {}", r.path)),
					text(&format!("start line number: {}", r.start_line_number)),
					text(&format!("line number count: {}", r.linenumber_count)),
				])
			},
			ToolCallParameters::RemoveFile(r) => {
				vstack([
					text("RemoveFile"),
					text(&r.path),
				])
			},
			ToolCallParameters::ListFolderContent(l) => {
				vstack([
					text("ListFolderContents"),
					text(&l.path),
				])
			},
			_ => text("Unknown tool"),
		},
	])
	.spacing(10)
}



fn project_view(project: &Project, state: &State) -> Item {
	hstack([
		vstack([
			hstack([
				text_input()
					.placeholder("Instructions")
					.svalue(&project.instructions)
					.id(INSTRUCTIONS_TEXT_INPUT)
					.grow(1),
				select([
					option("gpt-4o-mini", "gpt-4o-mini"),
					option("gpt-4o", "gpt-4o"),
				]).svalue(project.model.to_str()).id(MODEL_SELECT),
			]).spacing(5).height(35),
			send_message_view(&state.current_msg),
			vstack(project.history.items.iter().rev().map(|item| {
				hstack([
					match &item.content {
						LLMMessage::User(content) => {
							vstack([text("User"), text(&content)])
								.spacing(10)				
						}
						LLMMessage::System(content) => {
							vstack([text("System"), text(&content)])
								.spacing(10)
						},
						LLMMessage::Assistant(msg) => {
							hstack([
								vstack([
									text("Assistant"),
									text(&msg.content),
								]),
								vstack(msg.tool_calls.iter().map(|tool_call| tool_call_view(tool_call))),
							]).spacing(10)
						},
						_ => text("Unknown message type"),
					}
				])
				.padding(5)
				.border("1px solid black")
			}))
			.spacing(15)
			.grow(1)
			.overflow("auto")
		])
		.spacing(10)
		.grow(1),
		vstack([
			if project.modified {
				button("Save").id(SAVE_PRJECT_BUTTON)
			} else {
				text("Saved")
			},
			tokens_view(project),
			vstack([
				text("Info"),
				hstack([
					text("Name: "),
					text_input().svalue(&project.name).id(PROJECT_NAME_INPUT),
				]).spacing(10)
			]).border("1px solid black").padding(5),
			vstack([
				text("Project folder"),
				text(&project.folder_path),
				button("Select").id(SELECT_PROJECT_FOLDER),
			]).border("1px solid black").padding(5),
			tools_list_view(project), 
			forbidden_files(project),
			todo_list_view(&project.todo_items)
		]).spacing(10),
	])
	.spacing(10)
}

fn nav_item(t: &str) -> Item {
	text(t)
		.padding(10)
		.background_color("#f0f0f0")
		.cursor("pointer")
		.id(SELECT_PROJECT_LINK)
}

fn projects_tabs(state: &State) -> Item {
	hstack([
		hstack(
			state.projects.iter().enumerate().map(|(inx, project)| {
				let modified = if project.modified { "*" } else { "" };
				nav_item(&format!("{} {}", project.name, modified))
					.inx(inx as u32)
			})
		).spacing(10),
		button("New").id(NEW_PROJECT_BUTTON)
	]).spacing(10).overflow("auto")
}

pub fn ui(state: &State) -> Item {
	vstack([
		projects_tabs(&state),
		state
			.active_project
			.map(|project| project_view(&state.projects[project], state))
			.unwrap_or(text("no project selected")),
	])
	.spacing(10)
	.margin(10)
	// .height(500)
	.overflow("hidden")
}