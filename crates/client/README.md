# ICFPC 2025 Rust API Client

Rust API client for the ICFPC 2025 Aedificium contest.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
icfpc2025-client = { path = "crates/client" }
```

## Usage

```rust
use anyhow::Result;
use icfpc2025_client::{AedificiumClient, RegisterRequest, SelectRequest, ExploreRequest, GuessRequest, Map, MapConnection, RoomDoor};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AedificiumClient::new();

    // Register for the contest
    let register_response = client
        .register(RegisterRequest {
            name: "Team Name".to_string(),
            pl: "Rust".to_string(),
            email: "team@example.com".to_string(),
        })
        .await?;

    let id = register_response.id;

    // Select a problem
    client
        .select(SelectRequest {
            id: id.clone(),
            problem_name: "example-problem".to_string(),
        })
        .await?;

    // Explore routes with batch plans
    let explore_response = client
        .explore(ExploreRequest {
            id: id.clone(),
            plans: vec!["N".to_string(), "S".to_string(), "E".to_string()],
        })
        .await?;

    // Submit a map guess
    let guess_response = client
        .guess(GuessRequest {
            id,
            map: Map {
                rooms: vec![1, 2, 3],
                starting_room: 1,
                connections: vec![MapConnection {
                    from: RoomDoor { room: 1, door: 0 },
                    to: RoomDoor { room: 2, door: 1 },
                }],
            },
        })
        .await?;

    println!("Guess correct: {}", guess_response.correct);
    Ok(())
}
```

## API Endpoints

- `POST /register` - Register for the contest
- `POST /select` - Select a problem
- `POST /explore` - Explore routes with batch plans
- `POST /guess` - Submit a map guess

## Development

From the workspace root:

```bash
cargo check -p icfpc2025-client
cargo run --example basic_usage
```