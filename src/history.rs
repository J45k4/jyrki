use chrono::DateTime;
use chrono::Utc;

use crate::tool::Tool;
use crate::Message;


#[derive(Debug)]
pub struct UserMessage {

}

#[derive(Debug)]
pub struct AssistantMessage {

}

#[derive(Debug)]
pub struct ToolCall {

}

#[derive(Debug)]
pub struct ToolResponse {

}

#[derive(Debug)]
pub enum HistoryItemContent {
	UserMessage {
		content: String,
	},
	AssistantMessage {
		content: String,
	},
	ToolCall {
		id: String,
		tool: Tool
	}
}

#[derive(Debug)]
pub struct HistoryItem {
	pub timestamp: DateTime<Utc>,
	pub content: HistoryItemContent,
}

#[derive(Debug)]
pub struct History {
	pub items: Vec<HistoryItem>,
}

impl History {
	pub fn new() -> History {
		History {
			items: Vec::new(),
		}
	}

	pub fn add_user_msg(&mut self, msg: String) {
		self.items.push(HistoryItem {
			timestamp: Utc::now(),
			content: HistoryItemContent::UserMessage {
				content: msg,
			},
		});
	}

	pub fn add_assistant_msg(&mut self, msg: String) {
		self.items.push(HistoryItem {
			timestamp: Utc::now(),
			content: HistoryItemContent::AssistantMessage {
				content: msg,
			},
		});
	}

	pub fn add_tool_call(&mut self, id: String, tool: Tool) {
		self.items.push(HistoryItem {
			timestamp: Utc::now(),
			content: HistoryItemContent::ToolCall {
				id,
				tool,
			},
		});
	}

	// pub fn add_tool_response(&mut self, response: ToolResponse) {
	// 	self.items.push(HistoryItem::ToolResponse(response));
	// }

	pub fn get_context(&self) -> Vec<Message> {
		let mut context = Vec::new();
		for item in &self.items {
			match &item.content {
				HistoryItemContent::UserMessage { content } => {
					context.push(Message::User {
						content: content.clone(),
					});
				},
				HistoryItemContent::AssistantMessage { content } => {
					context.push(Message::Assistant {
						content: content.clone(),
					});
				},
				HistoryItemContent::ToolCall { id, tool } => {
					let content = serde_json::to_string(&tool).unwrap();
					context.push(Message::Tool {
						tool_call_id: id.clone(),
						content
					});
				},
			}
		}
		context
	}
}