use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;

use crate::commands::day5a::{Command, Crane, Crane9001, StackParser};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day5b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day5b {
    fn main(&self) -> Result<(), DynError> {
        let mut reader = BufReader::new(File::open(&self.input)?);

        let mut stacks_buffer = Vec::new();
        // Fill stacks first
        let mut line_buffer = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut line_buffer) {
            if line_buffer == "\n" {
                break;
            }

            stacks_buffer.extend_from_slice(line_buffer.as_bytes());
            line_buffer.clear();
        }

        let stacks = StackParser::parse_buffer(&stacks_buffer);
        let mut crane = Crane9001::new(stacks);

        // Now read and execute commands on the stacks
        while let Ok(bytes_read) = reader.read_line(&mut line_buffer) {
            if bytes_read == 0 {
                break;
            }

            let command: Command = line_buffer.parse()?;
            crane.exec(&command);

            line_buffer.clear();
        }

        // Now get the top crate for each stack
        let top_crates = crane.top_crates();
        println!("Top Crates: {}", top_crates);

        Ok(())
    }
}
