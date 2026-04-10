use crate::error::Result;
use serde::{Deserialize, Serialize};

/// System prompt injected into every AI conversation.
const SYSTEM_PROMPT: &str = "\
You are Sophisticated AI™, the blame engine behind git-blame-2.0. \
You write emails that are professional yet devastating. \
You maintain plausible deniability while being maximally passive-aggressive. \
Always sign emails as 'Sophisticated AI™' followed by 'https://gitblame.org' on the next line.";

/// OpenRouter API endpoint.
const API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Blocking client for the OpenRouter chat-completions API.
pub struct AiClient {
    api_key: String,
    model: String,
    http: reqwest::blocking::Client,
}

impl AiClient {
    /// Create a new [`AiClient`] with the given OpenRouter API key and model.
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            http: reqwest::blocking::Client::new(),
        }
    }

    /// Generate a completion for the given user `prompt`.
    ///
    /// Sends the prompt together with a system message to the OpenRouter API
    /// and returns the assistant's reply text.
    pub fn generate(&self, prompt: &str) -> Result<String> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: SYSTEM_PROMPT.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: prompt.into(),
                },
            ],
        };

        let resp = self
            .http
            .post(API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("OpenRouter request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body_text = resp.text().unwrap_or_default();
            return Err(format!("OpenRouter returned {status}: {body_text}").into());
        }

        let chat_resp: ChatResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse OpenRouter response: {e}"))?;

        chat_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "OpenRouter returned no choices".into())
    }
}

// ---------------------------------------------------------------------------
// Request / response DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}
