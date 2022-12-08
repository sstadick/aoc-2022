use std::path::PathBuf;

use clap::Parser;

use crate::{
    commands::day8a::{Grid, TreeLine},
    utils::slurp_file,
};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day8b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day8b {
    fn main(&self) -> Result<(), DynError> {
        let trees: Vec<TreeLine> = slurp_file(&self.input)?;
        let mut grid = Grid::new(trees);
        grid.score_views();
        println!("{:?}", grid.find_highest_view_score());
        Ok(())
    }
}
