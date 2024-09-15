use chrono::DateTime;
use chrono::Utc;
use crate::LLMMessage;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HistoryItem {
	pub timestamp: DateTime<Utc>,
	pub content: LLMMessage,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct History {
	pub items: Vec<HistoryItem>,
}

impl History {
	pub fn new() -> History {
		History {
			items: Vec::new(),
		}
	}

	pub fn add_message(&mut self, message: LLMMessage) {
		self.items.push(HistoryItem {
			timestamp: Utc::now(),
			content: message,
		});
	}

	pub fn get_context(&self) -> Vec<LLMMessage> {
		self.items.iter().map(|item| item.content.clone()).collect()
	}
}