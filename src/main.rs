use llm::*;
use std::collections::HashSet;
use wgui::*;

mod llm;
mod openai;

pub const TOOLS_DATA: &str = include_str!("./tools.json");

#[derive(Debug)]
enum Tool {
    WriteFile,
    ReadFile,
    ListFolderContents,
    CreateTodoItem,
    CompleteTodoItem,
    FindText { text: String, path: String },
}

#[derive(Debug)]
enum ConversationItem {
    ToolUses(Vec<Tool>),
    UserMessage(Vec<String>),
    AssistantMessage(Vec<String>),
}

#[derive(Debug)]
struct Project {
    output_token_count: u32,
    input_token_count: u32,
    input_token_cost: u32,
    output_token_cost: u32,
    todo_items: Vec<TodoItem>,
    items: Vec<ConversationItem>,
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
        text_input().placeholder("Message").grow(1),
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
            ]),
            vstack(project.items.iter().map(|item| {
                match item {
                    ConversationItem::ToolUses(tools) => hstack(tools.iter().map(|tool| {
                        text(&format!("{:?}", tool))
                            .border("1px solid black")
                            .padding(5)
                    }))
                    .spacing(10),
                    ConversationItem::UserMessage(msgs) => {
                        vstack([text("User"), text(&msgs.join(" "))])
                            .spacing(10)
                            .padding(5)
                            .border("1px solid black")
                    }
                    ConversationItem::AssistantMessage(msgs) => {
                        vstack([text("Assistant"), text(&msgs.join(" "))])
                            .spacing(10)
                            .padding(5)
                            .border("1px solid black")
                    }
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
}

impl App {
    pub fn new() -> App {
        let project = Project {
            input_token_cost: 150,
            output_token_cost: 380,
            input_token_count: 1000,
            output_token_count: 5000,
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
            items: vec![
                ConversationItem::AssistantMessage(vec!["Hello".to_string()]),
                ConversationItem::UserMessage(vec!["Hi".to_string()]),
                ConversationItem::ToolUses(vec![
                    Tool::WriteFile,
                    Tool::ReadFile,
                    Tool::ListFolderContents,
                ]),
                ConversationItem::AssistantMessage(vec!["Goodbye".to_string()]),
            ],
        };

        let mut state = State::new();
        state.projects.push(project);

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
                    // let req = GenRequest {

                    // }
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
            }
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    App::new().run().await;
}
