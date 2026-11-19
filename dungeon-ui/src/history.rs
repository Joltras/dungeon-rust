//! History of the shown floors, like the `FloorManager` of the Python UI.

use std::path::PathBuf;

use dungeon_generator::Floor;

/// A floor of the history with the file it belongs to.
pub struct Entry {
    pub floor: Floor,
    /// File the floor is saved to with "Save"
    pub file: PathBuf,
    /// True once the floor was opened from or saved to `file`, so saving
    /// it again does not ask before overwriting the file
    pub has_file: bool,
}

impl Entry {
    /// A generated floor that has not been saved yet.
    pub fn generated(floor: Floor, file: PathBuf) -> Self {
        Self {
            floor,
            file,
            has_file: false,
        }
    }

    /// A floor that was opened from the file.
    pub fn opened(floor: Floor, file: PathBuf) -> Self {
        Self {
            floor,
            file,
            has_file: true,
        }
    }
}

/// All floors shown so far and the position of the shown one.
pub struct History {
    entries: Vec<Entry>,
    index: usize,
}

impl History {
    pub fn new(first: Entry) -> Self {
        Self {
            entries: vec![first],
            index: 0,
        }
    }

    /// Adds a floor at the end and shows it.
    pub fn push(&mut self, entry: Entry) {
        self.entries.push(entry);
        self.index = self.entries.len() - 1;
    }

    pub fn current(&self) -> &Entry {
        &self.entries[self.index]
    }

    pub fn current_mut(&mut self) -> &mut Entry {
        &mut self.entries[self.index]
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
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
        if self.index + 1 >= self.entries.len() {
            return false;
        }
        self.index += 1;
        true
    }

    /// Position of the shown floor, e.g. "2 / 3".
    pub fn position_text(&self) -> String {
        format!("{} / {}", self.index + 1, self.entries.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(seed: &str) -> Entry {
        Entry::generated(Floor::new(8, 9, seed), PathBuf::from("floor.json"))
    }

    #[test]
    fn navigates_between_floors() {
        let mut history = History::new(entry("a"));
        assert!(!history.back());
        assert!(!history.forward());
        history.push(entry("b"));
        history.push(entry("c"));
        assert_eq!(history.position_text(), "3 / 3");
        assert!(history.back());
        assert!(history.back());
        assert_eq!(history.current().floor.seed(), "a");
        assert_eq!(history.position_text(), "1 / 3");
        assert!(history.forward());
        assert_eq!(history.current().floor.seed(), "b");
    }

    #[test]
    fn new_floors_are_added_at_the_end() {
        let mut history = History::new(entry("a"));
        history.push(entry("b"));
        history.back();
        history.push(entry("c"));
        assert_eq!(history.len(), 3);
        assert_eq!(history.current().floor.seed(), "c");
    }
}
