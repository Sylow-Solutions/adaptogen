use adaptogen::normalized::{ContentBlock, ContentFrame};
use adaptogen::parser::{ModelResponseParser, ParseError};
use serde_json::Value;

// Example implementation of a Claude model parser
pub struct ClaudeParser;

impl ModelResponseParser for ClaudeParser {
    fn supported_models(&self) -> Vec<String> {
        vec!["claude".to_string()]
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

        // Extract content from Claude format (Claude has a slightly different format)
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
                            "tool_use" => {
                                if let (Some(id), Some(name), Some(input)) = (
                                    block.get("id").and_then(|i| i.as_str()),
                                    block.get("name").and_then(|n| n.as_str()),
                                    block.get("input"),
                                ) {
                                    blocks.push(ContentBlock::ToolUse {
                                        id: id.to_string(),
                                        name: name.to_string(),
                                        input: input.clone(),
                                    });
                                }
                            }
                            _ => {
                                // Ignore other block types for this example
                            }
                        }
                    }
                }
            }
        }

        Ok(ContentFrame { id, model, blocks })
    }
}

// Simple example showing usage of the parser
fn main() {
    println!("Claude Parser Example");
    
    let claude_response = r#"{
        "id": "example-claude-id",
        "model": "claude",
        "content": [
            {"type": "text", "text": "Hello from Claude!"}
        ]
    }"#;
    
    let parser = ClaudeParser;
    match parser.parse(claude_response) {
        Ok(frame) => {
            println!("Successfully parsed Claude response:");
            println!("  ID: {}", frame.id);
            println!("  Model: {}", frame.model);
            println!("  Blocks: {}", frame.blocks.len());
            
            for (i, block) in frame.blocks.iter().enumerate() {
                match block {
                    ContentBlock::Text { text } => {
                        println!("  Block {}: Text - {}", i, text);
                    }
                    _ => println!("  Block {}: Other block type", i),
                }
            }
        },
        Err(e) => println!("Error parsing response: {:?}", e),
    }
}
