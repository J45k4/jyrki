use env::load_envs;
use generated::TOOLS;
use llm::*;
use tool::ToolExecutor;
use types::*;
use ui::*;
use utility::get_app_dir;
use utility::get_projects_dir;
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
mod utility;

struct App {
	wgui: Wgui,
	clients: HashSet<usize>,
	state: State,
	llm_client: LLMClient,
	executor: ToolExecutor,
}

impl App {
	pub fn new() -> App {
		let project = Project::default();

		let mut state = State::new();
		state.projects.push(project);

		App {
			wgui: Wgui::new("0.0.0.0:7765".parse().unwrap()),
			clients: HashSet::new(),
			state,
			llm_client: LLMClient::new(),
			executor: ToolExecutor::new(),
		}
	}

	async fn render_ui(&mut self) {
		let item = ui(&self.state);

		for client_id in &self.clients {
			self.wgui.render(*client_id, item.clone()).await;
		}
	}

	fn send_message(&mut self) {
		let project = self.state.projects.get_mut(0).unwrap();
		project.history.add_message(LLMMessage::User(self.state.current_msg.clone()));
		self.state.current_msg.clear();
		let messages = project.history.get_context();
		let req = GenRequest {
			model: Model::GPT4OMini,
			messages,
			tools: TOOLS.iter()
				.filter(|tool| project.activated_tools.contains(tool))
				.cloned().collect(),
		};

		self.llm_client.gen(req);
	}

	fn get_active_project(&mut self) -> Option<&mut Project> {
		let active_project = match self.state.active_project {
			Some(inx) => inx,
			None => return None,
		};

		self.state.projects.get_mut(active_project)
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
					self.state.active_project = Some(o.inx.unwrap() as usize);
				}
				SEND_MESSAGE_BUTTON => {
					log::info!("Send message button clicked");
					self.send_message();
				}
				TOOL_CHECKBOX => {
					let project = self.state.projects.get_mut(0).unwrap();
					let inx = o.inx.unwrap() as usize;
					match project.activated_tools.iter().position(|tool| tool == &TOOLS[inx]) {
						Some(i) => {
							project.activated_tools.remove(i);
						}
						None => {
							project.activated_tools.push(TOOLS[inx].clone());
						}
					}
				}
				SELECT_PROJECT_FOLDER => {
					match rfd::AsyncFileDialog::new().pick_folder().await {
						Some(handle) => {
							let project = self.state.projects.get_mut(0).unwrap();
							project.folder_path = handle.path().to_string_lossy().to_string();
						},
						None => {
							log::info!("No folder selected");
						}
					}
				}
				NEW_PROJECT_BUTTON => {
					let project = Project::default();
					self.state.projects.push(project);
				}
				SAVE_PRJECT_BUTTON => {
					if let Some(project) = self.get_active_project() {
						let save_path = get_projects_dir().join(format!("{}.json", project.name));
						let content = serde_json::to_string_pretty(project).unwrap();
						tokio::fs::write(save_path, content).await.unwrap();
						project.modified = false;
					}
				}
				_ => {}
			},
			ClientEvent::OnTextChanged(t) => match t.id {
				MESSAGE_INPUT => {
					self.state.current_msg = t.value;
				}
				PROJECT_NAME_INPUT => {
					if let Some(active_project) = self.state.active_project {
						let project = self.state.projects.get_mut(active_project).unwrap();
						project.name = t.value;
					}
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
