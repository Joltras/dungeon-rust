//! Reimplementation of the parts of Python's `random.Random` the generator uses.
//!
//! The Python generator seeds `random.Random` with a string, so a seed has to produce
//! exactly the same random numbers here to produce the same floor in both versions.
//! Python uses the Mersenne Twister MT19937 and seeds it with the string bytes followed
//! by their SHA-512 hash, read as one big-endian integer.

use sha2::{Digest, Sha512};

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// Random number generator that produces the same numbers as Python's `random.Random`.
#[derive(Clone)]
pub struct PyRandom {
    state: [u32; N],
    index: usize,
}

impl PyRandom {
    /// Creates a generator like `random.Random().seed(seed)` for a `str` seed in Python.
    pub fn from_seed(seed: &str) -> Self {
        let bytes = seed.as_bytes();
        let mut number = bytes.to_vec();
        number.extend_from_slice(&Sha512::digest(bytes));
        // Python splits the absolute value of the integer into 32-bit words,
        // starting with the least significant one
        number.reverse();
        let mut key: Vec<u32> = number
            .chunks(4)
            .map(|chunk| {
                let mut word = [0u8; 4];
                word[..chunk.len()].copy_from_slice(chunk);
                u32::from_le_bytes(word)
            })
            .collect();
        // Leading zero bytes of the integer are not part of the key
        while key.len() > 1 && key.last() == Some(&0) {
            key.pop();
        }
        let mut random = Self {
            state: [0; N],
            index: N,
        };
        random.init_by_array(&key);
        random
    }

    fn init_genrand(&mut self, seed: u32) {
        self.state[0] = seed;
        for i in 1..N {
            let previous = self.state[i - 1];
            self.state[i] = 1_812_433_253u32
                .wrapping_mul(previous ^ (previous >> 30))
                .wrapping_add(i as u32);
        }
        self.index = N;
    }

    fn init_by_array(&mut self, key: &[u32]) {
        self.init_genrand(19_650_218);
        let mut i = 1;
        let mut j = 0;
        for _ in 0..N.max(key.len()) {
            let previous = self.state[i - 1];
            self.state[i] = (self.state[i] ^ (previous ^ (previous >> 30)).wrapping_mul(1_664_525))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                self.state[0] = self.state[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            let previous = self.state[i - 1];
            self.state[i] = (self.state[i]
                ^ (previous ^ (previous >> 30)).wrapping_mul(1_566_083_941))
            .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                self.state[0] = self.state[N - 1];
                i = 1;
            }
        }
        self.state[0] = 0x8000_0000;
    }

    fn twist(&mut self) {
        for i in 0..N {
            let y = (self.state[i] & UPPER_MASK) | (self.state[(i + 1) % N] & LOWER_MASK);
            let mut value = self.state[(i + M) % N] ^ (y >> 1);
            if y & 1 != 0 {
                value ^= MATRIX_A;
            }
            self.state[i] = value;
        }
        self.index = 0;
    }

    /// Returns the next 32 random bits.
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }

    /// Like `getrandbits(bits)` in Python, limited to 32 bits.
    pub fn getrandbits(&mut self, bits: u32) -> u32 {
        assert!(bits <= 32, "only up to 32 bits are supported");
        if bits == 0 {
            return 0;
        }
        self.next_u32() >> (32 - bits)
    }

    /// Like `_randbelow(n)` in Python: a random number in `0..n`.
    fn randbelow(&mut self, n: u32) -> u32 {
        assert!(n > 0, "the range must not be empty");
        let bits = u32::BITS - n.leading_zeros();
        loop {
            let value = self.getrandbits(bits);
            if value < n {
                return value;
            }
        }
    }

    /// Like `randint(low, high)` in Python: a random number in `low..=high`.
    pub fn randint(&mut self, low: i32, high: i32) -> i32 {
        assert!(low <= high, "empty range for randint({low}, {high})");
        let width = (high - low) as u32 + 1;
        low + self.randbelow(width) as i32
    }

    /// Like `random()` in Python: a random float in `[0, 1)` with 53 random bits.
    pub fn random(&mut self) -> f64 {
        let high = (self.next_u32() >> 5) as f64;
        let low = (self.next_u32() >> 6) as f64;
        (high * 67_108_864.0 + low) * (1.0 / 9_007_199_254_740_992.0)
    }

    /// Like `shuffle(items)` in Python.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.randbelow(i as u32 + 1) as usize;
            items.swap(i, j);
        }
    }

    /// Like `choice(items)` in Python, but returns `None` for an empty slice.
    pub fn choice<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        Some(&items[self.randbelow(items.len() as u32) as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values were produced with CPython 3.14

    #[test]
    fn produces_the_same_numbers_as_python() {
        let mut random = PyRandom::from_seed("test/2");
        let bits: Vec<u32> = (0..3).map(|_| random.getrandbits(32)).collect();
        assert_eq!(bits, [1_491_647_969, 692_086_973, 2_448_194_528]);
        let ints: Vec<i32> = (0..5).map(|_| random.randint(1, 50)).collect();
        assert_eq!(ints, [1, 49, 8, 49, 10]);
        let ints: Vec<i32> = (0..3).map(|_| random.randint(0, 8)).collect();
        assert_eq!(ints, [1, 3, 4]);
        assert_eq!(random.random(), 0.6290945355484725);
        let mut items: Vec<u32> = (0..10).collect();
        random.shuffle(&mut items);
        assert_eq!(items, [9, 1, 7, 2, 3, 8, 6, 0, 5, 4]);
        assert_eq!(random.choice(&["a", "b", "c", "d", "e"]), Some(&"c"));
    }

    #[test]
    fn empty_seed_matches_python() {
        assert_eq!(PyRandom::from_seed("").getrandbits(32), 4_124_137_760);
    }

    #[test]
    fn choice_of_empty_slice_is_none() {
        let empty: [u8; 0] = [];
        assert_eq!(PyRandom::from_seed("x").choice(&empty), None);
    }
}
