use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use clap::Parser;

use crate::commands::day11a::{monkey_parser, split_on, Item};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day11b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day11b {
    fn main(&self) -> Result<(), DynError> {
        // This isn't complete - we need to do something clever with tracking the worry level, bigint is too slow.
        let mut reader = BufReader::new(File::open(&self.input).unwrap());

        let mut input = String::new();
        let bytes_read = reader.read_to_string(&mut input).unwrap();
        let (remainder, mut monkeys) = monkey_parser::parse_monkeys(&input).unwrap();

        for i in 0..10_000 {
            let monkey_len = monkeys.len();
            let slice = monkeys.as_mut_slice();
            for monkey_index in 0..monkey_len {
                let (left, monkey, right) = split_on(slice, monkey_index);

                while let Some(item) = monkey.items.pop_front() {
                    let new = (*monkey.operation)(item); // Increase worry as item is inspected
                    let new = Item(new.0); // Decrease worry as item is moved on from
                    let index = if new.0 % monkey.test_number == 0 {
                        monkey.test_true_branch
                    } else {
                        monkey.test_false_branch
                    };
                    if index < monkey_index {
                        left[index].items.push_back(new);
                    } else {
                        right[index - (left.len() + 1)].items.push_back(new);
                    }

                    monkey.total_inspections += 1;
                }
            }
        }

        monkeys.sort_by(|a, b| b.total_inspections.cmp(&a.total_inspections));
        eprintln!("{}", monkeys[0].total_inspections * monkeys[1].total_inspections);
        for monkey in monkeys {
            eprintln!("{}: {}", monkey.id, monkey.total_inspections);
        }
        Ok(())
    }
}
