// Room constants
pub const MAX_DOOR_AMOUNT: u8 = 4;
pub const ROOM_WIDTH: i32 = 120;
pub const ROOM_HEIGHT: i32 = 60;

pub const BASE_INDENT: &str = "  ";
pub const JSON_SUFFIX: &str = ".json";
pub const DEFAULT_FLOOR_NAME: &str = "floor";
pub const DEFAULT_FLOOR_DIRECTORY: &str = "generation";

// Floor
pub const FLOOR_WIDTH: i32 = 9;
pub const FLOOR_HEIGHT: i32 = 8;
pub const MAX_ROOMS: i32 = 20;

pub const START_ROOM_POSITION: (i32, i32) = (4, 3);
pub const START_ROOM_RESEED_MIN_ROOMS: i32 = 16;
pub const MAX_START_ROOM_RESEEDS: i32 = 10;

pub const SECRET_ROOM_SEARCH: [(i32, bool); 2] = [
    (3, true),
    (2, true),
];

pub const SECRET_ROOM_ATTEMPTS: i32 = 300;
pub const REQUIRED_DEAD_ENDS: i32 = 3;
pub const MAX_GENERATION_ATTEMPTS: i32 = 200;

pub const BIG_BOSS_ROOM_CHANCE: f32 = 0.1;
pub const LONG_BOSS_ROOM_CHANCE: f32 = 0.2;
pub const BOSS_ROOM_CANDIDATES: i32 = 2;

pub const X_OFFSET: f32 = (FLOOR_WIDTH * ROOM_WIDTH) as f32 / 2.0;
pub const Y_OFFSET: f32 = 0.0;

pub const WINDOW_MULTIPLIER: f32 = 2.0;
pub const WINDOW_HEIGHT: i32 = (FLOOR_HEIGHT * ROOM_HEIGHT) * WINDOW_MULTIPLIER as i32;
pub const WINDOW_WIDTH: i32 = (FLOOR_WIDTH * ROOM_WIDTH) * WINDOW_MULTIPLIER as i32;

pub const LINE_THICKNESS: i32 = 5;