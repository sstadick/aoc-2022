use std::path::PathBuf;

use clap::Parser;

use crate::{
    commands::{day9a::Direction, day9a::Rope},
    utils::slurp_file,
};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day9b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day9b {
    fn main(&self) -> Result<(), DynError> {
        let directions: Vec<Direction> = slurp_file(&self.input)?;
        let mut rope = Rope::new(10);
        for dir in &directions {
            rope.exec(dir);
        }
        println!("{}", rope);
        println!("Points: {:?}", rope.count_unique_tail_locations());
        Ok(())
    }
}
