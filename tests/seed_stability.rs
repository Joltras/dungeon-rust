//! Makes sure that a seed and floor id produce the same floor as the Python generator.
//!
//! The expected floors are stored as hashes in resources/floor_hashes.json,
//! copied from the seed stability test of the Python project.

use std::collections::BTreeMap;

use dungeon_generator::Generator;
use sha2::{Digest, Sha256};

const HASHES: &str = include_str!("resources/floor_hashes.json");

/// Hashes the floor like the Python test: sorted keys, no spaces, no ascii escapes.
fn floor_hash(seed: &str, floor_id: u32) -> String {
    let mut generator = Generator::new(seed, floor_id);
    generator.generate();
    // serde_json sorts the keys of a Value and writes non-ascii characters as they are
    let canonical = generator.floor().to_json_value()["_floor"].to_string();
    Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn seeds_produce_the_same_floors_as_python() {
    let expected: BTreeMap<String, String> = serde_json::from_str(HASHES).unwrap();
    assert!(!expected.is_empty());
    let changed: Vec<&String> = expected
        .iter()
        .filter(|(key, hash)| {
            let (floor_id, seed) = key.split_once('/').unwrap();
            floor_hash(seed, floor_id.parse().unwrap()) != **hash
        })
        .map(|(key, _)| key)
        .collect();
    assert!(
        changed.is_empty(),
        "{} of {} floors differ from Python (floor id/seed): {changed:?}",
        changed.len(),
        expected.len()
    );
}
