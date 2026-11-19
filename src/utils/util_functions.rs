use super::Position;
use super::constants::MAX_ROOMS;
use super::direction::Direction;
use super::python_random::PyRandom;

/// Calculates the amount of rooms the generator places,
/// without the boss extension, secret and teleport rooms.
pub fn calculate_room_amount(stage_id: u32, random: &mut PyRandom) -> i32 {
    // Formula of "The Binding of Isaac": random(2) + 5 + level * 2.6,
    // where the first floor is level 1. Integers avoid rounding errors.
    let level = i64::from(stage_id) + 1;
    let amount = i64::from(random.randint(0, 1)) + 5 + level * 26 / 10;
    amount.min(i64::from(MAX_ROOMS)) as i32
}

/// Checks if a new room should be placed.
pub fn place_room(random: &mut PyRandom) -> bool {
    random.randint(1, 50) >= 25
}

/// Returns the position next to the given position in the direction.
pub fn add_direction_to_coordinates(direction: Direction, position: Position) -> Position {
    let (delta_x, delta_y) = direction.value();
    (
        position.0 + i32::from(delta_x),
        position.1 + i32::from(delta_y),
    )
}

/// Squared distance between two positions.
pub fn calculate_distance(first: Position, second: Position) -> i32 {
    let delta_x = first.0 - second.0;
    let delta_y = first.1 - second.1;
    delta_x * delta_x + delta_y * delta_y
}

/// Returns if the two positions are directly next to each other.
pub fn is_next_to(first: Position, second: Position) -> bool {
    (first.0 - second.0).abs() + (first.1 - second.1).abs() == 1
}
