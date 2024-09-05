use env::load_envs;
use history::History;
use history::HistoryItemContent;
use llm::*;
use tool::Tool;
use tool::ToolExecutor;
use std::collections::HashSet;
use wgui::*;

mod llm;
mod openai;
mod env;
mod history;
mod tool;

pub const TOOLS_DATA: &str = include_str!("./tools.json");

// #[derive(Debug)]
// enum ConversationItem {
// 	ToolUses(Vec<Tool>),
// 	UserMessage(Vec<String>),
// 	AssistantMessage(String),
// }

#[derive(Debug)]
struct Project {
	output_token_count: u32,
	input_token_count: u32,
	input_token_cost: u32,
	output_token_cost: u32,
	todo_items: Vec<TodoItem>,
	// items: Vec<ConversationItem>,
	history: History,
	instructions: String,
}

#[derive(Debug)]
struct TodoItem {
	text: String,
	done: bool,
}

#[derive(Debug, Default)]
struct State {
	projects: Vec<Project>,
	disallowed_files: Vec<String>,
	active_project: Option<usize>,
	current_msg: String,
}

impl State {
	pub fn new() -> State {
		State::default()
	}
}

const SELECT_PROJECT_LINK: u32 = 1;
const SEND_MESSAGE_BUTTON: u32 = 2;
const MESSAGE_INPUT: u32 = 3;

fn todo_item_view(todo_item: &TodoItem) -> Item {
	hstack([
		text(&todo_item.text),
		text(if todo_item.done { "done" } else { "not done" }),
	])
	.spacing(10)
	.border("1px solid black")
	.padding(5)
}

fn format_cost(cost: u32) -> String {
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
				// match item {
				// 	ConversationItem::ToolUses(tools) => hstack(tools.iter().map(|tool| {
				// 		text(&format!("{:?}", tool))
				// 			.border("1px solid black")
				// 			.padding(5)
				// 	}))
				// 	.spacing(10),
				// 	ConversationItem::UserMessage(msgs) => {
				// 		vstack([text("User"), text(&msgs.join(" "))])
				// 			.spacing(10)
				// 			.padding(5)
				// 			.border("1px solid black")
				// 	}
				// 	ConversationItem::AssistantMessage(msgs) => {
				// 		vstack([text("Assistant"), text(&msgs)])
				// 			.spacing(10)
				// 			.padding(5)
				// 			.border("1px solid black")
				// 	}
				// }
				match &item.content {
					HistoryItemContent::UserMessage { content } => {
						vstack([text("User"), text(&content)])
							.spacing(10)
							.padding(5)
							.border("1px solid black")
					},
					HistoryItemContent::AssistantMessage { content } => {
						vstack([text("Assistant"), text(&content)])
							.spacing(10)
							.padding(5)
							.border("1px solid black")
					},
					HistoryItemContent::ToolCall { id, tool } => {
						match tool {
							Tool::WriteFile(w) => {
								vstack([
									text("Tool"),
									text(&format!("Write file {} with content {}", w.path, w.content)),
								])
								.spacing(10)
								.padding(5)
								.border("1px solid black")
							},
							_ => {
								vstack([text("Tool"), text(&format!("{:?}", tool))])
									.spacing(10)
									.padding(5)
									.border("1px solid black")
							}
							// Tool::ReadFile { path } => {
							// 	vstack([text("Tool"), text(&format!("Read file {}", path))])
							// 		.spacing(10)
							// 		.padding(5)
							// 		.border("1px solid black")
							// },
							// Tool::ListFolderContents { path } => {
							// 	vstack([text("Tool"), text(&format!("List folder contents {}", path))])
							// 		.spacing(10)
							// 		.padding(5)
							// 		.border("1px solid black")
							// },
						}
						// vstack([text("Tool"), text(&format!("{:?} {:?}", tool, args))])
						// 	.spacing(10)
						// 	.padding(5)
						// 	.border("1px solid black")
					},
				}
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

fn ui(state: &State) -> Item {
	vstack([
		projects_tabs(&state),
		state
			.active_project
			.map(|project| project_view(&state.projects[project]))
			.unwrap_or(text("no project selected")),
	])
	.spacing(10)
}

struct App {
	wgui: Wgui,
	clients: HashSet<usize>,
	state: State,
	llm_client: LLMClient,
	tools: serde_json::Value,
	executor: ToolExecutor,
}

impl App {
	pub fn new() -> App {
		let project = Project {
			input_token_cost: 0,
			output_token_cost: 0,
			input_token_count: 0,
			output_token_count: 0,
			instructions: "THIS is your instructions for the project".to_string(),
			todo_items: vec![
				TodoItem {
					text: "Do this thing".to_string(),
					done: false,
				},
				TodoItem {
					text: "Do that thing".to_string(),
					done: true,
				},
			],
			history: History::new(),
			// items: vec![
			//     ConversationItem::AssistantMessage(vec!["Hello".to_string()]),
			//     ConversationItem::UserMessage(vec!["Hi".to_string()]),
			//     ConversationItem::ToolUses(vec![
			//         Tool::WriteFile,
			//         Tool::ReadFile,
			//         Tool::ListFolderContents,
			//     ]),
			//     ConversationItem::AssistantMessage(vec!["Goodbye".to_string()]),
			// ],
		};

		let mut state = State::new();
		state.projects.push(project);

		let tools = serde_json::from_str(TOOLS_DATA).unwrap();

		App {
			wgui: Wgui::new("0.0.0.0:7765".parse().unwrap()),
			clients: HashSet::new(),
			state,
			llm_client: LLMClient::new(),
			tools,
			executor: ToolExecutor::new(),
		}
	}

	async fn render_ui(&mut self) {
		let item = ui(&self.state);

		for client_id in &self.clients {
			self.wgui.render(*client_id, item.clone()).await;
		}
	}

	async fn handle_event(&mut self, event: ClientEvent) {
		match event {
			ClientEvent::Disconnected { id } => {
				self.clients.remove(&id);
			}
			ClientEvent::Connected { id } => {
				self.clients.insert(id);
			}
			ClientEvent::OnClick(o) => match o.id {
				SELECT_PROJECT_LINK => {
					self.state.active_project = Some(0);
				}
				SEND_MESSAGE_BUTTON => {
					log::info!("Send message button clicked");
					let project = self.state.projects.get_mut(0).unwrap();
					project.history.add_user_msg(self.state.current_msg.clone());
					self.state.current_msg.clear();
					let messages = project.history.get_context();
					let req = GenRequest {
						model: GPT_4O_MINI.to_string(),
						messages,
						tools: self.tools.clone(),
					};

					self.llm_client.gen(req);
				}
				_ => {}
			},
			ClientEvent::OnTextChanged(t) => match t.id {
				MESSAGE_INPUT => {
					self.state.current_msg = t.value;
				}
				_ => {}
			},
			_ => {}
		};

		self.render_ui().await;
	}

	fn handle_result(&mut self, result: GenResult) {
		match result {
			GenResult::Response(res) => {
				log::info!("Response: {:?}", res);
				// let message = Message::Assistant { content: res.content };
				let project = self.state.projects.get_mut(0).unwrap();
				if !res.content.is_empty() {
					project.history.add_assistant_msg(res.content.clone());
				}
				project.input_token_count += res.prompt_tokens;
				project.output_token_count += res.completion_tokens;

				for t in res.tools {
					log::info!("Tool: {:?}", t);
					match t.name.as_str() {
						"write_file" => {
							let w = serde_json::from_str(&t.args).unwrap();
							let tool = Tool::WriteFile(w);
							project.history.add_tool_call(t.id.clone(), tool.clone());
							self.executor.execute(t.id, tool);
						}
						_ => {}
					}
				}
			},
			GenResult::Error(e) => {
				log::info!("Error: {:?}", e);
			},
		}
	}

	async fn run(mut self) {
		loop {
			tokio::select! {
				event = self.wgui.next() => {
					match event {
						Some(e) => {
							log::info!("Event: {:?}", e);
							self.handle_event(e).await;
						},
						None => {
							log::info!("No event");
							break;
						},
					}
				}
				result = self.llm_client.next() => {
					match result {
						Some(res) => {
							log::info!("Result: {:?}", res);
							self.handle_result(res);
						},
						None => {
							log::info!("No result");
							break;
						},
					}
				}
			}
			self.render_ui().await;
		}
	}
}

#[tokio::main]
async fn main() {
	simple_logger::init_with_level(log::Level::Info).unwrap();
	load_envs();
	App::new().run().await;
}
