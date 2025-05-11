# Adaptogen

Adaptogen is a Rust library for normalizing outputs from different Large Language Model (LLM) providers into a consistent format. It helps standardize the varied response structures from models like Claude, Qwen, and others into a unified content frame.

## Features

- **Model-agnostic parsing**: Parse responses from any LLM provider to a consistent format
- **Extensible architecture**: Easily implement custom parsers for new models
- **Registry system**: Simple registration and lookup of appropriate parsers for any given model
- **Normalized content blocks**: Consistent representation of text, tool calls, tool results, and thinking blocks

## Installation

Add adaptogen to your `Cargo.toml`:

```toml
[dependencies]
adaptogen = "0.1.0" 
```

## Usage

### Basic Usage

```rust
use std::sync::Arc;
use adaptogen::registry::ParserRegistry;
use adaptogen::normalized::ContentFrame;

// Create a registry and register your model parsers
fn main() {
    // Create a registry and register parsers
    let mut registry = ParserRegistry::new();
    
    // Register parsers for different models
    registry.register_parser(Arc::new(MyClaudeParser));
    registry.register_parser(Arc::new(MyQwenParser));
    
    // Parse a model response
    let response_json = r#"{"id": "msg_123", "model": "claude", "content": [...]}"#;
    match registry.parse(response_json) {
        Ok(frame) => {
            println!("ID: {}", frame.id);
            println!("Model: {}", frame.model);
            println!("Number of blocks: {}", frame.blocks.len());
            
            // Process the normalized content blocks
            for block in frame.blocks {
                match block {
                    ContentBlock::Text { text } => println!("Text: {}", text),
                    ContentBlock::ToolUse { id, name, input } => {
                        println!("Tool use: {}, Name: {}", id, name);
                    },
                    // Handle other block types
                    _ => {}
                }
            }
        },
        Err(e) => println!("Error parsing: {:?}", e),
    }
}
```

### Implementing a Custom Parser

To support a new model, implement the `ModelResponseParser` trait:

```rust
use adaptogen::normalized::{ContentBlock, ContentFrame};
use adaptogen::parser::{ModelResponseParser, ParseError};
use serde_json::Value;

struct MyCustomParser;

impl ModelResponseParser for MyCustomParser {
    fn supported_models(&self) -> Vec<String> {
        vec!["custom-model".to_string(), "custom-model-v2".to_string()]
    }
    
    fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
        // Parse the raw JSON response
        let json: Value = serde_json::from_str(raw_response)?;
        
        // Extract id and model information
        let id = json.get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("id".to_string()))?
            .to_string();
            
        let model = json.get("model")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParseError::MissingField("model".to_string()))?
            .to_string();
        
        // Extract and normalize content blocks according to your model's format
        let mut blocks = Vec::new();
        
        // Add normalized blocks based on your model's structure
        if let Some(content) = json.get("output").and_then(|o| o.as_str()) {
            blocks.push(ContentBlock::Text {
                text: content.to_string()
            });
        }
        
        // Return the normalized ContentFrame
        Ok(ContentFrame { id, model, blocks })
    }
}
```

### Full Example with Multiple Parsers

```rust
use std::sync::Arc;
use adaptogen::registry::ParserRegistry;
use adaptogen::normalized::ContentFrame;
use adaptogen::parser::ParseError;

// Import your model parsers
mod claude_parser;
mod qwen_parser;

use claude_parser::ClaudeParser;
use qwen_parser::QwenParser;

// Create a registry with default parsers
fn create_default_registry() -> ParserRegistry {
    let mut registry = ParserRegistry::new();
    registry.register_parser(Arc::new(QwenParser));
    registry.register_parser(Arc::new(ClaudeParser));
    registry
}

// Convenience function to parse responses
fn parse(raw_response: &str) -> Result<ContentFrame, ParseError> {
    let registry = create_default_registry();
    registry.parse(raw_response)
}

fn main() {
    // Example Claude response
    let claude_response = r#"{
        "id": "example-claude-id",
        "model": "claude",
        "content": [
            {"type": "text", "text": "Hello from Claude!"}
        ]
    }"#;
    
    // Parse and use the normalized content
    match parse(claude_response) {
        Ok(frame) => println!("Successfully parsed response from model: {}", frame.model),
        Err(e) => println!("Error parsing response: {:?}", e)
    }
}
```

## Content Block Types

Adaptogen normalizes content into the following block types:

- **Text**: Simple text content from the model
- **ToolUse**: Function or tool calls made by the model
- **ToolResult**: Results returned from tool executions
- **Thinking**: Internal reasoning processes from models that expose them

## License

[MIT License](LICENSE) 