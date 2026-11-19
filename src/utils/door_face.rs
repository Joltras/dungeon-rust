use serde::{Deserialize, Serialize};

use super::direction::Direction;

/// Side of a room a door is on. The values are the ones used in the json files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "i8", try_from = "i8")]
#[repr(i8)]
pub enum DoorFace {
    Top = 0,
    East = 1,
    Bottom = 2,
    West = 3,
}

impl DoorFace {
    pub const ALL: [DoorFace; 4] = [
        DoorFace::Top,
        DoorFace::East,
        DoorFace::Bottom,
        DoorFace::West,
    ];

    pub fn values() -> [i8; 4] {
        [
            DoorFace::Top as i8,
            DoorFace::East as i8,
            DoorFace::Bottom as i8,
            DoorFace::West as i8,
        ]
    }

    /// Direction from the room to the room behind the door.
    pub fn direction(&self) -> Direction {
        match self {
            DoorFace::Top => Direction::Up,
            DoorFace::East => Direction::Right,
            DoorFace::Bottom => Direction::Down,
            DoorFace::West => Direction::Left,
        }
    }

    /// Name for texts shown to the user.
    pub fn display_name(&self) -> &'static str {
        match self {
            DoorFace::Top => "top",
            DoorFace::East => "right",
            DoorFace::Bottom => "bottom",
            DoorFace::West => "left",
        }
    }
}

impl From<DoorFace> for i8 {
    fn from(door: DoorFace) -> Self {
        door as i8
    }
}

impl TryFrom<i8> for DoorFace {
    type Error = String;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        DoorFace::ALL
            .into_iter()
            .find(|door| *door as i8 == value)
            .ok_or_else(|| format!("{value} is not a valid door face"))
    }
}
