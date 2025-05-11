use adaptogen::normalized::{ContentBlock, ContentFrame};
use adaptogen::parser::{ModelResponseParser, ParseError};
use adaptogen::registry::ParserRegistry;

use std::sync::Arc;
use serde_json::Value;

// Custom parser for testing
struct TestClaudeParser;

impl ModelResponseParser for TestClaudeParser {
    fn supported_models(&self) -> Vec<String> {
        vec!["claude".to_string()]
    }

    fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
        let json: Value = serde_json::from_str(raw_response)?;

        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("id".to_string()))?
            .to_string();

        let model = json
            .get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("model".to_string()))?
            .to_string();

        let mut blocks = Vec::new();

        if let Some(content) = json.get("content") {
            if let Some(content_blocks) = content.as_array() {
                for block in content_blocks {
                    if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                        match block_type {
                            "text" => {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    blocks.push(ContentBlock::Text {
                                        text: text.to_string(),
                                    });
                                }
                            }
                            _ => {} // Ignore other types
                        }
                    }
                }
            }
        }

        Ok(ContentFrame { id, model, blocks })
    }
}

// Custom parser for testing
struct TestQwenParser;

impl ModelResponseParser for TestQwenParser {
    fn supported_models(&self) -> Vec<String> {
        vec!["qwen".to_string()]
    }

    fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
        let json: Value = serde_json::from_str(raw_response)?;

        // Extract basic metadata
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("id".to_string()))?
            .to_string();

        let model = json
            .get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("model".to_string()))?
            .to_string();

        let mut blocks = Vec::new();

        // Extract content blocks based on the Qwen format
        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            if let Some(first_choice) = choices.first() {
                if let Some(message) = first_choice.get("message") {
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        blocks.push(ContentBlock::Text {
                            text: content.to_string(),
                        });
                    }
                }
            }
        }

        Ok(ContentFrame { id, model, blocks })
    }
}

#[test]
fn test_registry_with_multiple_parsers() {
    // Create registry with both parsers
    let mut registry = ParserRegistry::new();
    registry.register_parser(Arc::new(TestClaudeParser));
    registry.register_parser(Arc::new(TestQwenParser));
    
    // Claude response
    let claude_response = r#"{
        "id": "claude-123",
        "model": "claude",
        "content": [
            {"type": "text", "text": "Hello from Claude!"}
        ]
    }"#;
    
    // Parse Claude response
    let result = registry.parse(claude_response);
    assert!(result.is_ok());
    
    let frame = result.unwrap();
    assert_eq!(frame.id, "claude-123");
    assert_eq!(frame.model, "claude");
    assert_eq!(frame.blocks.len(), 1);
    
    if let ContentBlock::Text { text } = &frame.blocks[0] {
        assert_eq!(text, "Hello from Claude!");
    } else {
        panic!("Expected Text block");
    }
    
    // Qwen response
    let qwen_response = r#"{
        "id": "qwen-456",
        "model": "qwen",
        "choices": [
            {
                "message": {
                    "content": "Hello from Qwen!"
                }
            }
        ]
    }"#;
    
    // Parse Qwen response
    let result = registry.parse(qwen_response);
    assert!(result.is_ok());
    
    let frame = result.unwrap();
    assert_eq!(frame.id, "qwen-456");
    assert_eq!(frame.model, "qwen");
    assert_eq!(frame.blocks.len(), 1);
    
    if let ContentBlock::Text { text } = &frame.blocks[0] {
        assert_eq!(text, "Hello from Qwen!");
    } else {
        panic!("Expected Text block");
    }
}

#[test]
fn test_unsupported_model() {
    let mut registry = ParserRegistry::new();
    registry.register_parser(Arc::new(TestClaudeParser));
    registry.register_parser(Arc::new(TestQwenParser));
    
    let unknown_response = r#"{
        "id": "unknown-789",
        "model": "unknown-model",
        "content": "Hello!"
    }"#;
    
    let result = registry.parse(unknown_response);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        ParseError::UnsupportedModel(model) => assert_eq!(model, "unknown-model"),
        _ => panic!("Expected UnsupportedModel error"),
    }
} 