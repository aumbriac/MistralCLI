mod constants;
mod types;
mod utils;

use crate::constants::DEFAULT_MODEL;
use crate::utils::make_mistral_request;
use std::{env, error::Error};

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
