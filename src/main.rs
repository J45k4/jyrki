use env::load_envs;
use history::History;
use llm::*;
use tool::ToolExecutor;
use types::Project;
use types::State;
use types::TodoItem;
use types::ToolDef;
use ui::ui;
use ui::MESSAGE_INPUT;
use ui::SELECT_PROJECT_LINK;
use ui::SEND_MESSAGE_BUTTON;
use std::collections::HashSet;
use wgui::*;

mod llm;
mod openai;
mod env;
mod history;
mod tool;
mod ui;
mod types;
mod generated;

//pub const TOOLS_DATA: &str = include_str!("./tools.json");


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
		let mut project = Project {
			input_token_cost: 0.0,
			output_token_cost: 0.0,
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
			current_msg: "".to_string(),
			history: History::new(),
			activated_tools: vec!["write_file".to_string(), "read_file".to_string()],
			disallowed_files: vec!["secret.txt".to_string()],
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

		for i in 0..200 {
			project.history.add_message(LLMMessage::User(format!("Hi {}", i)));
		}

		let mut state = State::new();
		state.projects.push(project);

		// let tooldef = ToolDef::Function { 
		// 	name: "read_file".to_string(), 
		// 	description: "Reads a file".to_string(), 
		// 	parameters: 
		// };

		let tools = serde_json::from_str(TOOLS_DATA).unwrap();
		open::that("http://localhost:7765").unwrap();

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
					project.history.add_message(LLMMessage::User(self.state.current_msg.clone()));
					self.state.current_msg.clear();
					let messages = project.history.get_context();
					let req = GenRequest {
						model: Model::GPT4OMini,
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

	async fn handle_result(&mut self, result: GenResult) {
		match result {
			GenResult::Response(res) => {
				log::info!("Response: {:?}", res);
				let project = self.state.projects.get_mut(0).unwrap();
				project.history.add_message(LLMMessage::Assistant(res.msg.clone()));
				project.input_token_count += res.prompt_tokens;
				project.output_token_count += res.completion_tokens;
				project.input_token_cost += res.promt_cost;
				project.output_token_cost += res.completion_cost;

				for tool_call in res.msg.tool_calls {
					match self.executor.execute(tool_call.tool).await {
						Ok(res) => {
							log::info!("Result: {:?}", res);
							project.history.add_message(LLMMessage::ToolResponse(ToolResponse { 
								id: tool_call.id, 
								content: res
							}))
						}
						Err(e) => {
							log::info!("Error: {:?}", e);
							project.history.add_message(LLMMessage::ToolResponse(ToolResponse { 
								id: tool_call.id, 
								content: e.to_string()
							}))
						}
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
							self.handle_result(res).await;
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
