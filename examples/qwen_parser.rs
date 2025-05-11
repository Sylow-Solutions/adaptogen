use adaptogen::normalized::{ContentBlock, ContentFrame};
use adaptogen::parser::{ModelResponseParser, ParseError};
use serde_json::{json, Value};

// Example implementation of a Qwen model parser
pub struct QwenParser;

impl ModelResponseParser for QwenParser {
    fn supported_models(&self) -> Vec<String> {
        vec!["qwen".to_string(), "accounts/fireworks/models/qwen3-30b-a3b".to_string()]
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
                    // Extract thinking block if present
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        // Check if there's a thinking block
                        if let Some(thinking_end) = content.find("</think>") {
                            if content.starts_with("<think>") {
                                let thinking = content[7..thinking_end].trim().to_string();
                                blocks.push(ContentBlock::Thinking {
                                    thinking: Some(thinking),
                                    signature: None,
                                });
                            }
                        }

                        // Add text block if content isn't empty after thinking
                        let text_content = content.split("</think>").last().unwrap_or("").trim();

                        if !text_content.is_empty() {
                            blocks.push(ContentBlock::Text {
                                text: text_content.to_string(),
                            });
                        }
                    }

                    // Extract tool calls
                    if let Some(tool_calls) = message.get("tool_calls").and_then(|t| t.as_array()) {
                        for tool_call in tool_calls {
                            if let (Some(id), Some(name), Some(args)) = (
                                tool_call.get("id").and_then(|i| i.as_str()),
                                tool_call
                                    .get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|n| n.as_str()),
                                tool_call
                                    .get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|a| a.as_str()),
                            ) {
                                let input: Value = serde_json::from_str(args)
                                    .unwrap_or_else(|_| json!({"raw": args}));

                                blocks.push(ContentBlock::ToolUse {
                                    id: id.to_string(),
                                    name: name.to_string(),
                                    input,
                                });
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
    println!("Qwen Parser Example");
    
    // Basic Qwen response
    let qwen_response = r#"{
        "id": "example-qwen-id",
        "model": "qwen",
        "choices": [
            {
                "message": {
                    "content": "Hello from Qwen!"
                }
            }
        ]
    }"#;
    
    // Qwen response with thinking block
    let qwen_thinking_response = r#"{
        "id": "example-qwen-thinking-id",
        "model": "qwen",
        "choices": [
            {
                "message": {
                    "content": "<think>This is my thinking process.</think>Here is my actual response."
                }
            }
        ]
    }"#;
    
    let parser = QwenParser;
    
    // Parse basic response
    println!("\nParsing basic Qwen response:");
    match parser.parse(qwen_response) {
        Ok(frame) => {
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
    
    // Parse thinking response
    println!("\nParsing Qwen response with thinking block:");
    match parser.parse(qwen_thinking_response) {
        Ok(frame) => {
            println!("  ID: {}", frame.id);
            println!("  Model: {}", frame.model);
            println!("  Blocks: {}", frame.blocks.len());
            
            for (i, block) in frame.blocks.iter().enumerate() {
                match block {
                    ContentBlock::Text { text } => {
                        println!("  Block {}: Text - {}", i, text);
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        println!("  Block {}: Thinking - {:?}", i, thinking);
                    }
                    _ => println!("  Block {}: Other block type", i),
                }
            }
        },
        Err(e) => println!("Error parsing response: {:?}", e),
    }
}
