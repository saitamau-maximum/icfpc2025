use std::{collections::HashSet, env};

use icfpc2025_client::AedificiumRemoteClient;
use icfpc2025_common::{AedificiumClient, Map, MapConnection, RoomDoor};
use rand::Rng;

const N: usize = 3;
const EDGES: usize = 6;
const RETRY_COUNT: usize = 10;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    let team_id = env::var("ICFPC_TEAM_ID").map_err(|_| {
        anyhow::anyhow!(
            "Team ID is required. Set via ICFPC_TEAM_ID environment variable or .env file"
        )
    })?;

    let client = AedificiumRemoteClient::new(team_id);

    let mut rng = rand::rng();

    for _ in 0..RETRY_COUNT {
        // Select a problem
        let select_response = client.select("probatio".to_string()).await?;
        println!("Selected problem: {:?}", select_response);

        // Explore with some plans
        let max_plans = 18 * N;

        // generate random [0~5]{max_plans} string
        let query = (0..max_plans)
            .map(|_| rng.random_range(0..=5))
            .collect::<Vec<usize>>();
        println!("Query: {:?}", query);

        let explore_response = client
            .explore(vec![
                query
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(""),
            ])
            .await?;
        println!("Explore response: {:?}", explore_response);

        let mut graph: Vec<Vec<i32>> = vec![vec![-1; EDGES]; N];
        let plan_result = explore_response.results[0].clone();

        let mut current_room_id = plan_result[0];

        assert!(plan_result.len() == query.len() + 1);

        for i in 0..query.len() {
            let door_type = query[i];
            let next_room_id = plan_result[i + 1];
            graph[current_room_id][door_type] = next_room_id as i32;
            current_room_id = next_room_id;
        }

        // validate if -1 is not in the graph
        let mut has_minus_one = false;
        println!("Graph:");
        for row in graph.iter() {
            for &value in row.iter() {
                print!("{} ", value);
                if value == -1 {
                    has_minus_one = true;
                }
            }
            println!();
        }
        if has_minus_one {
            println!("Graph contains -1");
            continue;
        }

        let mut map = Map {
            rooms: (0..N).collect(),
            starting_room: plan_result[0],
            connections: vec![],
        };

        let mut used_room_doors: HashSet<RoomDoor> = HashSet::new();

        for i in 0..N {
            for j in 0..EDGES {
                if graph[i][j] != -1 {
                    let from_door = RoomDoor { room: i, door: j };
                    if used_room_doors.contains(&from_door) {
                        continue;
                    }
                    let next_room_id = graph[i][j] as usize;
                    let mut reversed_door = None;
                    for k in 0..EDGES {
                        let door = RoomDoor {
                            room: next_room_id,
                            door: k,
                        };
                        if used_room_doors.contains(&door) {
                            continue;
                        }
                        if graph[next_room_id][k] == i as i32 {
                            reversed_door = Some(door);
                            break;
                        }
                    }

                    assert!(reversed_door.is_some());
                    let to_door = reversed_door.unwrap();
                    used_room_doors.insert(from_door.clone());
                    used_room_doors.insert(to_door.clone());
                    map.connections.push(MapConnection {
                        from: from_door,
                        to: to_door,
                    });
                }
            }
        }

        let guess_response = client.guess(map).await?;
        if guess_response.correct {
            eprintln!("Guess correct");
            return Ok(());
        } else {
            eprintln!("Guess incorrect");
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
