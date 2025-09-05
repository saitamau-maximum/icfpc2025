use anyhow::Result;
use icfpc2025_client::{AedificiumClient, Map, MapConnection, RoomDoor};

#[tokio::main]
async fn main() -> Result<()> {
    let client = AedificiumClient::new("example-id".to_string());

    // Select a problem
    let _select_response = client.select("example-problem".to_string()).await?;

    println!("Problem selected");

    // Explore routes
    let explore_response = client
        .explore(vec![
            "N".to_string(),
            "S".to_string(),
            "E".to_string(),
            "W".to_string(),
        ])
        .await?;

    println!("Explore results: {:?}", explore_response.results);
    println!("Query count: {}", explore_response.query_count);

    // Submit a guess
    let guess_response = client
        .guess(Map {
            rooms: vec![1, 2, 3],
            starting_room: 1,
            connections: vec![MapConnection {
                from: RoomDoor { room: 1, door: 0 },
                to: RoomDoor { room: 2, door: 1 },
            }],
        })
        .await?;

    println!("Guess correct: {}", guess_response.correct);

    Ok(())
}
