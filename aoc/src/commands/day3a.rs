use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

use crate::utils::{slurp_file, ParseError, SlurpError};

use super::{CommandImpl, DynError};

const ACII_TO_PRIORITY: &[u8] = &[
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25, 26,
];

#[derive(Parser, Debug)]
pub struct Day3a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day3a {
    fn main(&self) -> Result<(), DynError> {
        let total = naive(&self.input)?;
        println!("Total: {}", total);
        Ok(())
    }
}

fn naive<P: AsRef<Path>>(path: P) -> Result<usize, SlurpError> {
    Ok(slurp_file::<P, Score>(path)?.into_iter().map(|s| s.score).sum::<usize>())
}

/// Convert an ascii value to a priority value.
///
/// A-Z are priority 27-52
/// a-z are priority 1-26
///
/// This is done by subtracting the ascii value for `A` from the input byte and then looking up the
/// corresponding value in the lookup table. We assume all inputs are valid.
#[inline]
fn ascii_to_priority(byte: u8) -> u8 {
    ACII_TO_PRIORITY[(byte - 65) as usize]
}

/// We're computing the score as we parse the input, which is a little meh, but it's the weekend and
/// it's fun to indulge in premature-optimization sometimes
struct Score {
    score: usize,
}

impl FromStr for Score {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size = s.len() / 2;
        // For real speed we should swap to a different hashing algo
        // Or use a constant sized array for set ones values.
        // Create first side's hashset
        let p1: HashSet<&u8> = HashSet::from_iter(&s.as_bytes()[0..size]);
        let mut score = 0;
        for byte in &s.as_bytes()[size..] {
            if p1.contains(byte) {
                score = ascii_to_priority(*byte) as usize;
                break;
            }
        }

        Ok(Score { score })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example() {
        let lines = vec![
            "vJrwpWtwJgWrhcsFMMfFFhFp",
            "jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL",
            "PmmdzqPrVvPwwTWBwg",
            "wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn",
            "ttgJtRGJQctTZtZT",
            "CrZsJsPPZsGzwwsLwLmpwMDw",
        ];

        let scores = lines
            .into_iter()
            .map(|line| line.parse::<Score>())
            .collect::<Result<Vec<Score>, ParseError>>()
            .unwrap();
        let total = scores.into_iter().map(|s| s.score).sum::<usize>();
        assert_eq!(total, 157);
    }
}
