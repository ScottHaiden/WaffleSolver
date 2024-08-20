// This file is part of WaffleSolver.
//
// WaffleSolver is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// WaffleSolver is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with WaffleSolver. If
// not, see <https://www.gnu.org/licenses/>.
//
use std::path::Path;
use itertools::Itertools;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Coord {
    row: usize,
    col: usize,
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "({},{})", self.row, self.col);
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct Swap {
    a: Coord,
    b: Coord,
}

impl Swap {
    fn new(a: Coord, b: Coord) -> Self {
        let first = std::cmp::min(a, b);
        let second = std::cmp::max(a, b);
        return Self { a: first, b: second };
    }
}

#[derive(Debug, Eq, PartialEq)]
struct WaffleBoard {
    cells: Vec<Vec<char>>
}

impl Clone for WaffleBoard {
    fn clone(&self) -> Self {
        return Self {
            cells: self.cells.iter().cloned().collect(),
        };
    }
}

impl WaffleBoard {
    fn new(path: &Path) -> std::io::Result<Self> {
        let cells: Vec<String> = std::fs::read_to_string(path)?
            .lines()
            .map(|line| line.to_owned())
            .collect();

        assert!(cells.len() > 0, "Expected at least one line!");
        let len = cells[0].len();
        assert!(cells.iter().all(|line| line.len() == len),
                "Expected all lines to be the same length!");

        return Ok(Self {
            cells: cells.into_iter()
                        .map(|line| line.chars().collect())
                        .collect(),
        });
    }

    fn swap(&self, a: Coord, b: Coord) -> Self {
        let mut c = self.cells.clone();
        (c[a.row][a.col], c[b.row][b.col]) = (c[b.row][b.col], c[a.row][a.col]);
        return Self { cells: c };
    }

    fn size(&self) -> (usize, usize) {
        return (self.cells.len(), self.cells[0].len());
    }

    fn get(&self, coord: Coord) -> char {
        return self.cells[coord.row][coord.col];
    }

    fn diff(&self, other: &Self) -> Vec<Coord> {
        let (selfsize, othersize) = (self.size(), other.size());
        let fmt = |size: (usize, usize)| format!("{}x{}", size.0, size.1);
        assert!(self.size() == other.size(),
                "Size mismatch: {} vs {}", fmt(selfsize), fmt(othersize));

        let mut ret = Vec::new();
        for row in 0..selfsize.0 {
            for col in 0..selfsize.1 {
                let coord = Coord{ row: row, col: col };
                let selfcell = self.get(coord);
                let othercell = other.get(coord);
                if selfcell == othercell { continue; }
                ret.push(coord);
            }
        }
        return ret;
    }

    fn score(&self, other: &Self) -> i32 { return -(self.diff(&other).len() as i32); }

    fn display(&self) -> String {
        return self.cells.iter()
            .map(|row| row.iter().collect::<String>())
            .join("\n");
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct State {
    cur: WaffleBoard,
    dest: WaffleBoard,
    steps: Vec<Swap>,
}

impl State {
    fn new(cur: WaffleBoard, dest: &WaffleBoard, steps: Vec<Swap>) -> Self {
        return Self {
            cur: cur,
            dest: dest.clone(),
            steps: steps,
        };
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.cur.score(&self.dest).cmp(&other.cur.score(&other.dest));
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other));
    }
}

fn find_swaps(from: &WaffleBoard, into: &WaffleBoard) -> Option<Vec<Swap>> {
    let get_swaps = |board: &WaffleBoard| -> Vec<Swap> {
        let differences = board.diff(into);
        if differences.len() == 0 { return vec![]; }
        assert!(differences.len() >= 1, "Expected at least 2 differences; nothing to swap!");
        let mut swaps: Vec<Swap> = differences.into_iter()
            .combinations(2)
            .map(|pair| Swap::new(pair[0], pair[1]))
            .collect();
        swaps.sort();
        swaps.dedup();
        return swaps;
    };

    let mut states: BinaryHeap<State> = BinaryHeap::new();
    states.push(State::new(from.clone(), into, Vec::new()));

    while let Some(State { cur, dest, steps }) = states.pop() {
        if cur == dest { return Some(steps.into_iter().collect()); }
        for swap in get_swaps(&cur) {
            let next = cur.swap(swap.a, swap.b);
            let mut path: Vec<Swap> = steps.iter().cloned().collect();
            path.push(swap);
            let state = State::new(next, into, path);
            states.push(state);
        }
    }

    return None;
}

fn show_transformation(cur: &WaffleBoard, steps: &[Swap]) {
    println!("{}", cur.display());
    if steps.len() == 0 { return; }
    let step = &steps[0];
    println!("- swap '{}' at {} with '{}' at {}",
             cur.get(step.a), step.a,
             cur.get(step.b), step.b);
    return show_transformation(&cur.swap(step.a, step.b), &steps[1..]);
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Expected 2 command line arguments but got {}", args.len() - 1);
        std::process::exit(1);
    }

    let from_board = WaffleBoard::new(Path::new(&args[1]))?;
    let into_board = WaffleBoard::new(Path::new(&args[2]))?;

    match find_swaps(&from_board, &into_board) {
        Some(path) => show_transformation(&from_board, &path),
        None       => println!("Could not find a path."),
    };

    return Ok(());
}
