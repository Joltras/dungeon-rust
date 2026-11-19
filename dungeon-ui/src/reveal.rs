//! Step by step drawing of a new floor, spreading from the start room through the doors.

use std::collections::HashSet;

use dungeon_generator::Floor;

/// Time between two rings of rooms in seconds, like the 500 ms of the Python UI.
pub const STEP_SECONDS: f64 = 0.5;

/// Reveals the rooms of a floor ring by ring. After the last ring,
/// one more step passes before the teleport links are drawn.
#[derive(Debug, Clone)]
pub struct Reveal {
    rings: Vec<Vec<u32>>,
    started: f64,
}

impl Reveal {
    /// Starts revealing the floor at the given time in seconds.
    pub fn new(floor: &Floor, now: f64) -> Self {
        let rings = floor
            .rooms_by_distance()
            .into_iter()
            .map(|ring| ring.into_iter().map(|room| room.id()).collect())
            .collect();
        Self {
            rings,
            started: now,
        }
    }

    fn steps(&self, now: f64) -> usize {
        ((now - self.started).max(0.0) / STEP_SECONDS) as usize
    }

    /// Number of rings that are drawn at the time.
    pub fn shown_rings(&self, now: f64) -> usize {
        (self.steps(now) + 1).min(self.rings.len())
    }

    /// Ids of the rooms that are drawn at the time.
    pub fn visible_rooms(&self, now: f64) -> HashSet<u32> {
        self.rings[..self.shown_rings(now)]
            .iter()
            .flatten()
            .copied()
            .collect()
    }

    /// Returns true once all rooms and the teleport links are drawn.
    pub fn is_finished(&self, now: f64) -> bool {
        self.steps(now) >= self.rings.len()
    }

    /// Seconds until the next ring or the teleport links are drawn.
    pub fn time_to_next_step(&self, now: f64) -> f64 {
        let next = (self.steps(now) + 1) as f64 * STEP_SECONDS;
        (self.started + next - now).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dungeon_generator::Generator;

    fn floor() -> Floor {
        Generator::new("abc", 2).generate().clone()
    }

    #[test]
    fn reveals_one_ring_per_step() {
        let floor = floor();
        let rings = floor.rooms_by_distance();
        let reveal = Reveal::new(&floor, 10.0);

        assert_eq!(reveal.shown_rings(10.0), 1);
        let start = reveal.visible_rooms(10.0);
        assert_eq!(start, HashSet::from([rings[0][0].id()]));
        assert_eq!(reveal.shown_rings(10.49), 1);
        assert_eq!(reveal.shown_rings(10.5), 2);
        assert!((reveal.time_to_next_step(10.2) - 0.3).abs() < 1e-9);

        let last_ring = 10.0 + (rings.len() - 1) as f64 * STEP_SECONDS;
        assert_eq!(reveal.visible_rooms(last_ring).len(), floor.rooms().len());
        assert!(!reveal.is_finished(last_ring));
        assert!(reveal.is_finished(last_ring + STEP_SECONDS));
    }
}
