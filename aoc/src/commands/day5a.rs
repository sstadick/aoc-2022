use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, Cursor, Read},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

use crate::utils::ParseError;

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day5a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day5a {
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
        let mut crane = Crane9000::new(stacks);

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

/// A command for the crane operator
pub struct Command {
    /// Number of crates to move
    number_of_crates: usize,
    /// The stack to move them from, 0-based
    from: usize,
    /// The stack to move them to, 0-based
    to: usize,
}

impl FromStr for Command {
    type Err = ParseError;

    // Sneaking suspicion we're going to need multiple command types in the future / end up building a stack based virtual machine
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace().collect::<Vec<&str>>();
        let number_of_crates = tokens[1]
            .parse::<usize>()
            .map_err(|_e| ParseError::new(format!("Unable to parse `{}`", s)))?;
        let from = tokens[3]
            .parse::<usize>()
            .map_err(|_e| ParseError::new(format!("Unable to parse `{}`", s)))?
            - 1;
        let to = tokens[5]
            .parse::<usize>()
            .map_err(|_e| ParseError::new(format!("Unable to parse `{}`", s)))?
            - 1;
        Ok(Self { number_of_crates, from, to })
    }
}

pub trait Crane {
    fn exec(&mut self, cmd: &Command);
}

pub struct Crane9000 {
    stacks: Vec<Stack>,
}

impl Crane9000 {
    /// Create a new [`Crane`] object with an initial stack state
    pub fn new(stacks: Vec<Stack>) -> Self {
        Self { stacks }
    }
    fn top_crates(&self) -> String {
        let mut buffer = String::with_capacity(self.stacks.len());
        for stack in &self.stacks {
            let c = stack.peek_front().map(|c| c.0).unwrap_or(' ');
            buffer.push(c);
        }
        buffer
    }
}

impl Crane for Crane9000 {
    /// Execute a move command.
    fn exec(&mut self, cmd: &Command) {
        for _ in 0..cmd.number_of_crates {
            let container = self.stacks[cmd.from]
                .pop_front()
                .unwrap_or_else(|| panic!("No containers on stack: {:?}", &self.stacks[cmd.from]));
            self.stacks[cmd.to].push_front(container)
        }
    }
}

pub struct Crane9001 {
    stacks: Vec<Stack>,
}

impl Crane9001 {
    /// Create a new [`Crane`] object with an initial stack state
    pub fn new(stacks: Vec<Stack>) -> Self {
        Self { stacks }
    }

    pub fn top_crates(&self) -> String {
        let mut buffer = String::with_capacity(self.stacks.len());
        for stack in &self.stacks {
            let c = stack.peek_front().map(|c| c.0).unwrap_or(' ');
            buffer.push(c);
        }
        buffer
    }
}

impl Crane for Crane9001 {
    /// Execute a move command.
    fn exec(&mut self, cmd: &Command) {
        let to_add = self.stacks[cmd.from].take_stack(cmd.number_of_crates);
        self.stacks[cmd.to].push_stack(to_add);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Container(char);

#[derive(Debug, Clone)]
pub struct Stack(VecDeque<Container>);

impl Stack {
    pub fn pop_front(&mut self) -> Option<Container> {
        self.0.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<Container> {
        self.0.pop_back()
    }

    pub fn push_front(&mut self, container: Container) {
        self.0.push_front(container)
    }

    pub fn peek_front(&self) -> Option<&Container> {
        self.0.front()
    }

    /// Take N crates off the front of the stack.
    ///
    /// If less than N crates are available, this will return all available.
    pub fn take_stack(&mut self, size: usize) -> Stack {
        let new = self.0.split_off(size);
        let old = std::mem::replace(&mut self.0, new);
        Stack(old)
    }

    /// Push a stack of crates onto the front of this stack.
    pub fn push_stack(&mut self, mut stack: Stack) {
        // TODO: flip direction of stack usage so that we could use append here instead
        while let Some(container) = stack.pop_back() {
            self.push_front(container);
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum Token {
    /// The space that separates two containers
    ContainerSep = b' ',
    /// A newline indicating a new level
    LevelSep = b'\n',
    /// Left bracket
    LBracket = b'[',
    RBracket = b']',
    Unknown,
}

impl Token {
    const fn value(&self) -> u8 {
        *self as u8
    }

    const fn from_u8(value: u8) -> Self {
        use Token::*;
        match value {
            b' ' => ContainerSep,
            b'\n' => LevelSep,
            b'[' => LBracket,
            b']' => RBracket,
            _ => Unknown,
        }
    }
}

pub struct StackParser;

impl StackParser {
    const CONTAINER_WIDTH: usize = 3;
    const SEP_WIDTH: usize = 1;

    pub fn parse_buffer(buffer: &[u8]) -> Vec<Stack> {
        let mut stacks = vec![];
        let mut stack_cursor = 0;
        let mut cursor = 0;
        loop {
            // Dirty hack, peek ahead to seek if we've hit the bottom
            if buffer[cursor + 1] == b'1' {
                break;
            }
            // Check for container
            let maybe_container = Self::try_consume_container(buffer, cursor);
            cursor += Self::CONTAINER_WIDTH;
            if stacks.len() == stack_cursor {
                stacks.push(Stack(VecDeque::new()))
            }
            if let Some(container) = maybe_container {
                stacks[stack_cursor].0.push_back(container);
            }
            stack_cursor += 1;

            // Check for either level separator or container separator
            if let Some(sep) = Self::try_consume_sep(buffer, cursor) {
                match sep {
                    Token::ContainerSep => (),
                    Token::LevelSep => {
                        stack_cursor = 0;
                    }
                    _ => unreachable!(),
                }
                cursor += Self::SEP_WIDTH;
            } else {
                unreachable!()
            }
        }
        stacks
    }

    fn try_consume_container(buffer: &[u8], cursor: usize) -> Option<Container> {
        if buffer[cursor] == Token::LBracket as u8 {
            Some(Container(buffer[cursor + 1] as char))
        } else {
            None
        }
    }

    fn try_consume_sep(buffer: &[u8], cursor: usize) -> Option<Token> {
        use Token::*;
        match Token::from_u8(buffer[cursor]) {
            ContainerSep => Some(ContainerSep),
            LevelSep => Some(LevelSep),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::StackParser;

    #[test]
    fn test_parse_example() {
        let buffer = "    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 
 ";
        let mut stacks = StackParser::parse_buffer(buffer.as_bytes());
        assert_eq!(stacks.len(), 3);
        assert_eq!(stacks[0].0.pop_front().unwrap().0, 'N');
        assert_eq!(stacks[2].0.pop_front().unwrap().0, 'P');
    }
}
