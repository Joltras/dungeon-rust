use serde::{Deserialize, Serialize};

use crate::utils::Position;
use crate::utils::constants::MAX_DOOR_AMOUNT;
use crate::utils::door_face::DoorFace;
use crate::utils::room_type::RoomType;

/// A room on a floor, represented by its position, type and doors.
///
/// A teleport room has the id of the room it is connected to. Python models it as
/// the subclass `TeleportRoom`; here it is a room with `connected_room_id` set.
/// The field order and names match the json format of the Python generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    #[serde(rename = "_doors")]
    doors: Vec<DoorFace>,
    #[serde(rename = "_id")]
    id: u32,
    #[serde(rename = "_type")]
    room_type: RoomType,
    #[serde(rename = "_x")]
    x: i32,
    #[serde(rename = "_y")]
    y: i32,
    #[serde(
        rename = "_connected_room_id",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    connected_room_id: Option<u32>,
}

impl Room {
    /// Creates a new room without doors.
    pub fn new(x: i32, y: i32, id: u32, room_type: RoomType) -> Self {
        Self {
            doors: Vec::new(),
            id,
            room_type,
            x,
            y,
            connected_room_id: None,
        }
    }

    /// Creates a new teleport room that is connected to the room with the given id.
    pub fn teleport(x: i32, y: i32, id: u32, connected_room_id: u32) -> Self {
        Self {
            connected_room_id: Some(connected_room_id),
            ..Self::new(x, y, id, RoomType::Teleport)
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn position(&self) -> Position {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, (x, y): Position) {
        self.x = x;
        self.y = y;
    }

    pub fn room_type(&self) -> RoomType {
        self.room_type
    }

    pub fn set_type(&mut self, room_type: RoomType) {
        self.room_type = room_type;
    }

    /// Doors of the room in the order they were added.
    pub fn doors(&self) -> &[DoorFace] {
        &self.doors
    }

    /// Adds a door, unless the room already has the maximum amount of doors.
    pub fn add_door(&mut self, door: DoorFace) {
        if self.doors.len() < usize::from(MAX_DOOR_AMOUNT) {
            self.doors.push(door);
        }
    }

    /// Id of the room a teleport room is connected to, `None` for other rooms.
    pub fn connected_room_id(&self) -> Option<u32> {
        self.connected_room_id
    }

    pub fn is_teleport(&self) -> bool {
        self.connected_room_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_like_python() {
        let mut room = Room::new(4, 3, 0, RoomType::Start);
        room.add_door(DoorFace::East);
        room.add_door(DoorFace::Top);
        assert_eq!(
            serde_json::to_string(&room).unwrap(),
            r#"{"_doors":[1,0],"_id":0,"_type":4,"_x":4,"_y":3}"#
        );
        let teleport = Room::teleport(1, 2, 7, 3);
        assert_eq!(
            serde_json::to_string(&teleport).unwrap(),
            r#"{"_doors":[],"_id":7,"_type":5,"_x":1,"_y":2,"_connected_room_id":3}"#
        );
    }

    #[test]
    fn reads_python_json() {
        let room: Room =
            serde_json::from_str(r#"{"_doors":[2],"_id":5,"_type":1,"_x":0,"_y":7}"#).unwrap();
        assert_eq!(room.room_type(), RoomType::DeadEnd);
        assert_eq!(room.doors(), [DoorFace::Bottom]);
        assert!(!room.is_teleport());
    }

    #[test]
    fn has_at_most_four_doors() {
        let mut room = Room::new(0, 0, 0, RoomType::Normal);
        for _ in 0..6 {
            room.add_door(DoorFace::West);
        }
        assert_eq!(room.doors().len(), 4);
    }
}
