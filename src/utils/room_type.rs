#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum RoomType {
    Normal = 0,
    DeadEnd = -1,
    Item = 2,
    Shop = 3,
    Start = 4,
    Teleport = 5,
    Boss = 6,
    Secret = 7,
    SuperSecret = 8,
}

impl RoomType {

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

    pub const VALUES: [i8; 9] = [0, -1, 2, 3, 4, 5, 6, 7, 8];

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
            RoomType::SuperSecret => "SuperSecret"
        }
    }

    pub fn is_special_room(&self) -> bool {
        Self::SPECIAL_ROOMS.contains(self)
    }
}