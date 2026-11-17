#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}