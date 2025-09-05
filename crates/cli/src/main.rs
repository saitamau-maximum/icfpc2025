use anyhow::Result;
use clap::{Parser, Subcommand};
use icfpc2025_client::{AedificiumClient, Map};
use std::env;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "aedificium")]
#[command(about = "ICFPC 2025 Aedificium contest CLI tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Select a problem")]
    Select {
        #[arg(help = "Problem name, or read from stdin if not provided")]
        problem: Option<String>,
    },
    #[command(about = "Explore with plans")]
    Explore {
        #[arg(help = "Plans (comma-separated), or read from stdin if not provided")]
        plans: Option<String>,
    },
    #[command(about = "Submit a guess")]
    Guess {
        #[arg(help = "Map data as JSON string, or read from stdin if not provided")]
        map: Option<String>,
    },
}

fn get_input_or_stdin(arg: Option<String>, field_name: &str) -> Result<String> {
    match arg {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(anyhow::anyhow!("{} cannot be empty", field_name));
            }
            Ok(trimmed.to_string())
        }
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            let trimmed = buffer.trim();
            if trimmed.is_empty() {
                return Err(anyhow::anyhow!(
                    "{} cannot be empty. Provide via argument or stdin.",
                    field_name
                ));
            }
            Ok(trimmed.to_string())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let team_id = env::var("ICFPC_TEAM_ID").map_err(|_| {
        anyhow::anyhow!(
            "Team ID is required. Set via ICFPC_TEAM_ID environment variable or .env file"
        )
    })?;

    let client = AedificiumClient::new(team_id);

    match cli.command {
        Commands::Select { problem } => {
            let problem_input = get_input_or_stdin(problem, "Problem name")?;
            let response = client.select(problem_input).await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Commands::Explore { plans } => {
            let plans_input = get_input_or_stdin(plans, "Plans")?;

            let plans_vec: Vec<String> = serde_json::from_str(&plans_input)
                .map_err(|e| anyhow::anyhow!("Invalid JSON format for plans: {}", e))?;

            if plans_vec.is_empty() {
                return Err(anyhow::anyhow!("No valid plans found after parsing"));
            }

            let response = client.explore(plans_vec).await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Commands::Guess { map } => {
            let map_input = get_input_or_stdin(map, "Map JSON")?;
            let map_data: Map = serde_json::from_str(&map_input)
                .map_err(|e| anyhow::anyhow!("Invalid JSON format for map: {}", e))?;
            let response = client.guess(map_data).await?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}
