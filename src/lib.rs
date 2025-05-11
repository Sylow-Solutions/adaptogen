//! # Adaptogen
//!
//! Adaptogen is a Rust library for normalizing outputs from different Large Language Model (LLM) 
//! providers into a consistent format. It helps standardize the varied response structures from 
//! models like Claude, Qwen, and others into a unified content frame.
//!
//! ## Features
//!
//! - **Model-agnostic parsing**: Parse responses from any LLM provider to a consistent format
//! - **Extensible architecture**: Easily implement custom parsers for new models
//! - **Registry system**: Simple registration and lookup of appropriate parsers for any given model
//! - **Normalized content blocks**: Consistent representation of text, tool calls, tool results, and thinking blocks
//!
//! ## Basic Usage
//!
//! ```
//! use std::sync::Arc;
//! use adaptogen::registry::ParserRegistry;
//! use adaptogen::normalized::ContentFrame;
//! use adaptogen::parser::{ModelResponseParser, ParseError};
//! 
//! // Define a simple example parser
//! struct ExampleParser;
//! 
//! impl ModelResponseParser for ExampleParser {
//!     fn supported_models(&self) -> Vec<String> {
//!         vec!["model_name".to_string()]
//!     }
//!     
//!     fn parse(&self, _raw_response: &str) -> Result<ContentFrame, ParseError> {
//!         // This is just for the doctest example
//!         Ok(ContentFrame {
//!             id: "msg_123".to_string(),
//!             model: "model_name".to_string(),
//!             blocks: vec![]
//!         })
//!     }
//! }
//!
//! // Create a registry and register parsers
//! let mut registry = ParserRegistry::new();
//!
//! // Register your parser
//! registry.register_parser(Arc::new(ExampleParser));
//!
//! // Parse a model response
//! let response_json = r#"{"id": "msg_123", "model": "model_name", "content": []}"#;
//! match registry.parse(response_json) {
//!     Ok(frame) => {
//!         println!("Parsed {} blocks from {}", frame.blocks.len(), frame.model);
//!     },
//!     Err(e) => println!("Error parsing: {:?}", e),
//! }
//! ```

pub mod normalized;
pub mod parser;
pub mod registry;
