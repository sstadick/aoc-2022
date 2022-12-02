use std::{error::Error, path::PathBuf, str::FromStr};

use clap::Parser;

use crate::utils::{slurp_file, ParseError};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day2b {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day2b {
    fn main(&self) -> Result<(), DynError> {
        let rounds: Vec<Round> = slurp_file(&self.input)?;
        let total = rounds.into_iter().map(|r| r.score()).sum::<usize>();
        println!("Total Score: {}", total);

        Ok(())
    }
}

/// A hand sign represents a valid sign in rock paper scissors.
///
/// The value of each `HandSign` is the score when used.
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum HandSign {
    Rock = 1,
    Paper = 2,
    Scissors = 3,
}

impl HandSign {
    fn new(symbol: &str) -> Result<Self, ParseError> {
        use HandSign::*;
        match symbol {
            "A" => Ok(Rock),
            "B" => Ok(Paper),
            "C" => Ok(Scissors),
            _ => Err(ParseError::new(format!("Invalid HandSign: `{}`", symbol))),
        }
    }

    /// Get the value of the sign
    fn value(&self) -> usize {
        *self as u8 as usize
    }
}

/// The scoring for a round result.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RoundResult {
    Loss = 0,
    Draw = 3,
    Win = 6,
}

impl RoundResult {
    fn new(symbol: &str) -> Result<Self, ParseError> {
        use RoundResult::*;
        match symbol {
            "X" => Ok(Loss),
            "Y" => Ok(Draw),
            "Z" => Ok(Win),
            _ => Err(ParseError::new(format!("Invalid RoundResult: `{}`", symbol))),
        }
    }

    fn value(&self) -> usize {
        *self as u8 as usize
    }

    /// Compare two [`HandSign`]s and, from the perspective of player1,
    /// declare a loss, win, or draw.
    fn compete(&player1: &HandSign, player2: &HandSign) -> Self {
        use HandSign::*;
        use RoundResult::*;
        match (player1, player2) {
            (Rock, Rock) | (Scissors, Scissors) | (Paper, Paper) => Draw,
            (Rock, Scissors) | (Paper, Rock) | (Scissors, Paper) => Win,
            (Rock, Paper) | (Paper, Scissors) | (Scissors, Rock) => Loss,
        }
    }
}

/// A round represents each players hand signs.
pub struct Round {
    them: HandSign,
    needed_result: RoundResult,
}

impl Round {
    /// Score the round.
    ///
    /// Scoring is the sum of the following:
    ///
    /// - HandSign score (1 for Rock, 2 for Paper, 3 for Scissors)
    /// - Outcome score (0 for loss, 3 for draw, 6 for win)
    pub fn score(&self) -> usize {
        let us = self.determine_hand_sign();
        us.value() + self.needed_result.value()
    }

    pub fn determine_hand_sign(&self) -> HandSign {
        use HandSign::*;
        use RoundResult::*;
        match (self.needed_result, self.them) {
            (Loss, Rock) => Scissors,
            (Loss, Paper) => Rock,
            (Loss, Scissors) => Paper,
            (Draw, Rock) => Rock,
            (Draw, Paper) => Paper,
            (Draw, Scissors) => Scissors,
            (Win, Rock) => Paper,
            (Win, Paper) => Scissors,
            (Win, Scissors) => Rock,
        }
    }
}

impl FromStr for Round {
    type Err = ParseError;

    /// Impl FromStr for [`Round`] so that [`slurp_file`] can parse the strings for us
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (them, needed_result) =
            s.split_once(' ').ok_or_else(|| ParseError::new(format!("Invalid line: '{}'", s)))?;

        Ok(Round { needed_result: RoundResult::new(needed_result)?, them: HandSign::new(them)? })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_example() {
        let input = vec!["A Y", "B X", "C Z"];
        let rounds = input
            .into_iter()
            .map(|line| line.parse::<Round>())
            .collect::<Result<Vec<Round>, ParseError>>()
            .unwrap();
        let score = rounds.into_iter().map(|r| r.score()).sum::<usize>();
        assert_eq!(score, 12);
    }
}
