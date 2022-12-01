use std::path::PathBuf;

use clap::Parser;

use crate::utils::slurp_file;

use super::{CommandImpl, DynError};

/// Find the elf with the most calories in their pack.
#[derive(Parser, Debug)]
pub struct Day1a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day1a {
    fn main(&self) -> Result<(), DynError> {
        let lines: Vec<String> = slurp_file(&self.input)?;
        // We don't need a list of elves for this task, but I'm betting we will for part 2
        let mut elves = vec![];
        let mut current_elf = Elf::new();
        let mut most_cals_seen = 0;

        for line in lines {
            if line.is_empty() {
                if current_elf.total_calories > most_cals_seen {
                    most_cals_seen = current_elf.total_calories
                }
                elves.push(current_elf);
                current_elf = Elf::new();
            } else {
                let snack = line.parse::<usize>()?;
                current_elf.add_snack(snack);
            }
        }
        // Assumes input ends with a newline, otherwise we'd miss the last elf
        println!("Most Cals on an Elf: {}", most_cals_seen);

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
