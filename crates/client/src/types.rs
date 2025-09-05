use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest {
    pub name: String,
    pub pl: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectRequest {
    pub id: String,
    #[serde(rename = "problemName")]
    pub problem_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExploreRequest {
    pub id: String,
    pub plans: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreResponse {
    pub results: Vec<Vec<i32>>,
    #[serde(rename = "queryCount")]
    pub query_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConnection {
    pub from: RoomDoor,
    pub to: RoomDoor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDoor {
    pub room: i32,
    pub door: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub rooms: Vec<i32>,
    #[serde(rename = "startingRoom")]
    pub starting_room: i32,
    pub connections: Vec<MapConnection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuessRequest {
    pub id: String,
    pub map: Map,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuessResponse {
    pub correct: bool,
}
