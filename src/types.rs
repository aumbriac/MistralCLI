use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct MistralApiResponse {
    pub choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub delta: Delta,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct MessageRole {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct MistralRequestBody {
    pub model: String,
    pub messages: Vec<MessageRole>,
    pub stream: bool,
}
