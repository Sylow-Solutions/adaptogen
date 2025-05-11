use serde::{Deserialize, Serialize};
use serde_json::Value;
/// Core content block representation for normalized LLM responses
/// TODO: Right now we don't give users of this library a choice of the
/// "normalized" format. For this to be useful outside of sylow, we should
/// allow users to define their target format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: Vec<ContentResultBlock>,
        is_error: bool,
    },

    #[serde(rename = "thinking")]
    Thinking {
        thinking: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

/// Content result block for tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResultBlock {
    pub content: String,
}

/// A ContentFrame represents a complete message from an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFrame {
    pub id: String,
    pub model: String,
    pub blocks: Vec<ContentBlock>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_block_text_serialization() {
        let text_block = ContentBlock::Text {
            text: "Hello, world!".to_string(),
        };
        
        let serialized = serde_json::to_string(&text_block).unwrap();
        let expected = r#"{"type":"text","text":"Hello, world!"}"#;
        
        assert_eq!(serialized, expected);
        
        let deserialized: ContentBlock = serde_json::from_str(expected).unwrap();
        match deserialized {
            ContentBlock::Text { text } => assert_eq!(text, "Hello, world!"),
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[test]
    fn test_content_block_tool_use_serialization() {
        let tool_use_block = ContentBlock::ToolUse {
            id: "123".to_string(),
            name: "calculator".to_string(),
            input: json!({"expression": "2+2"}),
        };
        
        let serialized = serde_json::to_string(&tool_use_block).unwrap();
        let deserialized: ContentBlock = serde_json::from_str(&serialized).unwrap();
        
        match deserialized {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "123");
                assert_eq!(name, "calculator");
                assert_eq!(input["expression"], "2+2");
            },
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[test]
    fn test_content_frame() {
        let frame = ContentFrame {
            id: "msg_123".to_string(),
            model: "test-model".to_string(),
            blocks: vec![
                ContentBlock::Text { text: "Hello".to_string() },
                ContentBlock::Thinking { 
                    thinking: Some("Some thinking".to_string()),
                    signature: None,
                },
            ],
        };
        
        assert_eq!(frame.id, "msg_123");
        assert_eq!(frame.model, "test-model");
        assert_eq!(frame.blocks.len(), 2);
        
        let serialized = serde_json::to_string(&frame).unwrap();
        let deserialized: ContentFrame = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.id, frame.id);
        assert_eq!(deserialized.model, frame.model);
        assert_eq!(deserialized.blocks.len(), frame.blocks.len());
    }
}
