use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;

use crate::utils::{slurp_file, ParseError};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day8a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day8a {
    fn main(&self) -> Result<(), DynError> {
        let trees: Vec<TreeLine> = slurp_file(&self.input)?;
        let mut grid = Grid::new(trees);
        grid.mark_visible();
        println!("{:?}", grid.count_visible());
        Ok(())
    }
}

/// Helper type for implementing FromStr
#[derive(Debug, Clone)]
pub struct TreeLine(Vec<Tree>);

impl FromStr for TreeLine {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut trees = Vec::with_capacity(s.len());
        for c in s.chars() {
            let height = c
                .to_digit(10)
                .ok_or_else(|| ParseError::new(format!("Invalid tree height `{c}`")))?
                as usize;

            trees.push(Tree { height, visible: false, views: [0; 4] });
        }
        Ok(Self(trees))
    }
}

impl TreeLine {
    fn mark_visible_left(&mut self) {
        let mut max_height_seen = self.0[0].height();
        self.0[0].mark_visible();
        for tree in self.0.iter_mut().skip(1) {
            if tree.height() > max_height_seen {
                tree.mark_visible();
                max_height_seen = tree.height();
            }
        }
    }

    fn mark_visible_right(&mut self) {
        let len = self.0.len();
        let mut max_height_seen = self.0[len - 1].height();
        self.0[len - 1].mark_visible();
        for tree in self.0.iter_mut().rev().skip(1) {
            if tree.height() > max_height_seen {
                tree.mark_visible();
                max_height_seen = tree.height();
            }
        }
    }

    fn score_views_left(&mut self, direction: Direction) {
        let size = self.0.len();
        for i in (0..size).rev() {
            // Count the trees to the left of this tree that up to an edge or a tree with >= it's height
            if i == 0 {
                self.0[i].set_view(direction, 0);
                continue;
            }
            let mut score = 0;
            for tree_left in self.0.iter().rev().skip(size - i) {
                match tree_left.height().cmp(&self.0[i].height()) {
                    std::cmp::Ordering::Less => score += 1,
                    std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                        score += 1;
                        break;
                    }
                }
            }
            self.0[i].set_view(direction, score);
        }
    }

    fn score_views_right(&mut self, direction: Direction) {
        let size = self.0.len();
        // for (i, tree) in self.0.iter().enumerate() {
        for i in 0..size {
            // Count the trees to the right of this tree that up to an edge or a tree with >= it's height
            if i == size - 1 {
                self.0[i].set_view(direction, 0);
                continue;
            }
            let mut score = 0;
            for tree_right in self.0.iter().skip(i + 1) {
                match tree_right.height().cmp(&self.0[i].height()) {
                    std::cmp::Ordering::Less => score += 1,
                    std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                        score += 1;
                        break;
                    }
                }
            }
            self.0[i].set_view(direction, score);
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn count_visible(&self) -> usize {
        self.0.iter().filter(|t| t.visible).count()
    }

    fn find_highest_view_score(&self) -> usize {
        self.0.iter().map(|t| t.score()).max().unwrap_or(0)
    }
}

impl Index<usize> for TreeLine {
    type Output = Tree;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for TreeLine {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Display for TreeLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for tree in &self.0 {
            write!(
                f,
                "{}:{}:{:04} ",
                tree.height(),
                if tree.visible { "T" } else { "F" },
                tree.score()
            )?
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Grid {
    trees: Vec<TreeLine>,
}

impl Grid {
    pub fn new(trees: Vec<TreeLine>) -> Self {
        Self { trees }
    }

    pub fn score_views(&mut self) {
        for treeline in self.trees.iter_mut() {
            treeline.score_views_left(Direction::Left);
            treeline.score_views_right(Direction::Right);
        }
        self.transpose();
        for treeline in self.trees.iter_mut() {
            treeline.score_views_left(Direction::Top);
            treeline.score_views_right(Direction::Bottom);
        }
    }

    pub fn find_highest_view_score(&self) -> usize {
        self.trees.iter().map(|t| t.find_highest_view_score()).max().unwrap_or(0)
    }

    /// Mark all trees visible from the viewing perspective of each side
    pub fn mark_visible(&mut self) {
        // Viewing from the Left and the Right
        // println!("{}", self);
        for treeline in self.trees.iter_mut() {
            treeline.mark_visible_left();
            treeline.mark_visible_right();
        }
        // println!("{}", self);
        self.transpose();
        // println!("{}", self);
        for treeline in self.trees.iter_mut() {
            treeline.mark_visible_left();
            treeline.mark_visible_right();
        }
        // println!("{}", self);
    }

    pub fn count_visible(&self) -> usize {
        self.trees.iter().map(|t| t.count_visible()).sum()
    }

    /// Transpose the grid of trees.
    ///
    /// Assumes square matrix
    ///
    /// This will use 2x mem
    fn transpose(&mut self) {
        let size = self.trees[0].len();
        assert_eq!(size, self.trees.len(), "Grid isn't square");
        // I bet there's a clever way to do this with iterators
        for i in 0..size {
            for j in i + 1..size {
                let tmp = self.trees[j][i];
                self.trees[j][i] = self.trees[i][j];
                self.trees[i][j] = tmp;
            }
        }
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for treeline in &self.trees {
            writeln!(f, "{}", treeline)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tree {
    height: usize,
    visible: bool,
    views: [usize; 4],
}

impl Tree {
    pub fn mark_visible(&mut self) {
        self.visible = true;
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn score(&self) -> usize {
        self.views[0] * self.views[1] * self.views[2] * self.views[3]
    }

    pub fn set_view(&mut self, direction: Direction, score: usize) {
        self.views[direction as usize] = score;
    }
}

impl FromStr for Tree {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let height = s
            .parse::<usize>()
            .map_err(|e| ParseError::new(format!("Invalid tree height `{s}`: {e}")))?;
        Ok(Tree { height, visible: false, views: [0; 4] })
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum Direction {
    Left = 0,
    Right = 1,
    Top = 2,
    Bottom = 3,
}
