use crate::constants::MISTRAL_API_URL;
use crate::types::{MessageRole, MistralApiResponse, MistralRequestBody};
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::io::{self, Write};
use std::{env, error::Error};

pub async fn make_mistral_request(
    client: &reqwest::Client,
    model: &str,
    prompt: &str,
) -> Result<(), Box<dyn Error>> {
    let headers = HeaderMap::from_iter(vec![(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", env::var("MISTRAL_API_KEY")?))?,
    )]);
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.yellow} {msg}"),
    );
    spinner.enable_steady_tick(100);
    spinner.set_message("Generating response...");

    let request_body = MistralRequestBody {
        model: model.to_string(),
        messages: vec![MessageRole {
            role: "user".to_string(),
            content: prompt.to_string(),
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
