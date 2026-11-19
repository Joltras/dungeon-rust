//! Describes rooms of a floor in a short text, e.g. for tooltips.

use crate::floor::Floor;
use crate::room::Room;
use crate::utils::room_type::RoomType;

/// Describes a room with its type, id, position, doors and connections,
/// with one fact per line.
pub fn describe_room(floor: &Floor, room: &Room) -> String {
    let mut lines = vec![
        format!("{} · #{}", room.room_type().display_name(), room.id()),
        format!("Position {}, {}", room.x(), room.y()),
    ];
    let mut doors = room.doors().to_vec();
    doors.sort();
    let door_names: Vec<&str> = doors.iter().map(|door| door.display_name()).collect();
    lines.push(if door_names.is_empty() {
        "Doors: none".to_owned()
    } else {
        format!("Doors: {}", door_names.join(", "))
    });

    if let Some(target) = room
        .connected_room_id()
        .and_then(|id| floor.get_room_by_id(id))
    {
        lines.push(format!(
            "Teleports to #{} ({})",
            target.id(),
            target.room_type().display_name()
        ));
    }
    if room.room_type() == RoomType::Boss {
        let cells = floor
            .rooms()
            .iter()
            .filter(|other| other.room_type() == RoomType::Boss)
            .count();
        let shape = match cells {
            2 => Some("1 × 2"),
            4 => Some("2 × 2"),
            _ => None,
        };
        if let Some(shape) = shape {
            lines.push(format!("Part of a {shape} boss room"));
        }
    }
    for other in floor.rooms() {
        if other.connected_room_id() == Some(room.id()) {
            lines.push(format!("Reached through teleport #{}", other.id()));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_teleport_and_boss_room() {
        let json = r#"{"_width": 9, "_height": 8, "_floor": {"_seed": "", "_rooms": [
            {"_doors": [], "_id": 0, "_type": 6, "_x": 0, "_y": 0},
            {"_doors": [3, 0], "_id": 1, "_type": 5, "_x": 4, "_y": 3, "_connected_room_id": 0}]}}"#;
        let floor = Floor::from_json(json).unwrap();
        assert_eq!(
            describe_room(&floor, &floor.rooms()[1]),
            "Teleport Room · #1\nPosition 4, 3\nDoors: top, left\nTeleports to #0 (Boss Room)"
        );
        assert_eq!(
            describe_room(&floor, &floor.rooms()[0]),
            "Boss Room · #0\nPosition 0, 0\nDoors: none\nReached through teleport #1"
        );
    }
}
