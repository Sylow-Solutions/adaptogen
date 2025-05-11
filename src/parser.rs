use crate::normalized::ContentFrame;

/// Trait for parsing LLM model responses into ContentBlocks
pub trait ModelResponseParser: Send + Sync {
    /// Returns the model identifier(s) this parser supports
    fn supported_models(&self) -> Vec<String>;
    /// Parse raw response data into ContentBlocks
    fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError>;
    /// inside of the top level parse function in parser registry we do a naive search
    /// basically just loop through the vector to find the first parser that supports a specific
    /// model. This is the function that gets called there.
    fn can_handle(&self, model: &str) -> bool {
        self.supported_models().iter().any(|m| m == model)
    }
}

/// Error type for parsing failures
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),

    #[error("Parsing error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalized::{ContentBlock, ContentFrame};

    // Mock implementation of ModelResponseParser for testing
    struct MockParser {
        supported: Vec<String>,
    }

    impl ModelResponseParser for MockParser {
        fn supported_models(&self) -> Vec<String> {
            self.supported.clone()
        }

        fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
            // Simple check to validate it gets the expected input
            if !raw_response.contains("mock") {
                return Err(ParseError::Other("Expected mock in response".to_string()));
            }

            Ok(ContentFrame {
                id: "mock_id".to_string(),
                model: "mock_model".to_string(),
                blocks: vec![ContentBlock::Text {
                    text: "Mocked response".to_string(),
                }],
            })
        }
    }

    #[test]
    fn test_can_handle() {
        let parser = MockParser {
            supported: vec!["model1".to_string(), "model2".to_string()],
        };
        
        assert!(parser.can_handle("model1"));
        assert!(parser.can_handle("model2"));
        assert!(!parser.can_handle("model3"));
    }

    #[test]
    fn test_parse_success() {
        let parser = MockParser {
            supported: vec!["mock_model".to_string()],
        };
        
        let result = parser.parse("mock response data");
        assert!(result.is_ok());
        
        let frame = result.unwrap();
        assert_eq!(frame.id, "mock_id");
        assert_eq!(frame.model, "mock_model");
        assert_eq!(frame.blocks.len(), 1);
    }

    #[test]
    fn test_parse_error() {
        let parser = MockParser {
            supported: vec!["mock_model".to_string()],
        };
        
        let result = parser.parse("not containing expected keyword");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ParseError::Other(msg) => assert_eq!(msg, "Expected mock in response"),
            _ => panic!("Expected Other error variant"),
        }
    }
}
