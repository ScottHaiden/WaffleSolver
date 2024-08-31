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

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;
use std::{cmp, env, fmt, fs, io, process};

use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Coord {
    row: usize,
    col: usize,
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "({},{})", self.row, self.col);
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Swap {
    a: Coord,
    b: Coord,
}

impl Swap {
    fn new(a: Coord, b: Coord) -> Self {
        let first = cmp::min(a, b);
        let second = cmp::max(a, b);
        return Self { a: first, b: second };
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
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
    fn new(path: &Path) -> io::Result<Self> {
        let cells: Vec<String> = fs::read_to_string(path)?
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

    fn swap(&self, swap: Swap) -> Self {
        let Swap { a, b } = swap;
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

    fn score(&self, other: &Self) -> usize {
        // Score is just the number of different cells between itself and the target.
        return self.diff(other).len();
    }

    fn display(&self) -> String {
        return self.cells.iter()
            .map(|row| row.iter().collect::<String>())
            .join("\n");
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct State<'a> {
    cur: WaffleBoard,
    dest: &'a WaffleBoard,
}

impl<'a> State<'a> {
    fn new(cur: WaffleBoard, dest: &'a WaffleBoard) -> Self {
        return Self {
            cur: cur,
            dest: dest,
        };
    }
}

impl<'a> Ord for State<'a> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        return self.cur.score(self.dest).cmp(&other.cur.score(other.dest));
    }
}

impl<'a> PartialOrd for State<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        return Some(self.cmp(other));
    }
}

fn find_swaps(from: &WaffleBoard, into: &WaffleBoard) -> Option<Vec<Swap>> {
    let get_swaps = |board: &WaffleBoard| -> Vec<Swap> {
        let differences = board.diff(into);
        if differences.len() == 0 { return Vec::new(); }
        assert!(differences.len() > 1, "Expected at least 2 differences; nothing to swap!");
        // Find all unique possible swaps; note that Swap::new already acts as a sorted pair of
        // coordintes, so any two swaps of point a and point b are identical.
        let uniques: HashSet<Swap> = differences.into_iter()
            .combinations(2)
            .map(|pair| Swap::new(pair[0], pair[1]))
            .collect();
        // sort the possible swaps, so that the output will be deterministic.
        let mut sorted: Vec<Swap> = uniques.into_iter().collect();
        sorted.sort();
        return sorted;
    };

    let mut map: HashMap<WaffleBoard, Vec<Swap>> = HashMap::new();
    let mut states: BTreeSet<State> = BTreeSet::new();

    map.insert(from.clone(), Vec::new());
    states.insert(State::new(from.clone(), into));

    // BTreeSet is a max heap; pop will return the highest item. That means, we will continually
    // find the board with the fewest differences between itself and the target.
    while let Some(State { cur, dest: _ }) = states.pop_first() {
        let prev_path = map.get(&cur).unwrap();
        if prev_path.len() > 10 { continue; }
        let steps: Vec<Swap> = prev_path.iter().copied().collect();
        let cur_score = cur.score(into);
        if cur_score == 0 { return Some(steps); }
        for swap in get_swaps(&cur) {
            let next = cur.swap(swap);

            // If this swap makes our position worse (it is more different than cur is), skip it.
            if next.score(into) >= cur_score { continue; }

            let prev_len = match map.get(&next) {
                None => None,
                Some(prev_path) => Some(prev_path.len()),
            };

            // If we've already seen this state before, and the old path is no shorter than the
            // current path (ie, we have no improvement), then continue.
            if prev_len.is_some() && prev_len.unwrap() <= steps.len() + 1 { continue; }

            // Otherwise we have a new board state, or we have found a faster route to an old board
            // state, so update the map and re-add the current board state for re-evaluation.
            let mut path: Vec<Swap> = steps.iter().copied().collect();
            path.push(swap);
            map.insert(next.clone(), path);
            states.insert(State::new(next, into));
        }
    }

    return None;
}

fn show_transformation(cur: &WaffleBoard, steps: &[Swap]) {
    println!("{}", cur.display());
    if steps.len() == 0 { return; }
    let step: Swap = steps[0];
    println!("- swap '{}' at {} with '{}' at {}",
             cur.get(step.a), step.a,
             cur.get(step.b), step.b);
    return show_transformation(&cur.swap(step), &steps[1..]);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Expected 2 command line arguments but got {}", args.len() - 1);
        process::exit(1);
    }

    let from_board = WaffleBoard::new(Path::new(&args[1]))?;
    let into_board = WaffleBoard::new(Path::new(&args[2]))?;

    match find_swaps(&from_board, &into_board) {
        Some(path) => show_transformation(&from_board, &path),
        None       => println!("Could not find a path."),
    };

    return Ok(());
}
