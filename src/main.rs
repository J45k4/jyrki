use env::load_envs;
use generated::TOOLS;
use llm::*;
use types::*;
use ui::*;
use utility::get_projects_dir;
use std::collections::HashSet;
use std::fs::read_dir;
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
}

impl App {
	pub fn new(projects: Vec<Project>) -> App {
		let state = State {
			projects,
			..Default::default()
		};

		App {
			wgui: Wgui::new("0.0.0.0:7765".parse().unwrap()),
			clients: HashSet::new(),
			state,
			llm_client: LLMClient::new(),
		}
	}

	async fn render_ui(&mut self) {
		let item = ui(&self.state);

		for client_id in &self.clients {
			self.wgui.render(*client_id, item.clone()).await;
		}
	}

	fn send_message(&mut self) {
		if self.state.current_msg.is_empty() {
			return;
		}
		let current_msg = self.state.current_msg.clone();
		self.state.current_msg.clear();
		let project = match self.get_active_project() {
			Some(project) => project,
			None => return,
		};
		project.modified = true;
		project.history.add_message(LLMMessage::User(current_msg));
		let mut messages = Vec::new();
		let mut assistant_msg = String::new();

		if !project.instructions.is_empty() {
			assistant_msg += &format!("Instructions: {}\n", project.instructions);
		}
		if !project.name.is_empty() {
			assistant_msg += &format!("Project name: {}\n", project.name);
		}

		if !project.instructions.is_empty() {
			let msg = LLMMessage::System(project.instructions.clone());
			messages.push(msg);
		}
		messages.extend_from_slice(&project.history.get_context());
		let req = GenRequest {
			model: project.model.clone(),
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
					if let Some(project) = self.get_active_project() {
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
				}
				SELECT_PROJECT_FOLDER => {
					match rfd::AsyncFileDialog::new().pick_folder().await {
						Some(handle) => {
							if let Some(project) = self.get_active_project() {
								project.modified = true;
								project.folder_path = handle.path().to_string_lossy().to_string();
							}
						},
						None => {
							log::info!("No folder selected");
						}
					}
				}
				NEW_PROJECT_BUTTON => {
					let project = Project {
						modified: true,
						..Default::default()
					};
					self.state.projects.push(project);
				}
				SAVE_PRJECT_BUTTON => {
					if let Some(project) = self.get_active_project() {
						project.modified = false;
						let save_path = get_projects_dir().join(format!("{}.json", project.name));
						let content = serde_json::to_string_pretty(project).unwrap();
						tokio::fs::write(save_path, content).await.unwrap();
					}
				}
				NEW_FORBIDDEN_FILE_BUTTON => {
					let new_forbidden_file_name = self.state.new_forbidden_file_name.clone();
					if let Some(project) = self.get_active_project() {
						project.forbidden_files.push(new_forbidden_file_name);
						project.modified = true;
					}
				}
				DELETE_FORBIDDEN_FILE_BUTTON => {
					if let Some(project) = self.get_active_project() {
						project.forbidden_files.remove(o.inx.unwrap() as usize);
						project.modified = true;
					}
				}
				_ => {}
			},
			ClientEvent::OnTextChanged(t) => match t.id {
				MESSAGE_INPUT => {
					self.state.current_msg = t.value;
				}
				PROJECT_NAME_INPUT => {
					if let Some(project) = self.get_active_project() {
						project.name = t.value;
						project.modified = true;
					}
				}
				INSTRUCTIONS_TEXT_INPUT => {
					if let Some(project) = self.get_active_project() {
						project.instructions = t.value;
						project.modified = true;
					}
				}
				NEW_FORBIDDEN_FILE_NAME => {
					self.state.new_forbidden_file_name = t.value;
				}
				_ => {}
			},
			ClientEvent::OnSelect(event) => {
				match event.id {
					MODEL_SELECT => {
						log::info!("model selected: {:?}", event.value);
						if let Some(project) = self.get_active_project() {
							project.model = LLMModel::from_str(&event.value).unwrap();
							project.modified = true;
						}
					}
					_ => {}
				}
			}
			_ => {}
		};

		self.render_ui().await;
	}

	async fn handle_result(&mut self, result: GenResult) {
		match result {
			GenResult::Response(res) => {
				log::info!("Response: {:?}", res);
				if let Some(project) = self.get_active_project() {
					project.history.add_message(LLMMessage::Assistant(res.msg.clone()));
					project.input_token_count += res.prompt_tokens;
					project.output_token_count += res.completion_tokens;
					project.input_token_cost += res.promt_cost;
					project.output_token_cost += res.completion_cost;
					project.modified = true;
	
					for tool_call in res.msg.tool_calls {
						match tool::execute(&project, tool_call.tool).await {
							Ok(res) => {
								log::info!("tool call result: {:?}", res);
								project.history.add_message(LLMMessage::ToolResponse(ToolResponse { 
									id: tool_call.id, 
									content: res
								}))
							}
							Err(e) => {
								log::info!("tool call error: {:?}", e);
								project.history.add_message(LLMMessage::ToolResponse(ToolResponse { 
									id: tool_call.id, 
									content: e.to_string()
								}))
							}
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
	let projects_path = get_projects_dir();
	let conent = read_dir(projects_path).unwrap();
	let projects: Vec<Project> = conent
		.filter_map(|entry| {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_file() {
				log::info!("Loading project: {:?}", path);
				let content = std::fs::read_to_string(path).unwrap();
				Some(serde_json::from_str(&content).unwrap())
			} else {
				None
			}
		})
		.collect();

	App::new(projects).run().await;
}
