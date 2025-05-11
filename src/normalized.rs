use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Core content block representation for normalized LLM responses.
/// 
/// This enum represents the different types of content that can appear in an LLM response,
/// normalized into a consistent format regardless of the original model provider.
/// 
/// # Examples
/// 
/// ```
/// use adaptogen::normalized::ContentBlock;
/// 
/// let text_block = ContentBlock::Text {
///     text: "Hello, world!".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Simple text content from the model
    #[serde(rename = "text")]
    Text { text: String },

    /// A tool/function call made by the model
    #[serde(rename = "tool_use")]
    ToolUse {
        /// Unique identifier for this tool use
        id: String,
        /// Name of the tool being used
        name: String,
        /// Input parameters for the tool call
        input: Value,
    },

    /// Results returned from a tool execution
    #[serde(rename = "tool_result")]
    ToolResult {
        /// ID of the corresponding tool use
        tool_use_id: String,
        /// Content blocks containing the tool result
        content: Vec<ContentResultBlock>,
        /// Whether the tool execution resulted in an error
        is_error: bool,
    },

    /// Internal reasoning/thinking from the model
    #[serde(rename = "thinking")]
    Thinking {
        /// The thinking/reasoning content
        thinking: Option<String>,
        /// Optional signature or metadata for the thinking block
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

/// Content result block for tool results
///
/// Represents a single block of content within a tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentResultBlock {
    /// The content of the result block
    pub content: String,
}

/// A ContentFrame represents a complete message from an LLM
///
/// This structure contains metadata about the message and a collection
/// of ContentBlock instances representing the actual content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFrame {
    /// Unique identifier for this content frame
    pub id: String,
    /// The model that generated this content
    pub model: String,
    /// The normalized content blocks that make up the message
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
