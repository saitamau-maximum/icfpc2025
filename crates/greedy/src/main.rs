use std::{collections::HashSet, env, time::Instant};

use icfpc2025_client::AedificiumRemoteClient;
use icfpc2025_common::{AedificiumClient, Map, MapConnection, RoomDoor};
use rand::{
    Rng,
    seq::{IndexedRandom, SliceRandom},
};

const N: usize = 6;
const DOORS: usize = 6;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    let team_id = env::var("ICFPC_TEAM_ID").map_err(|_| {
        anyhow::anyhow!(
            "Team ID is required. Set via ICFPC_TEAM_ID environment variable or .env file"
        )
    })?;

    let mut client = AedificiumRemoteClient::new(team_id);

    let mut rng = rand::rng();

    'outer: loop {
        // Select a problem
        let select_response = client.select("primus".to_string()).await?;
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

        let plan_result = explore_response.results[0].clone();

        // backtrack法でグラフ構築
        struct State {
            graph: Vec<Vec<isize>>,
            graph_filled: usize,
            idx: usize,
            current_room: usize,
        }

        let initial_state = State {
            graph: vec![vec![-1; DOORS]; N],
            graph_filled: 0,
            current_room: plan_result[0],
            idx: 0,
        };
        let mut stack = Vec::new();
        stack.push(initial_state);
        let start = Instant::now();

        let mut graphs = Vec::new();

        while let Some(State {
            graph,
            graph_filled,
            idx,
            current_room,
        }) = stack.pop()
        {
            if start.elapsed().as_secs() > 10 {
                continue 'outer;
            }
            if graph_filled == N * DOORS {
                graphs.push(graph.clone());
                // break;
            }
            if idx == max_plans - 1 {
                continue;
            }
            // draw graph
            let door = query[idx];
            let next_room_mod = plan_result[idx + 1];
            let mut next_room_candidate = next_room_mod;
            let mut next_room_candidates = Vec::new();
            while next_room_candidate < N {
                next_room_candidates.push(next_room_candidate);
                next_room_candidate += 4;
            }
            next_room_candidates.shuffle(&mut rng);
            for next_room_candidate in next_room_candidates {
                if graph[current_room][door] == -1
                    || graph[current_room][door] == next_room_candidate as isize
                {
                    let mut graph = graph.clone();
                    graph[current_room][door] = next_room_candidate as isize;
                    // next_room_candidate側に自分向きのドアがあるか確認
                    let mut has_self_doorable = false;
                    for k in 0..DOORS {
                        if graph[next_room_candidate][k] == current_room as isize {
                            has_self_doorable = true;
                            break;
                        }
                        if graph[next_room_candidate][k] == -1 {
                            has_self_doorable = true;
                            break;
                        }
                    }
                    if !has_self_doorable {
                        continue;
                    }
                    let mut new_graph_filled = 0;
                    for i in 0..N {
                        for j in 0..DOORS {
                            if graph[i][j] != -1 {
                                new_graph_filled += 1;
                            }
                        }
                    }
                    stack.push(State {
                        graph,
                        graph_filled: new_graph_filled,
                        idx: idx + 1,
                        current_room: next_room_candidate,
                    });
                }
            }
        }

        let mut map = Map {
            rooms: (0..N).collect(),
            starting_room: plan_result[0],
            connections: vec![],
        };

        eprintln!("graphs.len(): {:?}", graphs.len());
        // println!("graphs: {:?}", graphs);
        for graph_candidate in &graphs {
            eprintln!("graph_candidate: {:?}", graph_candidate);
        }

        let mut real_connections_collection = Vec::new();
        'real_graph: for graph_candidate in &graphs {
            let mut used_room_doors: HashSet<RoomDoor> = HashSet::new();
            let mut real_connections = Vec::new();
            for i in 0..N {
                for j in 0..DOORS {
                    if graph_candidate[i][j] != -1 {
                        let from_door = RoomDoor { room: i, door: j };
                        if used_room_doors.contains(&from_door) {
                            continue;
                        }
                        let next_room_id = graph_candidate[i][j] as usize;
                        let mut reversed_door = None;
                        for k in 0..DOORS {
                            let door = RoomDoor {
                                room: next_room_id,
                                door: k,
                            };
                            if used_room_doors.contains(&door) {
                                continue;
                            }
                            if graph_candidate[next_room_id][k] == i as isize {
                                reversed_door = Some(door);
                                break;
                            }
                        }

                        // assert!(reversed_door.is_some());
                        if reversed_door.is_none() {
                            continue 'real_graph;
                        }
                        let to_door = reversed_door.unwrap();
                        used_room_doors.insert(from_door.clone());
                        used_room_doors.insert(to_door.clone());
                        real_connections.push(MapConnection {
                            from: from_door,
                            to: to_door,
                        });
                    }
                }
            }
            real_connections_collection.push((graph_candidate, real_connections));
        }

        // eprintln!("graph_candidate: {:?}", graph_candidate);
        // 全てのグラフを出す
        for graph_candidate in &graphs {
            eprintln!("graph_candidate: {:?}", graph_candidate);
        }

        eprintln!(
            "real_connections_collection.len(): {:?}",
            real_connections_collection.len()
        );

        if real_connections_collection.is_empty() {
            continue;
        }

        // random select one
        let (graph_candidate, real_connections) = real_connections_collection
            .choose(&mut rng)
            .unwrap()
            .clone();
        map.connections = real_connections;

        let guess_response = client.guess(map).await?;
        if guess_response.correct {
            eprintln!("Guess correct");
            return Ok(());
        } else {
            eprintln!("Guess incorrect");
        }
    }
}
