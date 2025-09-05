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

    println!("Registered with ID: {}", register_response.id);
    let id = register_response.id;

    // Select a problem
    let _select_response = client
        .select(SelectRequest {
            id: id.clone(),
            problem_name: "example-problem".to_string(),
        })
        .await?;

    println!("Problem selected");

    // Explore routes
    let explore_response = client
        .explore(ExploreRequest {
            id: id.clone(),
            plans: vec!["N".to_string(), "S".to_string(), "E".to_string(), "W".to_string()],
        })
        .await?;

    println!("Explore results: {:?}", explore_response.results);
    println!("Query count: {}", explore_response.query_count);

    // Submit a guess
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