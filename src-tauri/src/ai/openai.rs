use super::AiClient;
use serde_json::{json, Value};

pub struct OpenAiClient {
    api_key: String,
    base_url: String,
    model: String,
    is_gemini: bool,
}

impl OpenAiClient {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            is_gemini: false,
        }
    }

    pub fn new_for_gemini(api_key: &str, base_url: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            is_gemini: true,
        }
    }
}

impl AiClient for OpenAiClient {
    fn generate(&self, prompt: &str) -> Result<String, String> {
        let response = if self.is_gemini {
            self.call_gemini(prompt)
        } else {
            self.call_openai_format(prompt)
        };
        response
    }
}

impl OpenAiClient {
    fn call_openai_format(&self, prompt: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "You are a helpful assistant that generates professional daily work reports based on Git commit history."},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.7,
        });

        let res = ureq::post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| format!("AI request failed: {}", e))?;

        let status = res.status();
        let status_u16 = status.as_u16();
        let json: Value = res
            .into_body()
            .read_json()
            .map_err(|e| format!("AI response parse failed: {}", e))?;

        if status_u16 >= 400 {
            return Err(format!(
                "AI API error ({}): {}",
                status_u16,
                json["error"]["message"].as_str().unwrap_or("unknown error")
            ));
        }

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "AI response has no content".to_string())
    }

    fn call_gemini(&self, prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );
        let body = json!({
            "contents": [
                {
                    "parts": [
                        {"text": prompt}
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 0.7
            }
        });

        let res = ureq::post(&url)
            .header("Content-Type", "application/json")
            .send_json(&body)
            .map_err(|e| format!("AI request failed: {}", e))?;

        let status = res.status();
        let status_u16 = status.as_u16();
        let json: Value = res
            .into_body()
            .read_json()
            .map_err(|e| format!("AI response parse failed: {}", e))?;

        if status_u16 >= 400 {
            return Err(format!(
                "AI API error ({}): {}",
                status_u16,
                json["error"]["message"].as_str().unwrap_or("unknown error")
            ));
        }

        json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "AI response has no content".to_string())
    }
}
