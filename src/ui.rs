use wgui::Item;
use wgui::*;
use crate::tool::Tool;
use crate::types::Project;
use crate::types::State;
use crate::types::TodoItem;
use crate::LLMMessage;

pub const SELECT_PROJECT_LINK: u32 = 1;
pub const SEND_MESSAGE_BUTTON: u32 = 2;
pub const MESSAGE_INPUT: u32 = 3;

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

fn send_message_view() -> Item {
	hstack([
		text_input().placeholder("Message").grow(1).id(MESSAGE_INPUT),
		button("Send").id(SEND_MESSAGE_BUTTON),
	])
	.spacing(5)
	.height(35)
}

fn project_view(project: &Project) -> Item {
	hstack([
		vstack([
			hstack([
				text_input()
					.placeholder("Instructions")
					.svalue(&project.instructions)
					.grow(1),
				select([
					option("gpt-4o-mini", "gpt-4o-mini"),
					option("gpt-4o", "gpt-4o"),
				]),
			]).spacing(5),
			vstack(project.history.items.iter().map(|item| {
				hstack([
					match &item.content {
						LLMMessage::User(content) => {
							vstack([text("User"), text(&content)])
								.spacing(10)
								.padding(5)
								.border("1px solid black")
						}
						LLMMessage::System(content) => {
							vstack([text("System"), text(&content)])
								.spacing(10)
								.padding(5)
								.border("1px solid black")
						},
						LLMMessage::Assistant(msg) => {
							vstack([
								text("Assistant"),
								text(&msg.content),
								hstack(msg.tool_calls.iter().map(|tool_call| {
									vstack([
										text(&tool_call.id),
										text(&format!("{:?}", tool_call.tool)),
									])
									.spacing(10)
								})),
							]).spacing(10).border("1px solid black")
						},
						_ => text("Unknown message type"),
					}
				])
			}))
			.spacing(15),
			send_message_view(),
		])
		.spacing(10)
		.grow(1),
		vstack([tokens_view(project), todo_list_view(&project.todo_items)]).spacing(10),
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
	hstack([nav_item("project 1")]).spacing(10)
}

pub fn ui(state: &State) -> Item {
	vstack([
		projects_tabs(&state),
		state
			.active_project
			.map(|project| project_view(&state.projects[project]))
			.unwrap_or(text("no project selected")),
	])
	.spacing(10)
}