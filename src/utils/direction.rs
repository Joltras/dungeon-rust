#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    UpRight,
    UpLeft,
    DownRight,
    DownLeft,
}

impl Direction {
    pub const MAIN_DIRECTIONS: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];

    pub const ALL: [Direction; 8] = [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
        Direction::UpRight,
        Direction::UpLeft,
        Direction::DownRight,
        Direction::DownLeft,
    ];

    pub fn value(&self) -> (i8, i8) {
        match self {
            Direction::Up => (0, -1),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::UpRight => (1, -1),
            Direction::UpLeft => (-1, -1),
            Direction::DownRight => (1, 1),
            Direction::DownLeft => (-1, 1),
        }
    }
}
