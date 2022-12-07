use std::path::PathBuf;

use clap::Parser;

use crate::{commands::day7a::FileSystem, utils::slurp_file};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day7b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day7b {
    fn main(&self) -> Result<(), DynError> {
        let lines: Vec<String> = slurp_file(&self.input)?;
        let fs = FileSystem::from_bash_history(lines)?;

        let total_space = 70_000_000;
        let update_size: usize = 30_000_000;
        let root_size = fs.root_size();
        let unused_space = total_space - root_size;
        let needed_space = update_size.saturating_sub(unused_space);
        println!("Needed space: {}", needed_space);

        // Find the smallest dir to remove that gets the needed space

        // Find total size of each dir
        // visualization
        // let sizes = fs
        //     .dir_sizes()
        //     .into_iter()
        //     .filter(|(d, size)| *size <= 100_000)
        //     .map(|(d, size)| (d.as_ref().borrow().name().to_owned(), size))
        //     .collect::<Vec<_>>();
        let mut sizes = fs
            .dir_sizes()
            .into_iter()
            .filter(|(d, size)| *size >= needed_space)
            .map(|(d, size)| (d.as_ref().borrow().name().to_owned(), size))
            .collect::<Vec<_>>();
        sizes.sort_by(|a, b| a.1.cmp(&b.1));
        eprintln!("Sizes: {:?}", sizes[0]);
        Ok(())
    }
}
