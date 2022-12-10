use std::{collections::HashSet, fmt::Display, path::PathBuf, thread};

use clap::Parser;

use crate::{commands::day10a::Cpu, utils::slurp_file};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day10b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day10b {
    fn main(&self) -> Result<(), DynError> {
        let instructions = slurp_file(&self.input)?;
        let (mut cpu, rx) = Cpu::new(instructions);

        // Send cpu to another thread to execute
        let handle = thread::spawn(move || {
            cpu.exec();
            // Cpu get's dropped here, along with sender, which will close recv
        });

        let mut crt = Crt::new();

        let mut counter = 0;
        while let Ok((x_reg, cycle_num)) = rx.recv() {
            crt.current_cycle = cycle_num;
            if (crt.current_pixel % 40) as i64 >= x_reg - 1
                && (crt.current_pixel % 40) as i64 <= x_reg + 1
            {
                crt.draw_sprite()
            }
            crt.next_pixel()
        }

        handle.join().expect("Something went wrong on cpu thread");
        println!("{}", crt);
        Ok(())
    }
}

// TODO: if the problem gets built upon more, a separate clock should be made instead of what I'm about to do.

const SPRITE_WIDTH: usize = 3;
const SPRITE: char = '#';
const EMPTY: char = '.';

pub struct Crt {
    screen: Vec<Vec<char>>,
    current_cycle: usize,
    current_pixel: usize,
}

impl Crt {
    pub fn new() -> Self {
        Self {
            screen: vec![
                vec![EMPTY; 40],
                vec![EMPTY; 40],
                vec![EMPTY; 40],
                vec![EMPTY; 40],
                vec![EMPTY; 40],
                vec![EMPTY; 40],
            ],
            current_cycle: 0,
            current_pixel: 0,
        }
    }
}

impl Crt {
    pub fn next_pixel(&mut self) {
        self.current_pixel += 1;
    }

    pub fn draw_sprite(&mut self) {
        let row = self.current_pixel / 40;
        let pixel = self.current_pixel % 40;
        self.screen[row][pixel] = SPRITE;
    }
}

impl Display for Crt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.screen {
            for pixel in row {
                write!(f, "{}", pixel)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Default for Crt {
    fn default() -> Self {
        Self::new()
    }
}
