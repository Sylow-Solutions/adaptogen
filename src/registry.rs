use serde_json::Value;
use std::sync::Arc;

use crate::normalized::ContentFrame;
use crate::parser::ModelResponseParser;
use crate::parser::ParseError;

/// Registry of model parsers
pub struct ParserRegistry {
    parsers: Vec<Arc<dyn ModelResponseParser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: Vec::new(),
        }
    }

    pub fn register_parser(&mut self, parser: Arc<dyn ModelResponseParser>) {
        self.parsers.push(parser);
    }

    pub fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
        let model = Self::extract_model(raw_response)?;

        for parser in &self.parsers {
            if parser.can_handle(&model) {
                return parser.parse(raw_response);
            }
        }

        Err(ParseError::UnsupportedModel(model))
    }

    fn extract_model(raw_response: &str) -> Result<String, ParseError> {
        // TODO: This is certaintly not the most robust way to do extract the model
        // name from the raw json string, but it works for now.
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
