use std::{ops::RangeInclusive, path::PathBuf, str::FromStr};

use clap::Parser;

use crate::utils::{slurp_file, ParseError};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day4a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day4a {
    fn main(&self) -> Result<(), DynError> {
        let pairs: Vec<ElfPair> = slurp_file(&self.input)?;
        let to_check = pairs.into_iter().filter(|p| p.assignment_containment()).count();
        println!("Total: {}", to_check);
        Ok(())
    }
}

/// An elf and the zones that it covers
#[derive(Debug, Clone)]
pub struct Elf {
    zones: RangeInclusive<usize>,
}

impl Elf {
    /// Create a new elf with a given set of zones
    fn new(zones: RangeInclusive<usize>) -> Self {
        Self { zones }
    }

    /// Check if this elf wholey contains the zones from another elf
    fn fully_contains(&self, other: &Self) -> bool {
        self.zones.start() <= other.zones.start() && self.zones.end() >= other.zones.end()
    }

    /// Check for any overlap between two elves zones.
    fn overlap(&self, other: &Self) -> bool {
        self.zones.start() <= other.zones.end() && self.zones.end() >= other.zones.start()
    }
}

impl FromStr for Elf {
    type Err = ParseError;

    /// Parse an inclusive range from the format `\d--d`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, stop) = s
            .split_once('-')
            .ok_or_else(|| ParseError::new(format!("Unable to parse range: `{}`", s)))?;
        Ok(Self::new(RangeInclusive::new(
            start.parse::<usize>().map_err(|e| {
                ParseError::new(format!("Unable to parse start {} to usize", start))
            })?,
            stop.parse::<usize>()
                .map_err(|e| ParseError::new(format!("Unable to parse stop {} to usize", stop)))?,
        )))
    }
}

pub struct ElfPair(Elf, Elf);

impl FromStr for ElfPair {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (elf1, elf2) = s
            .split_once(',')
            .ok_or_else(|| ParseError::new(format!("Unable to parse elf pair: `{}`", s)))?;
        let elf1 = elf1.parse::<Elf>()?;
        let elf2 = elf2.parse::<Elf>()?;
        Ok(ElfPair::new(elf1, elf2))
    }
}

impl ElfPair {
    /// Create a new elf pair
    fn new(elf1: Elf, elf2: Elf) -> Self {
        Self(elf1, elf2)
    }

    /// Check if either elfs zone assignments fully contains the other.
    fn assignment_containment(&self) -> bool {
        self.0.fully_contains(&self.1) || self.1.fully_contains(&self.0)
    }

    /// Do the zones for elf1 and elf2 overlap at all?
    fn check_overlap(&self) -> bool {
        self.0.overlap(&self.1)
    }
}
