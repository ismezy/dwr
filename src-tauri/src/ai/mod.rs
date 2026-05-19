pub mod openai;

pub trait AiClient: Send + Sync {
    fn generate(&self, prompt: &str) -> Result<String, String>;
}

pub fn create_client(
    provider: &str,
    api_key: &str,
    base_url: Option<&str>,
    model: &str,
) -> Result<Box<dyn AiClient>, String> {
    match provider {
        "openai" | "deepseek" | "custom" => {
            let base = match provider {
                "openai" => "https://api.openai.com/v1",
                "deepseek" => "https://api.deepseek.com/v1",
                _ => base_url.ok_or("base_url required for custom provider")?,
            };
            Ok(Box::new(openai::OpenAiClient::new(api_key, base, model)))
        }
        "anthropic" => {
            let base = base_url.unwrap_or("https://api.anthropic.com/v1");
            Ok(Box::new(openai::OpenAiClient::new(api_key, base, model)))
        }
        "gemini" => {
            let base = base_url.unwrap_or("https://generativelanguage.googleapis.com/v1beta");
            Ok(Box::new(openai::OpenAiClient::new_for_gemini(api_key, base, model)))
        }
        _ => Err(format!("unsupported provider: {}", provider)),
    }
}
