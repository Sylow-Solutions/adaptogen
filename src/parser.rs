use crate::normalized::ContentFrame;

/// Trait for parsing LLM model responses into ContentFrames
///
/// Implement this trait to create parsers for specific LLM providers.
/// Each parser is responsible for converting the model-specific JSON format
/// into the normalized ContentFrame structure.
///
/// # Examples
///
/// ```
/// use adaptogen::normalized::{ContentBlock, ContentFrame};
/// use adaptogen::parser::{ModelResponseParser, ParseError};
/// use serde_json::Value;
///
/// struct MyModelParser;
///
/// impl ModelResponseParser for MyModelParser {
///     fn supported_models(&self) -> Vec<String> {
///         vec!["my-model".to_string()]
///     }
///
///     fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError> {
///         // Implementation of parsing logic
///         # Ok(ContentFrame { 
///         #    id: "example".into(), 
///         #    model: "my-model".into(), 
///         #    blocks: vec![] 
///         # })
///     }
/// }
/// ```
pub trait ModelResponseParser: Send + Sync {
    /// Returns the model identifier(s) this parser supports
    ///
    /// This method should return a list of model names that this parser can handle.
    /// The parser registry uses this to determine which parser to use for a given model.
    fn supported_models(&self) -> Vec<String>;
    
    /// Parse raw response data into a ContentFrame
    ///
    /// This method is responsible for converting the raw JSON string from the LLM
    /// into the normalized ContentFrame format.
    fn parse(&self, raw_response: &str) -> Result<ContentFrame, ParseError>;
    
    /// Determines if this parser can handle a specific model
    ///
    /// This method checks if the given model string matches any of the
    /// supported models returned by `supported_models()`.
    fn can_handle(&self, model: &str) -> bool {
        self.supported_models().iter().any(|m| m == model)
    }
}

/// Error type for parsing failures
///
/// This enum represents the different types of errors that can occur
/// during the parsing process.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Error when the response JSON is invalid
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// Error when a required field is missing from the response
    #[error("Missing field: {0}")]
    MissingField(String),

    /// Error when the model is not supported by any registered parser
    #[error("Unsupported model: {0}")]
    UnsupportedModel(String),

    /// General parsing error with a custom message
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
