use crate::config::{get_api_key, get_preferred_model};
use crate::constants::{MISTRAL_API_URL, DEFAULT_MODEL, MISTRAL_MODELS_ENDPOINT};
use crate::types::{MessageRole, MistralApiResponse, MistralRequestBody, ModelListResponse};
use crate::ocr::handle_ocr_request;
use crate::vision::handle_vision_request;
use crate::misc::create_spinner;
use futures::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::io::{self, Write};
use std::{error::Error, path::Path};
use mime_guess::from_path;

pub async fn make_mistral_request(
    client: &reqwest::Client,
    model: &str,
    prompt: &str,
    config_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let api_key = get_api_key(config_path)?;
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    )]);
    let spinner = create_spinner();
    spinner.set_message("Generating response...");

    let request_body = MistralRequestBody {
        model: model.to_string(),
        messages: vec![MessageRole {
            role: "user".to_string(),
            content: serde_json::Value::String(prompt.to_string()),
        }],
        stream: true,
    };

    let response = client
        .post(MISTRAL_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    spinner.finish_and_clear();
    if response.status().is_success() {
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            let chunk_str = String::from_utf8(chunk.to_vec())?;

            for line in chunk_str.split('\n') {
                let line = line.trim_start_matches("data: ").trim();
                if !line.is_empty() {
                    match serde_json::from_str::<MistralApiResponse>(line) {
                        Ok(api_response) => {
                            for choice in api_response.choices {
                                if let Some(content) = choice.delta.content {
                                    print!("{}", content);
                                    io::stdout().flush().unwrap();
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    } else {
        spinner.finish_with_message("Error!");
        return Err("An unexpected error has occurred".into());
    }
    println!();

    Ok(())
}

pub async fn list_models(api_key: &str) -> Result<ModelListResponse, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    )]);

    let response = client
        .get(MISTRAL_MODELS_ENDPOINT)
        .headers(headers)
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let models: ModelListResponse = serde_json::from_str(&body)?;
        Ok(models)
    } else {
        Err("Failed to fetch models".into())
    }
}

pub async fn handle_file_upload(
    client: &reqwest::Client,
    _model: &str,
    file_path: &str,
    prompt: &str,
    config_path: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    let mime = from_path(path)
        .first_or_octet_stream()
        .to_string();

    if mime.starts_with("image/") {
        handle_vision_request(client, file_path, prompt, config_path).await?;
    } else if mime == "application/pdf" {
        let ocr_text = handle_ocr_request(client, file_path, config_path).await?;
        if !ocr_text.is_empty() {
            let api_key = get_api_key(config_path)?;
            let headers = HeaderMap::from_iter(vec![(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))?,
            )]);

            let model = get_preferred_model(config_path)?.unwrap_or_else(|| DEFAULT_MODEL.to_string());

            let summary_request = MistralRequestBody {
                model,
                messages: vec![
                    MessageRole {
                        role: "system".to_string(),
                        content: serde_json::Value::String("User has provided a response from the OCR API and a question regarding the response. Answer it truthfully and avoid excessive formatting and markdown.".to_string()),
                    },
                    MessageRole {
                        role: "user".to_string(),
                        content: serde_json::Value::String(format!("{}\n\n\"{}\"", prompt, ocr_text)),
                    },
                ],
                stream: false,
            };

            let summary_response = client
                .post(MISTRAL_API_URL)
                .headers(headers)
                .json(&summary_request)
                .send()
                .await?;

            if summary_response.status().is_success() {
                let summary_text = summary_response.text().await?;
                let summary_result: serde_json::Value = serde_json::from_str(&summary_text)?;

                if let Some(summary) = summary_result["choices"][0]["message"]["content"].as_str() {
                    println!("{}", summary);
                } else {
                    println!("Could not extract summary from response");
                }
            } else {
                println!("Failed to get document summary");
            }
        } else {
            println!("No text extracted from OCR");
        }
    } else {
        return Err(format!("Unsupported file type: {}", mime).into());
    }

    Ok(())
}
