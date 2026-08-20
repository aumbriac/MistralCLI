mod config;
mod constants;
mod types;
mod utils;
mod ocr;
mod vision;
mod misc;

use crate::constants::DEFAULT_MODEL;
use crate::utils::{make_mistral_request, list_models, handle_file_upload};
use std::{env, error::Error};
use chrono::DateTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let mut model = DEFAULT_MODEL.to_string();
    let mut config_path: Option<&str> = None;
    let mut file_path: Option<&str> = None;
    let prompt: String;

    if args.len() < 2 {
        print_help();
        return Err("Invalid arguments".into());
    }

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => {
                if i + 1 < args.len() {
                    config_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Missing value for -c option");
                    return Err("Invalid arguments".into());
                }
            }
            "-m" => {
                if i + 1 < args.len() {
                    model = format!("mistral-{}", args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Missing value for -m option");
                    return Err("Invalid arguments".into());
                }
            }
            "-f" => {
                if i + 1 < args.len() {
                    file_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Missing value for -f option");
                    return Err("Invalid arguments".into());
                }
            }
            "-l" => {
                let api_key = crate::config::get_api_key(config_path)?;
                let models = list_models(&api_key).await?;
                for model in models.data {
                    let created_date = DateTime::from_timestamp(model.created as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!("{}: {} (created: {})", model.id, model.object, created_date);
                }
                return Ok(());
            }
            "-h" => {
                print_help();
                return Ok(());
            }
            _ => break,
        }
    }

    if let Ok(Some(config_model)) = crate::config::get_preferred_model(config_path) {
        model = config_model;
    }

    prompt = args[i..].join(" ");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    if let Some(file_path) = file_path {
        handle_file_upload(&client, &model, file_path, &prompt, config_path).await?;
    } else {
        make_mistral_request(&client, &model, &prompt, config_path).await?;
    }

    Ok(())
}


fn print_help() {
    eprintln!("Usage: mistral [-c config_path] [-m model_name] [-l] [-f file_path] [-h] <prompt>");
    eprintln!("\nOptions:");
    eprintln!("  -c config_path  Path to the configuration file");
    eprintln!("  -m model_name   Name of the model to use");
    eprintln!("  -l              List available models");
    eprintln!("  -f file_path    Path to a file to upload");
    eprintln!("  -h              Show this help message");
    eprintln!("\n<prompt>          The prompt to send to the model");
}
