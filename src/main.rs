use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::{Serialize, Deserialize};
use std::{env, error::Error};
use indicatif::{ProgressStyle, ProgressBar};
use futures::stream::StreamExt;
use std::io::{self, Write};

const MISTRAL_API_URL: &str = "https://api.mistral.ai/v1/chat/completions";
const DEFAULT_MODEL: &str = "mistral-tiny";

#[derive(Deserialize, Debug)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    delta: Delta,
}

#[derive(Deserialize, Debug)]
struct Delta {
    content: Option<String>,
}

#[derive(Serialize)]
struct MistralRequestBody {
    model: String,
    messages: Vec<MessageRole>,
    stream: bool,
}

#[derive(Serialize)]
struct MessageRole {
    role: String,
    content: String,
}

async fn make_mistral_request(client: &reqwest::Client, model: &str, prompt: &str) -> Result<(), Box<dyn Error>> {
    let headers = HeaderMap::from_iter(vec![
        (AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", env::var("MISTRAL_API_KEY")?))?),
    ]);
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template("{spinner:.yellow} {msg}"));
    spinner.enable_steady_tick(100);
    spinner.set_message("Generating response...");

    let request_body = MistralRequestBody {
        model: model.to_string(),
        messages: vec![
            MessageRole { role: "user".to_string(), content: prompt.to_string() },
        ],
        stream: true
    };

    let response = client.post(MISTRAL_API_URL)
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
                    match serde_json::from_str::<ApiResponse>(line) {
                        Ok(api_response) => {
                            for choice in api_response.choices {
                                if let Some(content) = choice.delta.content {
                                    print!("{}", content);
                                    io::stdout().flush().unwrap();
                                }
                            }
                        },
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    
    let mut model = DEFAULT_MODEL.to_string();
    let prompt: String;

    if args.len() < 2 {
        eprintln!("Usage: mistral [-m model_name] <prompt>");
        return Err("Invalid arguments".into());
    }

    if args.len() > 3 && args[1] == "-m" {
        model = format!("mistral-{}", args[2]);
        prompt = args[3..].join(" ");
    } else {
        prompt = args[1..].join(" ");
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    make_mistral_request(&client, &model, &prompt).await?;

    Ok(())
}
