use crate::args::DataFormat;
use prost::Message as ProstMessage;
use serde::{Deserialize, Serialize};

// Include the generated protobuf code
pub mod chat {
    include!(concat!(env!("OUT_DIR"), "/chat.rs"));
}

use chat::ChatMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMessage {
    pub username: String,
    pub message: String,
    pub replaces_offset: Option<i64>,  // Offset of the message this replaces (None = new message)
    pub delete_offset: Option<i64>,    // Offset of the message to delete (None = no deletion)
}

pub struct MessageHandler;

impl MessageHandler {
    pub fn serialize_message(
        username: &str,
        message: &str,
        format: DataFormat,
        replaces_offset: Option<i64>,
        delete_offset: Option<i64>,
    ) -> anyhow::Result<Vec<u8>> {
        match format {
            DataFormat::Text => {
                let text_message = format!("{}: {}", username, message);
                Ok(text_message.into_bytes())
            }
            DataFormat::Json => {
                let json_message = JsonMessage {
                    username: username.to_string(),
                    message: message.to_string(),
                    replaces_offset,
                    delete_offset,
                };
                let json_string = serde_json::to_string(&json_message)?;
                Ok(json_string.into_bytes())
            }
            DataFormat::Protobuf => {
                let proto_message = ChatMessage {
                    username: username.to_string(),
                    message: message.to_string(),
                    replaces_offset: replaces_offset.unwrap_or(0),
                    delete_offset: delete_offset.unwrap_or(0),
                };
                Ok(proto_message.encode_to_vec())
            }
        }
    }

    pub fn deserialize_message(
        data: &[u8],
        format: DataFormat,
    ) -> anyhow::Result<(String, String)> {
        match format {
            DataFormat::Text => {
                let text = String::from_utf8(data.to_vec())?;
                // Simple parsing for "username: message" format
                if let Some(colon_pos) = text.find(':') {
                    let username = text[..colon_pos].trim();
                    let message = text[colon_pos + 1..].trim();
                    Ok((username.to_string(), message.to_string()))
                } else {
                    Err(anyhow::anyhow!("Invalid text format"))
                }
            }
            DataFormat::Json => {
                let json_message: JsonMessage = serde_json::from_slice(data)?;
                Ok((json_message.username, json_message.message))
            }
            DataFormat::Protobuf => {
                let proto_message = ChatMessage::decode(data)?;
                Ok((proto_message.username, proto_message.message))
            }
        }
    }
}
