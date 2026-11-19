use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::room::Room;
use crate::utils::Position;
use crate::utils::direction::Direction;
use crate::utils::door_face::DoorFace;
use crate::utils::room_type::RoomType;
use crate::utils::util_functions::add_direction_to_coordinates;

/// Name written to `_generated_by` in the json files.
const GENERATED_BY: &str = "rust";

/// A floor is a grid of cells with the rooms placed on it.
#[derive(Debug, Clone)]
pub struct Floor {
    rooms: Vec<Room>,
    /// Cells that contain a room, indexed by `[y][x]`
    grid: Vec<Vec<bool>>,
    width: i32,
    height: i32,
    next_room_id: u32,
    seed: String,
    /// Id of the floor the rooms were generated for, unknown for old files
    floor_id: Option<u32>,
}

/// Json format of a floor file, compatible with the Python generator.
#[derive(Serialize, Deserialize)]
struct FloorFile {
    #[serde(rename = "_width")]
    width: i32,
    #[serde(rename = "_height")]
    height: i32,
    #[serde(rename = "_floor_id", default, skip_serializing_if = "Option::is_none")]
    floor_id: Option<u32>,
    #[serde(rename = "_floor")]
    floor: FloorData,
    #[serde(
        rename = "_generated_by",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    generated_by: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct FloorData {
    #[serde(rename = "_seed")]
    seed: String,
    #[serde(rename = "_rooms")]
    rooms: Vec<Room>,
}

impl Floor {
    /// Creates an empty floor with the given size.
    pub fn new(height: i32, width: i32, seed: impl Into<String>) -> Self {
        Self {
            rooms: Vec::new(),
            grid: vec![vec![false; width as usize]; height as usize],
            width,
            height,
            next_room_id: 0,
            seed: seed.into(),
            floor_id: None,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn seed(&self) -> &str {
        &self.seed
    }

    pub fn floor_id(&self) -> Option<u32> {
        self.floor_id
    }

    pub fn set_floor_id(&mut self, floor_id: Option<u32>) {
        self.floor_id = floor_id;
    }

    pub fn top_left(&self) -> Position {
        (0, 0)
    }

    pub fn top_right(&self) -> Position {
        (self.width - 1, 0)
    }

    pub fn bottom_left(&self) -> Position {
        (0, self.height - 1)
    }

    pub fn bottom_right(&self) -> Position {
        (self.width - 1, self.height - 1)
    }

    /// All rooms in the order they were added.
    pub fn rooms(&self) -> &[Room] {
        &self.rooms
    }

    pub(crate) fn room_mut(&mut self, index: usize) -> &mut Room {
        &mut self.rooms[index]
    }

    /// Marks the cell as containing a room without adding a room.
    pub fn add_to_floor_grid(&mut self, (x, y): Position) {
        self.grid[y as usize][x as usize] = true;
    }

    /// Creates a room with the next id and adds it to the floor.
    pub fn add_room(&mut self, x: i32, y: i32, room_type: RoomType) {
        self.add_to_floor_grid((x, y));
        self.rooms
            .push(Room::new(x, y, self.next_room_id, room_type));
        self.next_room_id += 1;
    }

    /// Creates a room next to the room with the given index.
    pub fn add_room_next_to(&mut self, index: usize, direction: Direction, room_type: RoomType) {
        let (x, y) = add_direction_to_coordinates(direction, self.rooms[index].position());
        self.add_room(x, y, room_type);
    }

    /// Creates a teleport room at the position of the room with the given index,
    /// connected to that room. The cell is not marked again.
    pub fn add_teleport_room(&mut self, index: usize) {
        let target = &self.rooms[index];
        let teleport = Room::teleport(target.x(), target.y(), self.next_room_id, target.id());
        self.rooms.push(teleport);
        self.next_room_id += 1;
    }

    /// Returns if the cell contains a room. Cells outside the floor contain no room.
    pub fn contains_room(&self, (x, y): Position) -> bool {
        self.is_within_border((x, y)) && self.grid[y as usize][x as usize]
    }

    /// Returns if the position is a cell of the floor.
    pub fn is_within_border(&self, (x, y): Position) -> bool {
        (0..self.width).contains(&x) && (0..self.height).contains(&y)
    }

    /// Adds the doors to all rooms, for every neighbouring cell with a room.
    pub fn add_doors_to_rooms(&mut self) {
        // The order of the doors is the one of the Python generator
        const DOORS: [(DoorFace, Position); 4] = [
            (DoorFace::East, (1, 0)),
            (DoorFace::West, (-1, 0)),
            (DoorFace::Top, (0, -1)),
            (DoorFace::Bottom, (0, 1)),
        ];
        for index in 0..self.rooms.len() {
            let (x, y) = self.rooms[index].position();
            for (door, (delta_x, delta_y)) in DOORS {
                if self.contains_room((x + delta_x, y + delta_y)) {
                    self.rooms[index].add_door(door);
                }
            }
        }
    }

    /// Counts the rooms directly above, below, left and right of the cell.
    pub fn count_neighbours(&self, position: Position) -> usize {
        Direction::MAIN_DIRECTIONS
            .into_iter()
            .filter(|direction| {
                self.contains_room(add_direction_to_coordinates(*direction, position))
            })
            .count()
    }

    /// Returns if the cell has exactly one neighbour.
    pub fn is_dead_end(&self, position: Position) -> bool {
        self.count_neighbours(position) == 1
    }

    pub fn has_no_neighbours(&self, position: Position) -> bool {
        self.count_neighbours(position) == 0
    }

    /// Returns the first room at the position.
    pub fn get_room(&self, position: Position) -> Option<&Room> {
        self.rooms.iter().find(|room| room.position() == position)
    }

    /// Returns the room with the id.
    pub fn get_room_by_id(&self, id: u32) -> Option<&Room> {
        self.rooms.iter().find(|room| room.id() == id)
    }

    /// Returns the first boss room cell.
    pub fn get_boss_room(&self) -> Option<&Room> {
        self.rooms
            .iter()
            .find(|room| room.room_type() == RoomType::Boss)
    }

    /// Returns if a room above, below, left or right of the cell is a boss room.
    pub fn has_boss_room_as_neighbour(&self, position: Position) -> bool {
        Direction::MAIN_DIRECTIONS.into_iter().any(|direction| {
            let neighbour = add_direction_to_coordinates(direction, position);
            self.contains_room(neighbour)
                && self
                    .get_room(neighbour)
                    .is_some_and(|room| room.room_type() == RoomType::Boss)
        })
    }

    /// Groups the rooms by how many doors away from the start room they are,
    /// following teleport rooms to their target. All cells of a big boss room
    /// are in the same group and secret rooms are hidden, so they come last.
    /// Each group is sorted by room id.
    pub fn rooms_by_distance(&self) -> Vec<Vec<&Room>> {
        let is_hidden =
            |room: &Room| matches!(room.room_type(), RoomType::Secret | RoomType::SuperSecret);
        let start: Vec<&Room> = self
            .rooms
            .iter()
            .filter(|room| room.room_type() == RoomType::Start)
            .collect();
        let mut seen: HashSet<u32> = start.iter().map(|room| room.id()).collect();
        let mut rings: Vec<Vec<&Room>> = Vec::new();
        let mut current = start;
        while !current.is_empty() {
            let mut next_ring: Vec<&Room> = Vec::new();
            for room in &current {
                for neighbour in self.connected_rooms(room) {
                    if is_hidden(neighbour) || !seen.insert(neighbour.id()) {
                        continue;
                    }
                    next_ring.push(neighbour);
                    if neighbour.room_type() == RoomType::Boss {
                        // All cells of a big boss room appear together
                        for boss_cell in &self.rooms {
                            if boss_cell.room_type() == RoomType::Boss
                                && seen.insert(boss_cell.id())
                            {
                                next_ring.push(boss_cell);
                            }
                        }
                    }
                }
            }
            next_ring.sort_by_key(|room| room.id());
            rings.push(std::mem::replace(&mut current, next_ring));
        }
        let mut remaining: Vec<&Room> = self
            .rooms
            .iter()
            .filter(|room| !seen.contains(&room.id()))
            .collect();
        remaining.sort_by_key(|room| room.id());
        if !remaining.is_empty() {
            rings.push(remaining);
        }
        rings
    }

    /// Rooms behind the doors of the room and the target of a teleport room.
    fn connected_rooms(&self, room: &Room) -> Vec<&Room> {
        let mut connected: Vec<&Room> = room
            .doors()
            .iter()
            .filter_map(|door| {
                self.get_room(add_direction_to_coordinates(
                    door.direction(),
                    room.position(),
                ))
            })
            .collect();
        if let Some(target) = room
            .connected_room_id()
            .and_then(|id| self.get_room_by_id(id))
        {
            connected.push(target);
        }
        connected
    }

    /// Counts the rooms of every type.
    pub fn count_room_types(&self) -> HashMap<RoomType, usize> {
        let mut counts = HashMap::new();
        for room in &self.rooms {
            *counts.entry(room.room_type()).or_insert(0) += 1;
        }
        counts
    }

    fn to_file(&self) -> FloorFile {
        FloorFile {
            width: self.width,
            height: self.height,
            floor_id: self.floor_id,
            floor: FloorData {
                seed: self.seed.clone(),
                rooms: self.rooms.clone(),
            },
            generated_by: Some(GENERATED_BY.to_owned()),
        }
    }

    /// Json value of the floor file, e.g. to compare floors.
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self.to_file()).expect("a floor can always be serialized")
    }

    /// Creates the json file content, in the format of the Python generator.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.to_file()).expect("a floor can always be serialized")
    }

    /// Reads a floor from a json file created by the Python or Rust generator.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let file: FloorFile = serde_json::from_str(json)?;
        let mut floor = Floor::new(file.height, file.width, file.floor.seed);
        for room in file.floor.rooms {
            if !floor.is_within_border(room.position()) {
                return Err(serde::de::Error::custom(format!(
                    "room {} at {:?} is outside of the floor",
                    room.id(),
                    room.position()
                )));
            }
            floor.add_to_floor_grid(room.position());
            floor.rooms.push(room);
        }
        floor.next_room_id = floor
            .rooms
            .iter()
            .map(|room| room.id() + 1)
            .max()
            .unwrap_or(0);
        floor.floor_id = file.floor_id;
        Ok(floor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Floor with a start room, a room right of it and a teleport to a boss room.
    fn small_floor() -> Floor {
        let mut floor = Floor::new(8, 9, "seed");
        floor.add_room(4, 3, RoomType::Start);
        floor.add_room(0, 0, RoomType::Boss);
        floor.add_room(4, 4, RoomType::Secret);
        floor.add_teleport_room(1);
        floor.room_mut(3).set_position((5, 3));
        floor.add_to_floor_grid((5, 3));
        floor.add_doors_to_rooms();
        floor
    }

    #[test]
    fn counts_neighbours_and_ignores_cells_outside() {
        let mut floor = Floor::new(8, 9, "");
        floor.add_room(0, 0, RoomType::Start);
        floor.add_room(1, 0, RoomType::Normal);
        assert_eq!(floor.count_neighbours((0, 0)), 1);
        assert!(floor.is_dead_end((1, 0)));
        assert!(floor.has_no_neighbours((8, 7)));
        assert!(!floor.contains_room((-1, 0)));
    }

    #[test]
    fn adds_doors_in_python_order() {
        let mut floor = Floor::new(8, 9, "");
        floor.add_room(4, 3, RoomType::Start);
        for (x, y) in [(4, 2), (5, 3), (4, 4), (3, 3)] {
            floor.add_room(x, y, RoomType::Normal);
        }
        floor.add_doors_to_rooms();
        assert_eq!(
            floor.rooms()[0].doors(),
            [
                DoorFace::East,
                DoorFace::West,
                DoorFace::Top,
                DoorFace::Bottom
            ]
        );
    }

    #[test]
    fn rooms_by_distance_follows_teleports_and_hides_secret_rooms() {
        let floor = small_floor();
        let rings: Vec<Vec<RoomType>> = floor
            .rooms_by_distance()
            .iter()
            .map(|ring| ring.iter().map(|room| room.room_type()).collect())
            .collect();
        assert_eq!(
            rings,
            [
                vec![RoomType::Start],
                vec![RoomType::Teleport],
                vec![RoomType::Boss],
                vec![RoomType::Secret],
            ]
        );
    }

    #[test]
    fn json_round_trip() {
        let mut floor = small_floor();
        floor.set_floor_id(Some(3));
        let loaded = Floor::from_json(&floor.to_json()).unwrap();
        assert_eq!(loaded.rooms(), floor.rooms());
        assert_eq!(loaded.seed(), "seed");
        assert_eq!(loaded.floor_id(), Some(3));
        assert!(loaded.contains_room((0, 0)));
        assert_eq!(loaded.next_room_id, 4);
    }

    #[test]
    fn reads_old_python_files_without_floor_id() {
        let json = r#"{"_width": 9, "_height": 8, "_floor": {"_seed": "abc", "_rooms": [
            {"_doors": [], "_id": 0, "_type": 4, "_x": 4, "_y": 3}]}}"#;
        let floor = Floor::from_json(json).unwrap();
        assert_eq!(floor.floor_id(), None);
        assert_eq!(floor.rooms().len(), 1);
    }

    #[test]
    fn rejects_rooms_outside_of_the_floor() {
        let json = r#"{"_width": 9, "_height": 8, "_floor": {"_seed": "", "_rooms": [
            {"_doors": [], "_id": 0, "_type": 4, "_x": 9, "_y": 3}]}}"#;
        assert!(Floor::from_json(json).is_err());
    }
}
