use serde::{Deserialize, Serialize};

/// Type of a room. The values are the ones used in the json files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "i8", try_from = "i8")]
#[repr(i8)]
pub enum RoomType {
    Normal = 0,
    DeadEnd = 1,
    Item = 2,
    Shop = 3,
    Start = 4,
    Teleport = 5,
    Boss = 6,
    Secret = 7,
    SuperSecret = 8,
}

impl RoomType {
    /// Rooms that are placed on random dead ends.
    pub const SPECIAL_ROOMS: [RoomType; 2] = [RoomType::Item, RoomType::Shop];

    pub const ALL: [RoomType; 9] = [
        RoomType::Normal,
        RoomType::DeadEnd,
        RoomType::Item,
        RoomType::Shop,
        RoomType::Start,
        RoomType::Teleport,
        RoomType::Boss,
        RoomType::Secret,
        RoomType::SuperSecret,
    ];

    pub const VALUES: [i8; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];

    pub fn name(&self) -> &'static str {
        match self {
            RoomType::Normal => "Normal",
            RoomType::DeadEnd => "DeadEnd",
            RoomType::Item => "Item",
            RoomType::Shop => "Shop",
            RoomType::Start => "Start",
            RoomType::Teleport => "Teleport",
            RoomType::Boss => "Boss",
            RoomType::Secret => "Secret",
            RoomType::SuperSecret => "SuperSecret",
        }
    }

    /// Name for texts shown to the user, e.g. "Dead End".
    pub fn display_name(&self) -> &'static str {
        match self {
            RoomType::Normal => "Normal Room",
            RoomType::DeadEnd => "Dead End",
            RoomType::Item => "Item Room",
            RoomType::Shop => "Shop Room",
            RoomType::Start => "Start Room",
            RoomType::Teleport => "Teleport Room",
            RoomType::Boss => "Boss Room",
            RoomType::Secret => "Secret Room",
            RoomType::SuperSecret => "Super Secret Room",
        }
    }

    /// Returns true for the rooms placed on random dead ends (item and shop room).
    pub fn is_special_room(&self) -> bool {
        Self::SPECIAL_ROOMS.contains(self)
    }

    /// Returns true for rooms with a special function:
    /// item, shop, boss, secret and super secret room.
    pub fn is_special(&self) -> bool {
        matches!(
            self,
            RoomType::Item
                | RoomType::Shop
                | RoomType::Boss
                | RoomType::Secret
                | RoomType::SuperSecret
        )
    }
}

impl From<RoomType> for i8 {
    fn from(room_type: RoomType) -> Self {
        room_type as i8
    }
}

impl TryFrom<i8> for RoomType {
    type Error = String;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        RoomType::ALL
            .into_iter()
            .find(|room_type| *room_type as i8 == value)
            .ok_or_else(|| format!("{value} is not a valid room type"))
    }
}
