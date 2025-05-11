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
//! ```rust
//! use std::sync::Arc;
//! use adaptogen::registry::ParserRegistry;
//! use adaptogen::normalized::ContentFrame;
//! use adaptogen::parser::ModelResponseParser;
//!
//! // Create a registry and register parsers
//! let mut registry = ParserRegistry::new();
//!
//! // After implementing your own parsers, register them
//! registry.register_parser(Arc::new(MyParser));
//!
//! // Parse a model response
//! let response_json = r#"{"id": "msg_123", "model": "model_name", "content": [...]}"#;
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
