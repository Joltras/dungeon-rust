//! History of the generated floors, like the `FloorManager` of the Python UI.

use dungeon_generator::Floor;

/// All floors shown so far and the position of the shown one.
pub struct History {
    floors: Vec<Floor>,
    index: usize,
}

impl History {
    pub fn new(first: Floor) -> Self {
        Self {
            floors: vec![first],
            index: 0,
        }
    }

    /// Adds a floor at the end and shows it.
    pub fn push(&mut self, floor: Floor) {
        self.floors.push(floor);
        self.index = self.floors.len() - 1;
    }

    pub fn current(&self) -> &Floor {
        &self.floors[self.index]
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.floors.len()
    }

    /// Shows the previous floor. Returns false if the first floor is shown.
    pub fn back(&mut self) -> bool {
        if self.index == 0 {
            return false;
        }
        self.index -= 1;
        true
    }

    /// Shows the next floor. Returns false if the last floor is shown.
    pub fn forward(&mut self) -> bool {
        if self.index + 1 >= self.floors.len() {
            return false;
        }
        self.index += 1;
        true
    }

    /// Position of the shown floor, e.g. "2 / 3".
    pub fn position_text(&self) -> String {
        format!("{} / {}", self.index + 1, self.floors.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floor(seed: &str) -> Floor {
        Floor::new(8, 9, seed)
    }

    #[test]
    fn navigates_between_floors() {
        let mut history = History::new(floor("a"));
        assert!(!history.back());
        assert!(!history.forward());
        history.push(floor("b"));
        history.push(floor("c"));
        assert_eq!(history.position_text(), "3 / 3");
        assert!(history.back());
        assert!(history.back());
        assert_eq!(history.current().seed(), "a");
        assert_eq!(history.position_text(), "1 / 3");
        assert!(history.forward());
        assert_eq!(history.current().seed(), "b");
    }

    #[test]
    fn new_floors_are_added_at_the_end() {
        let mut history = History::new(floor("a"));
        history.push(floor("b"));
        history.back();
        history.push(floor("c"));
        assert_eq!(history.len(), 3);
        assert_eq!(history.current().seed(), "c");
    }
}
