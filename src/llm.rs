use serde_json::Value;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use crate::generated::*;
use crate::openai;

pub const GPT_4O: &str = "gpt-4o";
pub const GPT_4O_MINI: &str = "gpt-4o-mini";

#[derive(Debug, Clone)]
pub struct ToolCall {
	pub id: String,
	pub tool: ToolCallParameters
}

#[derive(Debug, Clone)]
pub struct AssistantMsg {
	pub content: String,
	pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone)]
pub struct ToolResponse {
	pub id: String,
	pub content: String
}

const GPT_40_INPUT_COST: f32 = 2.5 / 1_000_000.0;
const GPT_40_OUTPUT_COST: f32 = 10.0 / 1_000_000.0;
const GPT_40_MINI_INPUT_COST: f32 = 0.150 / 1_000_000.0;
const GPT_40_MINI_OUTPUT_COST: f32 = 0.600 / 1_000_000.0;

#[derive(Debug)]
pub enum Model {
	GPT4O,
	GPT4OMini,
}

impl Model {
	pub fn input_cost(&self, token_count: u32) -> f32 {
		match self {
			Model::GPT4O => token_count as f32 * GPT_40_INPUT_COST,
			Model::GPT4OMini => token_count as f32 * GPT_40_MINI_INPUT_COST,
		}
	}

	pub fn output_cost(&self, token_count: u32) -> f32 {
		match self {
			Model::GPT4O => token_count as f32 * GPT_40_OUTPUT_COST,
			Model::GPT4OMini => token_count as f32 * GPT_40_MINI_OUTPUT_COST,
		}
	}
}

impl Model {
	pub fn to_str(&self) -> &str {
		match self {
			Model::GPT4O => GPT_4O,
			Model::GPT4OMini => GPT_4O_MINI,
		}
	}
}

#[derive(Debug, Clone)]
pub enum LLMMessage {
    System(String),
    User(String),
    Assistant(AssistantMsg),
    ToolResponse(ToolResponse),
}

#[derive(Debug)]
pub struct GenRequest {
    pub model: Model,
    pub messages: Vec<LLMMessage>,
    pub tools: Vec<Tool>,
}

#[derive(Debug)]
pub struct ToolUse {
	pub id: String,
    pub name: String,
    pub args: String,
}

#[derive(Debug)]
pub struct SuccessfullGenResponse {
	pub prompt_tokens: u32,
	pub completion_tokens: u32,
	pub total_tokens: u32,
	pub promt_cost: f32,
	pub completion_cost: f32,
    pub msg: AssistantMsg
}

#[derive(Debug)]
pub enum GenResult {
    Response(SuccessfullGenResponse),
    Error(String),
}

pub struct LLMClient {
    client: reqwest::Client,
    tx: UnboundedSender<GenResult>,
    rx: UnboundedReceiver<GenResult>,
}

impl LLMClient {
    pub fn new() -> LLMClient {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        LLMClient {
            client: reqwest::Client::new(),
            tx,
            rx,
        }
    }

    pub fn gen(&mut self, req: GenRequest) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
			let res = match req.model {
				Model::GPT4O | Model::GPT4OMini => openai::gen(req, client).await,
			};

            match res {
                Ok(res) => tx.send(GenResult::Response(res)).unwrap(),
                Err(err) => {
                    log::error!("gen failed: {:?}", err);
                    tx.send(GenResult::Error(err.to_string())).unwrap();
                }
            };
        });
    }

    pub async fn next(&mut self) -> Option<GenResult> {
        self.rx.recv().await
    }
}
