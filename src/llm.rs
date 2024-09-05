use serde_json::Value;
use tokio::sync::mpsc::{Receiver, UnboundedReceiver, UnboundedSender};

use crate::openai;

pub const GPT_4O: &str = "gpt-4o";
pub const GPT_4O_MINI: &str = "gpt-4o-mini";

#[derive(Debug, serde::Serialize)]
#[serde(tag = "role")]
pub enum Message {
	#[serde(rename = "system")]
    System {
        content: String,
    },
	#[serde(rename = "user")]
    User {
        content: String,
    },
	#[serde(rename = "assistant")]
    Assistant {
        content: String,
    },
	#[serde(rename = "tool")]
    Tool {
        tool_call_id: String,
        content: String,
    },
}

#[derive(Debug, serde::Serialize)]
pub struct GenRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub tools: Value
}

#[derive(Debug, serde::Deserialize)]
pub struct ToolUse {
	pub id: String,
    pub name: String,
    pub args: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SuccessfullGenResponse {
    pub content: String,
	pub prompt_tokens: u32,
	pub completion_tokens: u32,
	pub total_tokens: u32,
    pub tools: Vec<ToolUse>,
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
            let res = match req.model.as_str() {
                GPT_4O | GPT_4O_MINI => openai::gen(req, client).await,
                _ => panic!("Invalid model {}", req.model),
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
