use crate::config::get_api_key;
use crate::constants::{MISTRAL_API_URL, DEFAULT_VISION_MODEL};
use crate::types::{MessageRole, MistralRequestBody};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::{error::Error, fs, path::Path};
use base64::{self, Engine};
use mime_guess::from_path;
use crate::misc::create_spinner;

pub async fn handle_vision_request(
    client: &reqwest::Client,
    file_path: &str,
    prompt: &str,
    config_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    let file_content = fs::read(path)?;
    let base64_content = base64::engine::general_purpose::STANDARD.encode(&file_content);

    let mime = from_path(path)
        .first_or_octet_stream()
        .to_string();

    let api_key = get_api_key(config_path)?;
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    )]);

    let spinner = create_spinner();
    spinner.set_message("Processing image...");

    let request_body = MistralRequestBody {
        model: DEFAULT_VISION_MODEL.to_string(),
        messages: vec![
            MessageRole {
                role: "user".to_string(),
                content: serde_json::json!([
                    {
                        "type": "text",
                        "text": prompt
                    },
                    {
                        "type": "image_url",
                        "image_url": format!("data:{};base64,{}", mime, base64_content)
                    }
                ]),
            },
        ],
        stream: false,
    };

    let response = client
        .post(MISTRAL_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    spinner.finish_and_clear();
    let status = response.status();
    if status.is_success() {
        let response_text = response.text().await?;
        let response_result: serde_json::Value = serde_json::from_str(&response_text)?;

        if let Some(content) = response_result["choices"][0]["message"]["content"].as_str() {
            println!("{}", content);
        } else {
            println!("Could not extract response from vision API");
        }
    } else {
        spinner.finish_with_message("Error!");
        let error_text = response.text().await?;
        eprintln!("API Error: {}\nResponse: {}", status, error_text);
        return Err(format!("API Error: {}", status).into());
    }

    Ok(())
}
