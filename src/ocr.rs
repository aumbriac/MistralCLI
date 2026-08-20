use crate::config::get_api_key;
use crate::constants::{MISTRAL_OCR_API_URL, DEFAULT_OCR_MODEL};
use crate::misc::create_spinner;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::{error::Error, fs, path::Path};
use base64::{self, Engine};
use mime_guess::from_path;
use serde_json::Value;

pub async fn handle_ocr_request(
    client: &reqwest::Client,
    file_path: &str,
    config_path: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    let file_content = fs::read(path)?;
    let base64_content = base64::engine::general_purpose::STANDARD.encode(&file_content);

    let _mime = from_path(path)
        .first_or_octet_stream()
        .to_string();

    let api_key = get_api_key(config_path)?;
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    )]);

    let spinner = create_spinner();
    spinner.set_message("Processing document...");

    let request_body = serde_json::json!({
        "model": DEFAULT_OCR_MODEL,
        "document": {
            "type": "document_url",
            "document_url": format!("data:application/pdf;base64,{}", base64_content),
        },
        "include_image_base64": true
    });

    let response = client
        .post(MISTRAL_OCR_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    spinner.finish_and_clear();
    let status = response.status();
    if status.is_success() {
        let response_text = response.text().await?;
        let ocr_result: Value = serde_json::from_str(&response_text)?;

        let mut full_text = String::new();
        if let Some(pages) = ocr_result["pages"].as_array() {
            for page in pages {
                if let Some(markdown) = page["markdown"].as_str() {
                    full_text.push_str(markdown);
                    full_text.push('\n');
                }
            }
        } else {
            println!("No pages found in OCR response");
        }

        Ok(full_text)
    } else {
        spinner.finish_with_message("Error!");
        let error_text = response.text().await?;
        eprintln!("API Error: {}\nResponse: {}", status, error_text);
        Err(format!("API Error: {}", status).into())
    }
}
