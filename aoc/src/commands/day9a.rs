use std::{collections::HashSet, fmt::Display, path::PathBuf, str::FromStr};

use clap::Parser;

use crate::utils::{slurp_file, ParseError};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day9a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day9a {
    fn main(&self) -> Result<(), DynError> {
        let directions: Vec<Direction> = slurp_file(&self.input)?;
        let mut rope = Rope::new(2);
        for dir in &directions {
            rope.exec(dir);
        }
        eprintln!("{}", rope);
        println!("Points: {:?}", rope.count_unique_tail_locations());
        Ok(())
    }
}

// The idea here is as follows:
// - Record Head move
// - Check distance between head and tail
// - If distance is greater than 1, move tail to previous head location

#[derive(Debug, Clone)]
pub struct Rope {
    knots: Vec<Vec<Point>>,
}

impl Rope {
    pub fn new(knots: usize) -> Self {
        Self { knots: vec![vec![Point::new(0, 0)]; knots] }
    }

    // Head makes only upward moves (doesn't work, )
    // head_old = (4, 0)
    // head_new = (4, 1)
    // tail_old = (3, 0)
    // tail_new = (3, 0)
    // -----
    // head_old = (4, 1)
    // head_new = (4, 2)
    // tail_old = (3, 0)
    // tail_new = (4, 1)

    // Head made a diagonal move (this works)
    // head_old = (3, 0)
    // head_new = (4, 1)
    // tail_old = (2, 0)
    // tail_new = (3, 1)

    /// Make the moves indicated by direction
    pub fn exec(&mut self, direction: &Direction) {
        // eprintln!("Applying {:?}", direction);
        for _ in 0..direction.magnitude() {
            let new_head = direction.increment_point(self.get_latest_knot_point(0));
            self.knots[0].push(new_head);
            // Now update the tails for each segment
            for knot in 1..self.knots.len() {
                // I want the diff from head old to head new.
                let curr_head = self.get_latest_knot_point(knot - 1);
                // If the dist floor of the dist is greater than 1, move the tail of
                // this segment to the previous head position
                let curr_tail = self.get_latest_knot_point(knot);
                let dist = curr_head.dist(self.get_latest_knot_point(knot));
                // eprintln!("--- working on {} ---", knot);
                // eprintln!("{} - curr_head={:?}", knot, curr_head);
                // eprintln!("{} - curr_tail={:?}", knot, curr_tail);
                // eprintln!("{} - dist={:?}", knot, dist);

                if dist.floor() > 1. {
                    let diff = curr_head.diff_old_new(self.get_prev_knot_point(knot - 1));
                    // eprintln!("{} - diff={:?}", knot, diff);
                    if diff.x == 0 || diff.y == 0 {
                        // eprintln!("{} - moved tail to prev head", knot);
                        // If the head didn't move diagonally, then move the tail to the previous head
                        let new_tail = *self.get_prev_knot_point(knot - 1);
                        // eprintln!("{} - new_tail={:?}", knot, new_tail);
                        self.knots[knot].push(new_tail);
                    } else if curr_head.x == curr_tail.x {
                        // Already in the same X plane, move along y
                        // eprintln!("{} - Staying on x plane, moving y", knot);
                        // If the head didn't move diagonally, then move the tail to the previous head
                        // let new_tail = *self.get_prev_knot_point(knot - 1);
                        let new_tail = Point::new(
                            curr_tail.x,
                            curr_tail.y + ((curr_head.y - curr_tail.y) / 2),
                        );
                        // eprintln!("{} - new_tail={:?}", knot, new_tail);
                        self.knots[knot].push(new_tail);
                    } else if curr_head.y == curr_tail.y {
                        // Already in the same X plane, move along y
                        // eprintln!("{} - Staying on y plane, moving x", knot);
                        let new_tail = Point::new(
                            curr_tail.x + ((curr_head.x - curr_tail.x) / 2),
                            curr_tail.y,
                        );
                        // eprintln!("{} - new_tail={:?}", knot, new_tail);
                        self.knots[knot].push(new_tail);
                    } else {
                        // If the head moved diagonally, compute a new pos
                        // eprintln!("{} - computed new tail", knot);
                        let new_tail = self.get_latest_knot_point(knot).apply_diff(&diff);
                        // eprintln!("{} - new_tail={:?}", knot, new_tail);
                        self.knots[knot].push(new_tail);
                    }
                }
            }
        }
        // eprintln!("End location afte {:?}", direction);
        // for i in 0..self.knots.len() {
        //     eprintln!("{}: {:?}", i, self.get_latest_knot_point(i))
        // }
    }

    /// Get the latest position of the knot indicated by `index`.
    fn get_latest_knot_point(&self, index: usize) -> &Point {
        // safe because we will always have one element here
        &self.knots[index][self.knots[index].len() - 1]
    }

    /// Get the previous position of the knot indicated by `index`.
    fn get_prev_knot_point(&self, index: usize) -> &Point {
        // Should be safe since this will never be called unless head has updated
        // in which case there will be two points in the history.
        &self.knots[index][self.knots[index].len() - 2]
    }

    pub fn count_unique_tail_locations(&self) -> usize {
        let mut set: HashSet<(i64, i64)> = HashSet::new();
        for pos in &self.knots[self.knots.len() - 1] {
            set.insert(pos.as_int_tuple());
        }
        set.len()
    }
}

impl Display for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for knot in &self.knots {
            writeln!(f, "{:?}", knot)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn dist(&self, other: &Self) -> f64 {
        (((other.x - self.x) as f64).powi(2) + ((other.y - self.y) as f64).powi(2)).sqrt()
    }

    pub fn diff_old_new(&self, old: &Self) -> Self {
        Self { x: self.x - old.x, y: self.y - old.y }
    }

    pub fn apply_diff(&self, diff: &Self) -> Self {
        Self { x: self.x + diff.x, y: self.y + diff.y }
    }

    fn as_int_tuple(&self) -> (i64, i64) {
        (self.x as i64, self.y as i64)
    }
}

/// Direction and magnitude to move.
#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
}

impl Direction {
    fn magnitude(&self) -> usize {
        match self {
            Direction::Up(m) => *m,
            Direction::Down(m) => *m,
            Direction::Left(m) => *m,
            Direction::Right(m) => *m,
        }
    }

    /// Move a point 1 unit in the given direction, returning the new location
    fn increment_point(&self, current: &Point) -> Point {
        match self {
            Direction::Up(_) => Point::new(current.x, current.y + 1),
            Direction::Down(_) => Point::new(current.x, current.y - 1),
            Direction::Left(_) => Point::new(current.x - 1, current.y),
            Direction::Right(_) => Point::new(current.x + 1, current.y),
        }
    }
}

impl FromStr for Direction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, mag) = s
            .split_once(' ')
            .ok_or_else(|| ParseError::new(format!("Invalid direction: `{s}`")))?;
        let mag = mag
            .parse::<usize>()
            .map_err(|_| ParseError::new(format!("Invalid direction magnitude: `{mag}`.")))?;
        match dir {
            "D" => Ok(Direction::Down(mag)),
            "U" => Ok(Direction::Up(mag)),
            "L" => Ok(Direction::Left(mag)),
            "R" => Ok(Direction::Right(mag)),
            _ => Err(ParseError::new(format!("Invalid direction: `{dir}`"))),
        }
    }
}
