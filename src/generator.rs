//! Generation of floors, inspired by the floor generation of "The Binding of Isaac".
//!
//! The random numbers are drawn in exactly the same order as in the Python generator,
//! so the same seed and floor id produce the same floor in both versions.

use std::collections::{HashMap, VecDeque};
use std::io;
use std::path::Path;

use crate::floor::Floor;
use crate::utils::Position;
use crate::utils::constants::{
    BIG_BOSS_ROOM_CHANCE, BOSS_ROOM_CANDIDATES, FLOOR_HEIGHT, FLOOR_WIDTH, LONG_BOSS_ROOM_CHANCE,
    MAX_GENERATION_ATTEMPTS, MAX_START_ROOM_RESEEDS, REQUIRED_DEAD_ENDS, SECRET_ROOM_ATTEMPTS,
    SECRET_ROOM_SEARCH, START_ROOM_POSITION, START_ROOM_RESEED_MIN_ROOMS,
};
use crate::utils::direction::Direction;
use crate::utils::python_random::PyRandom;
use crate::utils::room_type::RoomType;
use crate::utils::util_functions::{
    add_direction_to_coordinates, calculate_distance, calculate_room_amount, is_next_to, place_room,
};

/// Rooms that were dead ends of the floor plan, the teleport room replaces a boss room.
const END_ROOM_TYPES: [RoomType; 5] = [
    RoomType::DeadEnd,
    RoomType::Item,
    RoomType::Shop,
    RoomType::Boss,
    RoomType::Teleport,
];

/// Combinations for a 2 × 2 boss room: two sides and the corner between them.
const BIG_BOSS_ROOM_DIRECTIONS: [[Direction; 3]; 4] = [
    [Direction::Right, Direction::Up, Direction::UpRight],
    [Direction::Right, Direction::Down, Direction::DownRight],
    [Direction::Left, Direction::Up, Direction::UpLeft],
    [Direction::Left, Direction::Down, Direction::DownLeft],
];

/// Generates floors for a seed and a floor id.
pub struct Generator {
    stage_id: u32,
    seed: String,
    floor: Floor,
    random: PyRandom,
}

impl Generator {
    /// Creates a generator. The floor is empty until [`Generator::generate`] is called.
    pub fn new(seed: impl Into<String>, stage_id: u32) -> Self {
        let seed = seed.into();
        Self {
            stage_id,
            random: Self::seeded_random(&seed, stage_id),
            floor: Floor::new(FLOOR_HEIGHT, FLOOR_WIDTH, seed.clone()),
            seed,
        }
    }

    /// Seeds the random numbers with the seed and the floor id,
    /// so the same seed produces different floors for different floor ids.
    fn seeded_random(seed: &str, stage_id: u32) -> PyRandom {
        PyRandom::from_seed(&format!("{seed}/{stage_id}"))
    }

    pub fn seed(&self) -> &str {
        &self.seed
    }

    pub fn stage_id(&self) -> u32 {
        self.stage_id
    }

    /// Sets the floor id for the next generation. Like in Python, the random numbers
    /// are not seeded again, use [`Generator::generate_with_seed`] for that.
    pub fn set_stage_id(&mut self, stage_id: u32) {
        self.stage_id = stage_id;
    }

    /// The last generated floor.
    pub fn floor(&self) -> &Floor {
        &self.floor
    }

    pub fn into_floor(self) -> Floor {
        self.floor
    }

    /// Generates a floor with a new seed.
    pub fn generate_with_seed(&mut self, seed: impl Into<String>) -> &Floor {
        self.seed = seed.into();
        self.random = Self::seeded_random(&self.seed, self.stage_id);
        self.generate()
    }

    /// Generates the rooms for the floor.
    /// Like in "The Binding of Isaac", a floor is generated again if it does not
    /// reach the room amount or has not enough dead ends for the special rooms.
    /// Dead ends directly next to the start room do not count,
    /// so the item and shop room are not placed right next to the start.
    pub fn generate(&mut self) -> &Floor {
        let mut dead_ends = Vec::new();
        for attempt in 0..MAX_GENERATION_ATTEMPTS {
            let last_attempt = attempt == MAX_GENERATION_ATTEMPTS - 1;
            let (start, complete, end_cells) = self.place_rooms();
            let marked = self.mark_dead_ends();
            dead_ends = self.order_like(marked, &end_cells);
            let rooms = self.floor.rooms();
            let usable = dead_ends
                .iter()
                .filter(|&&index| !is_next_to(rooms[index].position(), start))
                .count();
            let boss_next_to_start = dead_ends
                .last()
                .is_some_and(|&index| is_next_to(rooms[index].position(), start));
            if last_attempt
                || (complete && usable >= REQUIRED_DEAD_ENDS as usize && !boss_next_to_start)
            {
                break;
            }
        }
        self.add_boss_room(&mut dead_ends);
        self.add_special_rooms(&dead_ends);
        self.place_secret_room();
        self.place_super_secret_room();
        self.floor.add_doors_to_rooms();
        &self.floor
    }

    fn create_floor(&mut self) {
        self.floor = Floor::new(FLOOR_HEIGHT, FLOOR_WIDTH, self.seed.clone());
        self.floor.set_floor_id(Some(self.stage_id));
    }

    /// Adds the cell to the queue and the floor grid if a room can and should be placed.
    fn add_new_room(&mut self, cell: Position, queue: &mut VecDeque<Position>) -> bool {
        // The random number is only drawn if the cell is free, like in Python
        if self.floor.is_within_border(cell)
            && !self.floor.contains_room(cell)
            && self.floor.count_neighbours(cell) <= 1
            && place_room(&mut self.random)
        {
            queue.push_back(cell);
            self.floor.add_to_floor_grid(cell);
            return true;
        }
        false
    }

    /// Creates a new floor and places the start room and the normal rooms.
    /// Returns the start room position, if the room amount for the floor has been
    /// reached and the dead ends in the order they were found.
    fn place_rooms(&mut self) -> (Position, bool, Vec<Position>) {
        self.create_floor();
        let number_of_rooms = calculate_room_amount(self.stage_id, &mut self.random);

        // Like in "The Binding of Isaac" the start room is always cell 35
        let start = START_ROOM_POSITION;
        self.floor.add_room(start.0, start.1, RoomType::Start);
        let mut current_rooms = 1;
        let mut queue = VecDeque::from([start]);
        let mut cells: Vec<Position> = Vec::new();
        let mut end_cells: Vec<Position> = Vec::new();
        let mut reseeds = 0;
        while current_rooms < number_of_rooms {
            // A queue makes the floor grow evenly in all directions (breadth first)
            let Some(cell) = queue.pop_front() else {
                break;
            };
            let mut added_rooms = 0;
            for direction in Direction::MAIN_DIRECTIONS {
                if current_rooms >= number_of_rooms {
                    break;
                }
                if self.add_new_room(add_direction_to_coordinates(direction, cell), &mut queue) {
                    current_rooms += 1;
                    added_rooms += 1;
                }
            }
            if cell != start {
                cells.push(cell);
                if added_rooms == 0 {
                    // A room that does not add a neighbour is a dead end
                    end_cells.push(cell);
                }
            }
            if queue.is_empty()
                && current_rooms < number_of_rooms
                && number_of_rooms > START_ROOM_RESEED_MIN_ROOMS
                && reseeds < MAX_START_ROOM_RESEEDS
            {
                // Like in "The Binding of Isaac", big floors grow again from the start
                queue.push_back(start);
                reseeds += 1;
            }
        }

        // Rooms that are still queued did not add neighbours either
        let remaining: Vec<Position> = queue.into_iter().filter(|&cell| cell != start).collect();
        cells.extend(&remaining);
        end_cells.extend(remaining);
        while let Some((x, y)) = cells.pop() {
            self.floor.add_room(x, y, RoomType::Normal);
        }
        (start, current_rooms >= number_of_rooms, end_cells)
    }

    /// Sorts the indices of the dead ends by their number of steps from the start
    /// room and then by the order their cells were found, like the end room list
    /// of "The Binding of Isaac". Sorting by steps keeps the farthest dead end last
    /// even when big floors grew again from the start.
    fn order_like(&self, mut dead_ends: Vec<usize>, end_cells: &[Position]) -> Vec<usize> {
        let rooms = self.floor.rooms();
        let order: HashMap<Position, i64> = end_cells
            .iter()
            .enumerate()
            .map(|(position, cell)| (*cell, position as i64))
            .collect();
        let steps = self.steps_from(START_ROOM_POSITION);
        // The sort is stable like the one in Python
        dead_ends.sort_by_key(|&index| {
            let cell = rooms[index].position();
            (
                steps.get(&cell).copied().unwrap_or(0),
                order.get(&cell).copied().unwrap_or(-1),
            )
        });
        dead_ends
    }

    /// Counts the steps from the start to every reachable cell of the floor grid.
    fn steps_from(&self, start: Position) -> HashMap<Position, u32> {
        let mut steps = HashMap::from([(start, 0)]);
        let mut queue = VecDeque::from([start]);
        while let Some(cell) = queue.pop_front() {
            let next_steps = steps[&cell] + 1;
            for direction in Direction::MAIN_DIRECTIONS {
                let neighbour = add_direction_to_coordinates(direction, cell);
                if !steps.contains_key(&neighbour) && self.floor.contains_room(neighbour) {
                    steps.insert(neighbour, next_steps);
                    queue.push_back(neighbour);
                }
            }
        }
        steps
    }

    /// Marks all normal rooms with exactly one neighbour as dead ends
    /// and returns their indices.
    fn mark_dead_ends(&mut self) -> Vec<usize> {
        let mut dead_ends = Vec::new();
        for index in 0..self.floor.rooms().len() {
            let room = &self.floor.rooms()[index];
            if self.floor.is_dead_end(room.position()) && room.room_type() == RoomType::Normal {
                self.floor.room_mut(index).set_type(RoomType::DeadEnd);
                dead_ends.push(index);
            }
        }
        dead_ends
    }

    /// Turns the last found dead end into the boss room. Because the floor grows
    /// breadth first, it is one of the rooms farthest away from the start room.
    /// If it has no free space next to it, the dead end found before it is tried,
    /// otherwise the boss room is moved away and connected with a teleport room.
    fn add_boss_room(&mut self, dead_ends: &mut Vec<usize>) {
        let Some(&farthest) = dead_ends.last() else {
            return;
        };
        let boss_index = dead_ends
            .iter()
            .rev()
            .take(BOSS_ROOM_CANDIDATES as usize)
            .copied()
            .find(|&index| !self.free_boss_locations(index).is_empty())
            .unwrap_or(farthest);
        let locations = self.free_boss_locations(boss_index);

        if locations.is_empty() {
            self.place_boss_with_teleport_room(boss_index);
        } else {
            let size_roll = self.random.random();
            let placed = locations.len() >= 2
                && size_roll < BIG_BOSS_ROOM_CHANCE
                && self.place_big_boss_room(&locations, boss_index);
            if !placed && size_roll < BIG_BOSS_ROOM_CHANCE + LONG_BOSS_ROOM_CHANCE {
                // Create a 1 × 2 boss room
                self.floor
                    .add_room_next_to(boss_index, locations[0], RoomType::Boss);
            }
        }

        self.floor.room_mut(boss_index).set_type(RoomType::Boss);
        if let Some(position) = dead_ends.iter().position(|&index| index == boss_index) {
            dead_ends.remove(position);
        }
    }

    /// Directions with a free cell next to the room that is only next to this room.
    fn free_boss_locations(&self, index: usize) -> Vec<Direction> {
        let position = self.floor.rooms()[index].position();
        Direction::MAIN_DIRECTIONS
            .into_iter()
            .filter(|&direction| {
                let cell = add_direction_to_coordinates(direction, position);
                self.floor.is_within_border(cell)
                    && !self.floor.contains_room(cell)
                    && self.floor.count_neighbours(cell) == 1
            })
            .collect()
    }

    /// Extends the boss room to 2 × 2 if two sides and the corner between them are free.
    fn place_big_boss_room(&mut self, locations: &[Direction], boss_index: usize) -> bool {
        let position = self.floor.rooms()[boss_index].position();
        for directions in BIG_BOSS_ROOM_DIRECTIONS {
            let corner = add_direction_to_coordinates(directions[2], position);
            if locations.contains(&directions[0])
                && locations.contains(&directions[1])
                && self.floor.has_no_neighbours(corner)
            {
                for direction in directions {
                    self.floor
                        .add_room_next_to(boss_index, direction, RoomType::Boss);
                }
                return true;
            }
        }
        false
    }

    /// Moves the boss room to a free cell without neighbours and places a teleport
    /// room at its old position. If there is no such cell, the boss room stays.
    fn place_boss_with_teleport_room(&mut self, boss_index: usize) {
        let Some(destination) = self.find_teleport_destination() else {
            return;
        };
        self.floor.add_teleport_room(boss_index);
        self.floor.room_mut(boss_index).set_position(destination);
        self.floor.add_to_floor_grid(destination);
    }

    /// Searches for a free cell without neighbours, preferring the corners.
    fn find_teleport_destination(&self) -> Option<Position> {
        let floor = &self.floor;
        let corners = [
            floor.top_left(),
            floor.top_right(),
            floor.bottom_left(),
            floor.bottom_right(),
        ];
        let all_cells = (0..FLOOR_HEIGHT).flat_map(|y| (0..FLOOR_WIDTH).map(move |x| (x, y)));
        corners
            .into_iter()
            .chain(all_cells)
            .find(|&cell| !floor.contains_room(cell) && floor.has_no_neighbours(cell))
    }

    /// Places the special rooms on randomly chosen dead ends of the floor.
    /// Dead ends directly next to the start room are only used
    /// if there are not enough other dead ends.
    fn add_special_rooms(&mut self, dead_ends: &[usize]) {
        let rooms = self.floor.rooms();
        let start = rooms
            .iter()
            .find(|room| room.room_type() == RoomType::Start)
            .map(|room| room.position());
        let next_to_start =
            |index: usize| start.is_some_and(|start| is_next_to(rooms[index].position(), start));
        let (mut preferred, mut fallback): (Vec<usize>, Vec<usize>) =
            dead_ends.iter().partition(|&&index| !next_to_start(index));
        self.random.shuffle(&mut preferred);
        self.random.shuffle(&mut fallback);
        for (room_type, index) in RoomType::SPECIAL_ROOMS
            .into_iter()
            .zip(preferred.into_iter().chain(fallback))
        {
            self.floor.room_mut(index).set_type(room_type);
        }
    }

    /// Places the super secret room as close as possible to the boss room, but not
    /// next to it and next to exactly one room.
    fn place_super_secret_room(&mut self) {
        // Python fails here without a boss room, which no generated floor reaches
        let Some(boss) = self.floor.get_boss_room().map(|room| room.position()) else {
            return;
        };
        let mut candidate: Option<(i32, Position)> = None;
        for room in self.floor.rooms() {
            for direction in Direction::MAIN_DIRECTIONS {
                let neighbour = add_direction_to_coordinates(direction, room.position());
                if self.floor.is_within_border(neighbour)
                    && !self.floor.contains_room(neighbour)
                    && !self.floor.has_boss_room_as_neighbour(neighbour)
                    && self.floor.count_neighbours(neighbour) == 1
                {
                    let distance = calculate_distance(boss, neighbour);
                    if candidate.is_none_or(|(best, _)| distance < best) {
                        candidate = Some((distance, neighbour));
                    }
                }
            }
        }
        if let Some((_, (x, y))) = candidate {
            self.floor.add_room(x, y, RoomType::SuperSecret);
        }
    }

    /// Places the secret room like "The Binding of Isaac": it randomly searches
    /// for an empty cell next to at least three rooms and not next to an end room.
    /// After some attempts the criteria are loosened, so a secret room is always
    /// placed if there is any empty cell next to a room that is not the boss room.
    fn place_secret_room(&mut self) {
        for (min_neighbours, avoid_end_rooms) in SECRET_ROOM_SEARCH {
            for _ in 0..SECRET_ROOM_ATTEMPTS {
                let cell = (
                    self.random.randint(0, FLOOR_WIDTH - 1),
                    self.random.randint(0, FLOOR_HEIGHT - 1),
                );
                if self.fits_secret_room(cell, min_neighbours as usize, avoid_end_rooms) {
                    self.floor.add_room(cell.0, cell.1, RoomType::Secret);
                    return;
                }
            }
        }
        let candidates: Vec<Position> = (0..FLOOR_HEIGHT)
            .flat_map(|y| (0..FLOOR_WIDTH).map(move |x| (x, y)))
            .filter(|&cell| self.fits_secret_room(cell, 1, false))
            .collect();
        if let Some(&(x, y)) = self.random.choice(&candidates) {
            self.floor.add_room(x, y, RoomType::Secret);
        }
    }

    /// Checks if the secret room can be placed in the cell.
    fn fits_secret_room(
        &self,
        cell: Position,
        min_neighbours: usize,
        avoid_end_rooms: bool,
    ) -> bool {
        let floor = &self.floor;
        if floor.contains_room(cell) || floor.has_boss_room_as_neighbour(cell) {
            return false;
        }
        if floor.count_neighbours(cell) < min_neighbours {
            return false;
        }
        !avoid_end_rooms
            || Direction::MAIN_DIRECTIONS.into_iter().all(|direction| {
                floor
                    .get_room(add_direction_to_coordinates(direction, cell))
                    .is_none_or(|room| !END_ROOM_TYPES.contains(&room.room_type()))
            })
    }

    /// Creates the json file content of the generated floor.
    pub fn to_json(&self) -> String {
        self.floor.to_json()
    }

    /// Writes the generated floor to the file.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        std::fs::write(path, self.to_json())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate(seed: &str, stage_id: u32) -> Floor {
        let mut generator = Generator::new(seed, stage_id);
        generator.generate();
        generator.into_floor()
    }

    fn count(floor: &Floor, room_type: RoomType) -> usize {
        floor
            .rooms()
            .iter()
            .filter(|room| room.room_type() == room_type)
            .count()
    }

    #[test]
    fn floor_has_all_room_types() {
        for stage_id in 0..6 {
            let floor = generate("abc", stage_id);
            assert_eq!(count(&floor, RoomType::Start), 1);
            assert!(count(&floor, RoomType::Boss) >= 1);
            assert_eq!(count(&floor, RoomType::Item), 1);
            assert_eq!(count(&floor, RoomType::Shop), 1);
            assert_eq!(count(&floor, RoomType::Secret), 1);
            assert_eq!(floor.floor_id(), Some(stage_id));
        }
    }

    #[test]
    fn same_seed_produces_same_floor() {
        assert_eq!(generate("x", 2).rooms(), generate("x", 2).rooms());
        assert_ne!(generate("x", 2).rooms(), generate("x", 3).rooms());
    }

    #[test]
    fn every_room_is_reached_from_the_start() {
        for seed in 0..20 {
            let floor = generate(&seed.to_string(), 3);
            let reached: usize = floor.rooms_by_distance().iter().map(Vec::len).sum();
            assert_eq!(reached, floor.rooms().len());
        }
    }
}
