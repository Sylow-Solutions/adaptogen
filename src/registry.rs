use serde_json::Value;
use std::sync::Arc;

use crate::normalized::ContentFrame;
use crate::parser::ModelResponseParser;
use crate::parser::ParseError;

/// Registry of model parsers
///
/// The `ParserRegistry` maintains a collection of model parsers and provides
/// functionality to parse LLM responses by selecting the appropriate parser
/// based on the model identifier in the response.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use adaptogen::registry::ParserRegistry;
/// use adaptogen::parser::ModelResponseParser;
///
/// // Create a new registry
/// let mut registry = ParserRegistry::new();
///
/// // Register parsers for different models
/// registry.register_parser(Arc::new(MyClaudeParser));
/// registry.register_parser(Arc::new(MyQwenParser));
///
/// // Parse a response
/// let response = r#"{"id": "msg_123", "model": "claude", "content": [...]}"#;
/// let result = registry.parse(response);
/// ```
pub struct ParserRegistry {
    parsers: Vec<Arc<dyn ModelResponseParser>>,
}

impl ParserRegistry {
    /// Create a new empty registry
    ///
    /// Returns a `ParserRegistry` with no registered parsers.
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    /// Register a new parser
    ///
    /// Adds a parser to the registry. When parsing responses, parsers are checked in
    /// the order they were registered.
    pub fn register_parser(&mut self, parser: Arc<dyn ModelResponseParser>) {
        self.parsers.push(parser);
    }

    /// Parse a raw LLM response
    ///
    /// Attempts to parse the given response string by:
    /// 1. Extracting the model identifier from the response
    /// 2. Finding a parser that supports the identified model
    /// 3. Using that parser to parse the complete response
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if:
    /// - The JSON is invalid
    /// - The model field is missing
    /// - No parser is registered for the identified model
    /// - The selected parser fails to parse the response
    pub fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
        let model = Self::extract_model(raw_response)?;

        for parser in &self.parsers {
            if parser.can_handle(&model) {
                return parser.parse(raw_response);
            }
        }

        Err(ParseError::UnsupportedModel(model))
    }

    /// Extract the model identifier from a response
    ///
    /// Parses the response as JSON and extracts the "model" field.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if:
    /// - The JSON is invalid
    /// - The model field is missing
    fn extract_model(raw_response: &str) -> Result<String, ParseError> {
        let json: Value = serde_json::from_str(raw_response)?;

        if let Some(model) = json.get("model").and_then(|m| m.as_str()) {
            Ok(model.to_string())
        } else {
            Err(ParseError::MissingField("model".to_string()))
        }
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalized::{ContentBlock, ContentFrame};

    // Mock parser for testing
    struct MockParser {
        models: Vec<String>,
        should_succeed: bool,
    }

    impl ModelResponseParser for MockParser {
        fn supported_models(&self) -> Vec<String> {
            self.models.clone()
        }

        fn parse(&self, _raw_response: &str) -> Result<ContentFrame, ParseError> {
            if self.should_succeed {
                Ok(ContentFrame {
                    id: "test_id".to_string(),
                    model: self.models.first().unwrap_or(&"unknown".to_string()).clone(),
                    blocks: vec![ContentBlock::Text { 
                        text: "Test response".to_string() 
                    }],
                })
            } else {
                Err(ParseError::Other("Simulated failure".to_string()))
            }
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = ParserRegistry::new();
        assert_eq!(registry.parsers.len(), 0);
    }

    #[test]
    fn test_register_parser() {
        let mut registry = ParserRegistry::new();
        let parser = Arc::new(MockParser {
            models: vec!["test_model".to_string()],
            should_succeed: true,
        });
        
        registry.register_parser(parser);
        assert_eq!(registry.parsers.len(), 1);
    }

    #[test]
    fn test_extract_model_success() {
        let json_str = r#"{"id": "123", "model": "test_model", "content": "test"}"#;
        let result = ParserRegistry::extract_model(json_str);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_model");
    }

    #[test]
    fn test_extract_model_missing() {
        let json_str = r#"{"id": "123", "content": "test"}"#;
        let result = ParserRegistry::extract_model(json_str);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::MissingField(field) => assert_eq!(field, "model"),
            _ => panic!("Expected MissingField error"),
        }
    }

    #[test]
    fn test_parse_success() {
        let mut registry = ParserRegistry::new();
        let parser = Arc::new(MockParser {
            models: vec!["test_model".to_string()],
            should_succeed: true,
        });
        
        registry.register_parser(parser);
        
        let response = r#"{"id": "123", "model": "test_model", "content": "test"}"#;
        let result = registry.parse(response);
        
        assert!(result.is_ok());
        let frame = result.unwrap();
        assert_eq!(frame.model, "test_model");
    }

    #[test]
    fn test_parse_unsupported_model() {
        let mut registry = ParserRegistry::new();
        let parser = Arc::new(MockParser {
            models: vec!["test_model".to_string()],
            should_succeed: true,
        });
        
        registry.register_parser(parser);
        
        let response = r#"{"id": "123", "model": "unsupported_model", "content": "test"}"#;
        let result = registry.parse(response);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::UnsupportedModel(model) => assert_eq!(model, "unsupported_model"),
            _ => panic!("Expected UnsupportedModel error"),
        }
    }
}
