use std::sync::Arc;
use adaptogen::registry::ParserRegistry;
use adaptogen::normalized::ContentFrame;
use adaptogen::parser::ParseError;

mod claude_parser;
mod qwen_parser;

use claude_parser::ClaudeParser;
use qwen_parser::QwenParser;

// Register your parsers 
fn create_default_registry() -> ParserRegistry {
    let mut registry = ParserRegistry::new();
    registry.register_parser(Arc::new(QwenParser));
    registry.register_parser(Arc::new(ClaudeParser));
    registry
}

// simple convenience method to contruct the registy and call parse
fn parse(raw_response: &str) -> Result<ContentFrame, ParseError> {
    let registry = create_default_registry();
    registry.parse(raw_response)
}

// Example usage
fn main() {
    
    // Example Claude response
    let claude_response = r#"{
        "id": "example-claude-id",
        "model": "claude",
        "content": [
            {"type": "text", "text": "Hello from Claude!"}
        ]
    }"#;
    
    // Example Qwen response with function calls
    let qwen_response = r#"{"id":"example-id","object":"chat.completion","created":1746977262,"model":"accounts/fireworks/models/qwen3-30b-a3b","choices":[{"index":0,"message":{"role":"assistant","content":"<think>\nOkay, the user is asking for the capital of France. Let me check the tools available. There's a function called search_capital that takes a country name as a parameter. So I need to call that function with \"France\" as the argument. I should make sure the country name is correctly spelled and formatted. Once I get the result from the function, I can present the capital city to the user. Alright, let's proceed with the function call.\n</think>\n\n","tool_calls":[{"index":0,"id":"call_Qi2Is8SYTdRWjAToAViVLGeE","type":"function","function":{"name":"search_capital","arguments":"{\"country\": \"France\"}"}}]},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":172,"total_tokens":290,"completion_tokens":118}}"#;
    
    // Parse both responses using our registry
    match parse(claude_response) {
        Ok(frame) => println!("Successfully parsed Claude response: model={}, blocks={:?}", 
                             frame.model, frame.blocks),
        Err(e) => println!("Error parsing Claude response: {:?}", e)
    }
    
    match parse(qwen_response) {
        Ok(frame) => println!("Successfully parsed Qwen response: model={}, blocks={:?}", 
                             frame.model, frame.blocks),
        Err(e) => println!("Error parsing Qwen response: {:?}", e)
    }
}
