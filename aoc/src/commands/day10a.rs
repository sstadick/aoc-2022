use std::{
    collections::HashSet,
    hash::Hash,
    path::PathBuf,
    str::FromStr,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

use clap::Parser;

use crate::utils::{slurp_file, ParseError};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day10a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day10a {
    fn main(&self) -> Result<(), DynError> {
        let cycles_to_inspect = &[20, 60, 100, 140, 180, 220];
        let inspect: HashSet<usize> = HashSet::from_iter(cycles_to_inspect.iter().copied());
        // let inspect = HashSet::from_iter([1, 2, 3, 4, 5, 6].into_iter());
        let instructions = slurp_file(&self.input)?;
        let (mut cpu, rx) = Cpu::new(instructions);

        // Send cpu to another thread to execute
        let handle = thread::spawn(move || {
            cpu.exec();
            // Cpu get's dropped here, along with sender, which will close recv
        });

        let mut result = vec![];
        let mut counter = 0;
        while let Ok((x_reg, cycle_num)) = rx.recv() {
            if inspect.contains(&cycle_num) {
                result.push(x_reg);
            }
        }

        handle.join().expect("Something went wrong on cpu thread");

        eprintln!("Inspection results: {:?}", result);
        let total = result
            .iter()
            .zip(cycles_to_inspect.iter())
            .map(|(x, y)| x * *y as i64)
            .inspect(|x| println!("{}", x))
            .sum::<i64>();
        eprintln!("Total: {}", total);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Cpu {
    /// The X register
    x_reg: i64,
    /// Pointer to the current instruction being executed
    current_instruction: usize,
    /// Counter tracking the number of cycles this instruction has been computing for
    instruction_cycle_counter: usize,

    /// The instructions to execute
    instructions: Vec<Instruction>,
    /// Counter tracking the current cycle number
    current_cycle: usize,

    // A channel that will have the x_reg value and the current cycle number sent once per cycle.
    x_reg_channel: Sender<(i64, usize)>,
}

impl Cpu {
    pub fn new(instructions: Vec<Instruction>) -> (Self, Receiver<(i64, usize)>) {
        assert!(!instructions.is_empty());
        let (tx, rx) = channel();
        (
            Self {
                current_cycle: 0,
                instruction_cycle_counter: 0,
                current_instruction: 0,
                x_reg: 1,
                instructions,
                x_reg_channel: tx,
            },
            rx,
        )
    }

    /// Execute the instructions, inspect the x_reg at the given cycle number and return it's values
    pub fn exec(&mut self) {
        loop {
            self.current_cycle += 1;
            self.instruction_cycle_counter += 1;
            // Inspect
            self.x_reg_channel
                .send((self.x_reg, self.current_cycle))
                .expect("Failed to x_reg value on sender");
            if self.instructions[self.current_instruction].cost() == self.instruction_cycle_counter
            {
                // Instruction is done computing, perform any register modifications
                match &self.instructions[self.current_instruction] {
                    Instruction::Addx(v) => self.x_reg += *v,
                    Instruction::Noop => (),
                }
                self.current_instruction += 1;
                self.instruction_cycle_counter = 0;
            }

            if self.current_instruction == self.instructions.len() {
                break;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Addx(i64),
    Noop,
}

impl Instruction {
    /// The cost, in cpu cycles, for this instruction
    const fn cost(&self) -> usize {
        match self {
            Instruction::Addx(_) => 2,
            Instruction::Noop => 1,
        }
    }
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("noop") {
            Ok(Instruction::Noop)
        } else if s.starts_with("addx") {
            let (_instr, v) = s
                .split_once(' ')
                .ok_or_else(|| ParseError::new(format!("Invalid addx instructio: `{s}`")))?;
            let v = v.parse::<i64>().map_err(|_e| {
                ParseError::new(format!("Invalid value in addx instructiong: `{v}`"))
            })?;
            Ok(Instruction::Addx(v))
        } else {
            unimplemented!()
        }
    }
}
