use std::path::PathBuf;

use clap::Parser;

use crate::{commands::day6a::Message, utils::slurp_bytes};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day6b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day6b {
    fn main(&self) -> Result<(), DynError> {
        let bytes = slurp_bytes(&self.input)?;
        let start = Message::detect_message_start(&bytes);
        println!("Start: {}", start);
        Ok(())
    }
}
