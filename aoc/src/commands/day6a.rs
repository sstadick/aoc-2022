use std::path::PathBuf;

use clap::Parser;

use crate::utils::slurp_bytes;

use super::{CommandImpl, DynError};
use lowercase_alpha::{LowercaseAlphaBitSet, LowercaseAlphaByte};

pub mod lowercase_alpha {
    //! Module for working with lowercase alphabet letters.

    /// Helper type to ensure inputs to [`LowercaseAlphaBitSet`] are correct.
    ///
    /// This stores the byte as 0-based. i.e. `byte - b'a'`.
    #[derive(Debug, Clone, Copy)]
    pub struct LowercaseAlphaByte(u8);

    impl LowercaseAlphaByte {
        /// Validate that the byte is within the correct ascii range
        pub fn new(byte: u8) -> Self {
            assert!((b'a'..=b'z').contains(&byte));
            Self(byte - b'a')
        }
    }

    /// A bitset for storing values for the lowercase alphabet values in ascii range. (i.e. a-z)
    #[derive(Debug, Clone, Copy)]
    pub struct LowercaseAlphaBitSet {
        bits: u32,
    }

    impl LowercaseAlphaBitSet {
        /// Create a new collection
        pub fn new() -> Self {
            Self { bits: 0 }
        }

        /// Add an item to this collection
        pub fn add(&mut self, alpha: LowercaseAlphaByte) {
            self.bits |= 1 << alpha.0;
        }

        /// Check to see if this collection contains an item
        pub fn contains(&self, alpha: LowercaseAlphaByte) -> bool {
            (self.bits >> alpha.0) & 1 == 1
        }

        /// Remove an item from this collection
        pub fn remove(&mut self, alpha: LowercaseAlphaByte) {
            self.bits &= !(1 << alpha.0);
        }

        /// Get number of items in this collection
        pub fn len(&self) -> usize {
            self.bits.count_ones() as usize
        }
    }
}

#[derive(Parser, Debug)]
pub struct Day6a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day6a {
    fn main(&self) -> Result<(), DynError> {
        let bytes = slurp_bytes(&self.input)?;
        let start = Message::detect_packet_start(&bytes);
        println!("Start: {}", start);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Message;

impl Message {
    /// Find the offset into the buffer that indicates how many bytes had to be processed to find the start of packet marker
    /// is found.
    ///
    /// Packet start is indicated by a series of 4 values that are all different.
    pub(crate) fn detect_packet_start(buffer: &[u8]) -> usize {
        if let Some((i, window)) = buffer.windows(4).enumerate().find(|(i, window)| {
            // Create a bitset.
            let mut collection = LowercaseAlphaBitSet::new();
            // Add values to the bitset.
            window.iter().for_each(|v| collection.add(LowercaseAlphaByte::new(*v)));
            // If the total number of items isn't 4, whe know there was a dup.
            collection.len() == 4
        }) {
            println!("Window: {}", String::from_utf8_lossy(window));
            i + 4
        } else {
            panic!("No start location found in input buffer");
        }
    }

    /// Find the offset into the buffer that indicates how many bytes had to be processed to find the start of message marker
    /// is found.
    ///
    /// Message start is indicated by a series of 14 values that are all different.
    pub(crate) fn detect_message_start(buffer: &[u8]) -> usize {
        if let Some((i, window)) = buffer.windows(14).enumerate().find(|(i, window)| {
            // Create a bitset.
            let mut collection = LowercaseAlphaBitSet::new();
            // Add values to the bitset.
            window.iter().for_each(|v| collection.add(LowercaseAlphaByte::new(*v)));
            // If the total number of items isn't 4, whe know there was a dup.
            collection.len() == 14
        }) {
            println!("Window: {}", String::from_utf8_lossy(window));
            i + 14
        } else {
            panic!("No start location found in input buffer");
        }
    }
}
