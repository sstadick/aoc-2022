use std::path::PathBuf;

use clap::Parser;

use crate::utils::slurp_file;

use super::{CommandImpl, DynError};

/// Find the elf with the most calories in their pack.
#[derive(Parser, Debug)]
pub struct Day1b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day1b {
    fn main(&self) -> Result<(), DynError> {
        let lines: Vec<String> = slurp_file(&self.input)?;

        let mut elves = vec![];
        let mut current_elf = Elf::new();

        for line in lines {
            if line.is_empty() {
                elves.push(current_elf);
                current_elf = Elf::new();
            } else {
                let snack = line.parse::<usize>()?;
                current_elf.add_snack(snack);
            }
        }
        // Assumes input ends with a newline, otherwise we'd miss the last elf

        // Note that we could use `elves.sort_by` here, but this is a good chance to look at the PartialOrd and Ord traits
        elves.sort_unstable();

        println!(
            "Total Cals of Top Three Elves: {}",
            elves.iter().rev().map(|e| e.total_calories).take(3).sum::<usize>()
        );

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Elf {
    /// List of snacks for this elf, represented by the calories of each snack
    snacks: Vec<usize>,
    /// Sum of snacks
    total_calories: usize,
}

impl Elf {
    pub fn new() -> Self {
        Self { snacks: vec![], total_calories: 0 }
    }

    pub fn add_snack(&mut self, snack: usize) {
        self.snacks.push(snack);
        self.total_calories += snack;
    }
}

impl Default for Elf {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialOrd for Elf {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.total_calories.partial_cmp(&other.total_calories)
    }
}

impl PartialEq for Elf {
    fn eq(&self, other: &Self) -> bool {
        self.total_calories == other.total_calories
    }
}

impl Ord for Elf {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_calories.cmp(&other.total_calories)
    }
}
impl Eq for Elf {}
