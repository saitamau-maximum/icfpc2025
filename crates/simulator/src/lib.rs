use anyhow::{bail, Result};
use async_trait::async_trait;
use icfpc2025_common::{
    ExploreResponse, GuessResponse, Map, MapConnection, RoomDoor, SelectResponse,
};
use rand::prelude::Rng;
use std::collections::{HashMap, HashSet, VecDeque};

// Re-export the trait for convenience
pub use icfpc2025_common::AedificiumClient;

const DOORS: usize = 6;

#[derive(Debug, Clone)]
pub struct Room {
    pub label: usize,
    pub connections: [Option<usize>; DOORS], // Door 0-5 connections to other rooms
}

impl Room {
    pub fn new(label: usize) -> Self {
        Self {
            label,
            connections: [None; DOORS],
        }
    }

    pub fn connect_door(&mut self, door: usize, room_id: usize) {
        if door < DOORS {
            self.connections[door] = Some(room_id);
        }
    }
}

#[derive(Debug)]
pub struct Library {
    rooms: HashMap<usize, Room>,
    starting_room: usize,
    room_count: usize,
}

impl Library {
    pub fn generate(room_count: usize, rng: &mut impl Rng) -> Result<Self> {
        if room_count == 0 {
            bail!("Library must have at least one room");
        }

        let mut library = Self {
            rooms: HashMap::new(),
            starting_room: 0,
            room_count,
        };

        // Create rooms with
        for i in 0..room_count {
            let label = rng.gen_range(0..room_count);
            library.rooms.insert(i, Room::new(label));
        }

        // Generate connections to ensure the library is connected
        library.generate_connections(rng)?;

        Ok(library)
    }

    fn generate_connections(&mut self, rng: &mut impl Rng) -> Result<()> {
        // Use a modified version of Kruskal's algorithm to create a connected graph
        let mut connected = HashSet::new();
        let mut to_connect = VecDeque::new();

        // Start with room 0
        connected.insert(0);
        to_connect.push_back(0);

        while connected.len() < self.room_count && !to_connect.is_empty() {
            let current_room = to_connect.pop_front().unwrap();

            // Try to connect to unconnected rooms
            let available_doors: Vec<usize> = (0..DOORS)
                .filter(|&door| self.rooms[&current_room].connections[door].is_none())
                .collect();

            if !available_doors.is_empty() {
                // Pick a random available door
                let door = available_doors[rng.gen_range(0..available_doors.len())];

                // Find an unconnected room to connect to
                let unconnected: Vec<usize> = (0..self.room_count)
                    .filter(|&id| !connected.contains(&id))
                    .collect();

                if !unconnected.is_empty() {
                    let target_room = unconnected[rng.gen_range(0..unconnected.len())];

                    // Find available door in target room
                    let target_doors: Vec<usize> = (0..DOORS)
                        .filter(|&d| self.rooms[&target_room].connections[d].is_none())
                        .collect();

                    if !target_doors.is_empty() {
                        let target_door = target_doors[rng.gen_range(0..target_doors.len())];

                        // Create bidirectional connection
                        self.rooms.get_mut(&current_room).unwrap().connections[door] =
                            Some(target_room);
                        self.rooms.get_mut(&target_room).unwrap().connections[target_door] =
                            Some(current_room);

                        connected.insert(target_room);
                        to_connect.push_back(target_room);
                        to_connect.push_back(current_room); // Re-queue current room for more connections
                    }
                }
            }
        }

        // Add some additional random connections to make the graph more interesting
        for _ in 0..(self.room_count / 2) {
            let room1 = rng.gen_range(0..self.room_count);
            let room2 = rng.gen_range(0..self.room_count);

            if room1 != room2 {
                let available_doors1: Vec<usize> = (0..DOORS)
                    .filter(|&door| self.rooms[&room1].connections[door].is_none())
                    .collect();
                let available_doors2: Vec<usize> = (0..DOORS)
                    .filter(|&door| self.rooms[&room2].connections[door].is_none())
                    .collect();

                if !available_doors1.is_empty() && !available_doors2.is_empty() {
                    let door1 = available_doors1[rng.gen_range(0..available_doors1.len())];
                    let door2 = available_doors2[rng.gen_range(0..available_doors2.len())];

                    self.rooms.get_mut(&room1).unwrap().connections[door1] = Some(room2);
                    self.rooms.get_mut(&room2).unwrap().connections[door2] = Some(room1);
                }
            }
        }

        Ok(())
    }

    pub fn max_doorways(&self) -> usize {
        18 * self.room_count
    }
}

#[derive(Debug)]
pub struct Simulator {
    library: Library,
    current_doorways_used: usize,
}

impl Simulator {
    pub fn new(room_count: usize, rng: &mut impl Rng) -> Result<Self> {
        let library = Library::generate(room_count, rng)?;
        Ok(Self {
            library,
            current_doorways_used: 0,
        })
    }

    fn _explore(&mut self, plans: Vec<String>) -> Result<ExploreResponse> {
        let mut results = Vec::new();

        for plan in plans {
            let mut current_room = self.library.starting_room;
            let mut room_labels = Vec::new();

            // Start with the starting room's label
            room_labels.push(self.library.rooms[&current_room].label);
            self.current_doorways_used += 1;

            // Follow the plan
            for door_char in plan.chars() {
                if self.current_doorways_used >= self.library.max_doorways() {
                    bail!("Maximum doorways exceeded for this library");
                }

                let door = match door_char.to_digit(10) {
                    Some(d) if d < DOORS as u32 => d as usize,
                    _ => bail!("Invalid door number in plan: {}", door_char),
                };

                match self.library.rooms[&current_room].connections[door] {
                    Some(next_room) => {
                        current_room = next_room;
                        room_labels.push(self.library.rooms[&current_room].label);
                        self.current_doorways_used += 1;
                    }
                    None => {
                        // Dead end - exploration stops here
                        break;
                    }
                }
            }

            results.push(room_labels);
        }

        Ok(ExploreResponse {
            results: results
                .iter()
                .map(|x| x.iter().map(|y| y % 4).collect())
                .collect(),
            query_count: self.current_doorways_used,
        })
    }

    fn _guess(&self, map: Map) -> Result<GuessResponse> {
        // Verify the map matches the actual library structure

        // Check if starting room matches
        if map.starting_room != self.library.starting_room {
            return Ok(GuessResponse { correct: false });
        }

        // Check if all rooms are present with correct labels
        let mut expected_rooms: HashMap<usize, usize> = self
            .library
            .rooms
            .iter()
            .map(|(id, room)| (*id, room.label))
            .collect();

        for &room_id in &map.rooms {
            if !expected_rooms.contains_key(&room_id) {
                return Ok(GuessResponse { correct: false });
            }
            expected_rooms.remove(&room_id);
        }

        if !expected_rooms.is_empty() {
            return Ok(GuessResponse { correct: false });
        }

        // Check connections
        let mut expected_connections = HashSet::new();

        for (room_id, room) in &self.library.rooms {
            for (door, &connected_room) in room.connections.iter().enumerate() {
                if let Some(connected_room) = connected_room {
                    // Add connection in canonical form (smaller room first)
                    let conn = if *room_id < connected_room {
                        MapConnection {
                            from: RoomDoor {
                                room: *room_id,
                                door,
                            },
                            to: RoomDoor {
                                room: connected_room,
                                door: self.find_reverse_door(*room_id, door, connected_room),
                            },
                        }
                    } else {
                        MapConnection {
                            from: RoomDoor {
                                room: connected_room,
                                door: self.find_reverse_door(*room_id, door, connected_room),
                            },
                            to: RoomDoor {
                                room: *room_id,
                                door,
                            },
                        }
                    };
                    expected_connections.insert(conn);
                }
            }
        }

        let mut provided_connections = HashSet::new();
        for conn in &map.connections {
            // Normalize connection order
            let normalized_conn = if conn.from.room < conn.to.room {
                conn.clone()
            } else {
                MapConnection {
                    from: conn.to.clone(),
                    to: conn.from.clone(),
                }
            };
            provided_connections.insert(normalized_conn);
        }

        Ok(GuessResponse {
            correct: expected_connections == provided_connections,
        })
    }

    fn find_reverse_door(&self, from_room: usize, _from_door: usize, to_room: usize) -> usize {
        if let Some(to_room_data) = self.library.rooms.get(&to_room) {
            for (door, &connected) in to_room_data.connections.iter().enumerate() {
                if connected == Some(from_room) {
                    return door;
                }
            }
        }
        0 // Fallback, shouldn't happen in a well-formed library
    }

    pub fn get_library_info(&self) -> (usize, usize) {
        (self.library.room_count, self.current_doorways_used)
    }

    pub fn reset_exploration(&mut self) {
        self.current_doorways_used = 0;
    }

    pub fn get_actual_map(&self) -> Map {
        let mut connections = Vec::new();
        let rooms: Vec<usize> = self.library.rooms.keys().cloned().collect();

        for (room_id, room) in &self.library.rooms {
            for (door, &connected_room) in room.connections.iter().enumerate() {
                if let Some(connected_room) = connected_room {
                    // Only add each connection once (avoid duplicates)
                    if *room_id < connected_room {
                        let reverse_door = self.find_reverse_door(*room_id, door, connected_room);
                        connections.push(MapConnection {
                            from: RoomDoor {
                                room: *room_id,
                                door,
                            },
                            to: RoomDoor {
                                room: connected_room,
                                door: reverse_door,
                            },
                        });
                    }
                }
            }
        }

        Map {
            rooms,
            starting_room: self.library.starting_room,
            connections,
        }
    }

    pub fn remaining_doorways(&self) -> usize {
        self.library
            .max_doorways()
            .saturating_sub(self.current_doorways_used)
    }
}

#[async_trait]
impl AedificiumClient for Simulator {
    async fn select(&self, _problem_name: String) -> Result<SelectResponse> {
        unimplemented!()
    }

    async fn explore(&mut self, plans: Vec<String>) -> Result<ExploreResponse> {
        self._explore(plans)
    }

    async fn guess(&self, data: Map) -> Result<GuessResponse> {
        self._guess(data)
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, SeedableRng};

    use super::*;

    #[test]
    fn test_library_generation() {
        let mut rng = StdRng::seed_from_u64(42);
        let library = Library::generate(5, &mut rng).unwrap();
        assert_eq!(library.rooms.len(), 5);
        assert_eq!(library.starting_room, 0);
    }

    #[test]
    fn test_simulator_creation() {
        let mut rng = StdRng::seed_from_u64(123);
        let simulator = Simulator::new(3, &mut rng).unwrap();
        let (room_count, doorways_used) = simulator.get_library_info();
        assert_eq!(room_count, 3);
        assert_eq!(doorways_used, 0);
    }

    #[tokio::test]
    async fn test_simple_exploration() {
        let mut rng = StdRng::seed_from_u64(456);
        let mut simulator = Simulator::new(6, &mut rng).unwrap();
        let response = simulator
            .explore(vec![
                "0".to_string(),
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
            ])
            .await
            .unwrap();

        assert!(!response.results.is_empty());
        assert!(response.query_count > 0);
        assert_eq!(response.results.len(), 6);
        for result in response.results {
            for observed_room in result {
                assert!(observed_room < 4);
            }
        }
    }
}
