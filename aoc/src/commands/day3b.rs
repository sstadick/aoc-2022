use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;

use super::{CommandImpl, DynError};

const ACII_TO_PRIORITY: &[u8] = &[
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26,
];

#[derive(Parser, Debug)]
pub struct Day3b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day3b {
    fn main(&self) -> Result<(), DynError> {
        // We're gonna get crazy with this one
        let mut reader = BufReader::new(File::open(&self.input)?);

        let mut buffer1 = String::new();
        let mut buffer2 = String::new();
        let mut buffer3 = String::new();
        let mut score = 0;

        loop {
            let bytes_read = reader.read_line(&mut buffer1)?;
            if bytes_read == 0 {
                // EOF
                break;
            }
            // We are assuming good inputs an that our groups always come in threes
            _ = reader.read_line(&mut buffer2)?;
            _ = reader.read_line(&mut buffer3)?;

            score += find_badge_raw(
                // This is gross, but read_line will include the newlines
                &buffer1.as_bytes()[0..buffer1.as_bytes().len() - 1],
                &buffer2.as_bytes()[0..buffer2.as_bytes().len() - 1],
                &buffer3.as_bytes()[0..buffer3.as_bytes().len() - 1],
            ) as usize;

            // Clear buffers before appending again.
            buffer1.clear();
            buffer2.clear();
            buffer3.clear();
        }

        println!("Total Score: {}", score);

        Ok(())
    }
}

/// Find the shared item between the three elves raw bytes
// Extracted to make this testable
fn find_badge_raw(elf1: &[u8], elf2: &[u8], elf3: &[u8]) -> u8 {
    let elf1_ruck = Ruck::from_slice(elf1);
    let elf2_ruck = Ruck::from_slice(elf2);
    let elf3_ruck = Ruck::from_slice(elf3);
    elf1_ruck.find_badge(&elf2_ruck, &elf3_ruck)
}

mod item {
    //! Putting [`Item`] in a module so that it can only be created using blessed methods.

    use super::ACII_TO_PRIORITY;

    /// Helper newtype to enforce the conversion from an ascii value to an Item (priority).
    #[derive(Copy, Clone)]
    #[repr(transparent)]
    pub struct Item(u8);

    impl Item {
        /// Get the priority value of this [`Item`].
        #[inline]
        pub fn priority(&self) -> u8 {
            self.0
        }
    }

    impl From<u8> for Item {
        /// Convert an ascii value to a priority value.
        ///
        /// A-Z are priority 27-52
        /// a-z are priority 1-26
        ///
        /// This is done by subtracting the ascii value for `A` from the input byte and then looking up the
        /// corresponding value in the lookup table. We assume all inputs are valid.
        fn from(byte: u8) -> Self {
            Self(ACII_TO_PRIORITY[(byte - 65) as usize])
        }
    }
}

use item::Item;

/// Our ruck is a bitset, since there are only 52 possible values for priorities
struct Ruck(u64);
impl Ruck {
    /// Create a new [`Ruck`].
    fn new() -> Self {
        Self(0)
    }

    /// Convert raw bytes to a ruck.
    ///
    /// This preforms the conversion from ascii to our item codes (aka priorities)
    fn from_slice(ascii_items: &[u8]) -> Self {
        let mut me = Self::new();
        for byte in ascii_items {
            me.add_item((*byte).into());
        }
        me
    }

    /// Add an item to the rucksack.
    ///
    /// **Note**: Items are assumed to have been converted to a priority.
    fn add_item(&mut self, item: Item) {
        // Set the bit for the new item
        self.0 |= 1 << item.priority();
    }

    /// Remove an item from the ruck
    fn remove_item(&mut self, item: Item) {
        // Unset the bit for the item
        self.0 &= !(1 << item.priority());
    }

    /// Check if the ruck contains an item
    fn contains(&self, item: Item) -> bool {
        (self.0 >> item.priority()) & 1 == 1
    }

    /// Find the item shared across three elves
    fn find_badge(&self, elf2: &Self, elf3: &Self) -> u8 {
        // Find the bit that is set in all three rucks
        match self.0 & elf2.0 & elf3.0 {
            0 => 0,
            val => (val.trailing_zeros()) as u8,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example() {
        let group1 = vec![
            "vJrwpWtwJgWrhcsFMMfFFhFp",
            "jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL",
            "PmmdzqPrVvPwwTWBwg",
        ];
        let group2 =
            vec!["wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn", "ttgJtRGJQctTZtZT", "CrZsJsPPZsGzwwsLwLmpwMDw"];

        let g1_score =
            find_badge_raw(group1[0].as_bytes(), group1[1].as_bytes(), group1[2].as_bytes())
                as usize;
        let g2_score =
            find_badge_raw(group2[0].as_bytes(), group2[1].as_bytes(), group2[2].as_bytes())
                as usize;

        assert_eq!(dbg!(dbg!(g1_score) + dbg!(g2_score)), 70);
    }
}
