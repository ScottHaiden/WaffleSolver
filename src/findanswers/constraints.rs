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

use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

fn cell_index(cell: usize) -> Option<usize> {
    return match cell & 1usize {
        0 => Some(cell / 2),
        1 => None,
        _ => panic!("Something went horribly wrong!"),
    };
}

#[derive(Debug, Clone)]
pub struct Constraint {
    constraints: HashMap<usize, char>,
}

impl Constraint {
    pub fn new() -> Self {
        return Self { constraints: HashMap::new() };
    }

    pub fn from(pattern: &str) -> Self {
        let mut constraints = HashMap::new();

        for (i, c) in pattern.chars().enumerate() {
            if c == '?' { continue; }
            constraints.insert(i, c);
        }

        return Self {
            constraints: constraints,
        };
    }

    pub fn with(&self, idx: usize, val: char) -> Self {
        let mut constraints = self.constraints.clone();
        constraints.insert(idx, val);
        return Self { constraints };
    }

    pub fn matches(&self, word: &str) -> bool {
        for (i, c) in word.chars().enumerate() {
            if let Some(&expected) = self.constraints.get(&i) {
                if expected != c { return false; }
            }
        }

        return true;
    }

    pub fn get(&self, index: usize) -> Option<char> {
        return self.constraints.get(&index).copied();
    }

    pub fn num_set(&self) -> usize { return self.constraints.len(); }
}

#[derive(Debug, Clone)]
pub struct ConstraintBoard {
    rows: Vec<Constraint>,
    cols: Vec<Constraint>,
    unused: HashMap<char, usize>,
}

impl ConstraintBoard {
    pub fn from_file(path: &Path) -> io::Result<Self> {
        let cells: Vec<String> = fs::read_to_string(path)?
            .lines()
            .map(&str::to_owned)
            .collect();

        assert!(cells.len() > 0, "Expected at least one line!");
        let len = cells[0].len();
        assert!(cells.iter().all(|line| line.len() == len),
                "Expected all lines to be the same length!");

        let chars: HashMap<char, usize> = cells.iter()
            .map(|line| line.chars())
            .flatten()
            .filter(&char::is_ascii_alphanumeric)
            .fold(HashMap::new(), |acc, c| {
                let mut ret = acc;
                let lowercase = c.to_lowercase().collect::<Vec<char>>();
                assert!(lowercase.len() == 1, "Multiletter lowercase!");
                let entry = ret.entry(lowercase[0]).or_default();
                *entry += 1;
                return ret;
            });

        let mut ret = Self {
            rows: vec![Constraint::new(); len / 2 + 1],
            cols: vec![Constraint::new(); len / 2 + 1],
            unused: chars,
        };

        for (row, rowstr) in cells.iter().enumerate() {
            for (col, cell) in rowstr.chars().enumerate() {
                if !cell.is_uppercase() { continue; }
                let lower = cell.to_lowercase().next().unwrap();
                ret = match ret.with(row, col, lower) {
                    Some(board) => board,
                    None => panic!("Invalid board"),
                };
            }
        }

        return Ok(ret);
    }

    pub fn get(&self, row: usize, col: usize) -> Option<char> {
        if let Some(row_idx) = cell_index(row) {
            return self.rows[row_idx].get(col);
        }
        if let Some(col_idx) = cell_index(col) {
            return self.cols[col_idx].get(row);
        }
        return None;
    }

    pub fn with(&self, row: usize, col: usize, val: char) -> Option<Self> {
        // First check if the cell is already set. If it is, and it's set to what we're setting it
        // to, we can just do nothing. If there's a mismatch, that's an error. We're not handling
        // the error at this point, as for now that's just considered a caller error. That's a good
        // candidate for something to change later on.
        if let Some(cur) = self.get(row, col) {
            if cur == val { return Some(self.clone()); }
            panic!("Cannot set ({}, {}) to {}: Already set to {}", row, col, val, cur);
        }

        // If we get here, the cell is empty. Check if we even have the budget for this new
        // character. If we do, find how many we had left over and subtract one, that's how many of
        // this character we have now.
        let remaining = match self.unused.get(&val) {
            None => return None,
            Some(0) => panic!("Invalid board!"),
            Some(&remainder) => remainder,
        } - 1;

        // Copy the fields.
        let mut rows = self.rows.clone();
        let mut cols = self.cols.clone();
        let mut unused = self.unused.clone();

        if remaining == 0 {
            unused.remove(&val);
        } else {
            *unused.get_mut(&val).unwrap() -= 1;
        }

        if let Some(constraint_row) = cell_index(row) {
            let row = rows.get_mut(constraint_row).unwrap();
            *row = row.with(col, val);
        }
        if let Some(constraint_col) = cell_index(col) {
            let col = cols.get_mut(constraint_col).unwrap();
            *col = col.with(row, val);
        }

        return Some(Self {
            rows: rows,
            cols: cols,
            unused: unused,
        });
    }

    pub fn get_all_words(&self) -> Vec<(Constraint, Vec<(usize, usize)>)> {
        let len = self.rows.len() * 2 - 1;
        let mut ret = Vec::new();

        for (row, constraint) in self.rows.iter().enumerate() {
            if constraint.num_set() == len { continue; }
            let row_idx = row * 2;
            let cells = (0..len).map(|c| (row_idx, c)).collect();
            ret.push((constraint.clone(), cells));
        }

        for (col, constraint) in self.cols.iter().enumerate() {
            if constraint.num_set() == len { continue; }
            let col_idx = col * 2;
            let cells = (0..len).map(|r| (r, col_idx)).collect();
            ret.push((constraint.clone(), cells));
        }

        return ret;
    }
}

impl ToString for ConstraintBoard {
    fn to_string(&self) -> String {
        let len = self.rows.len() * 2 - 1;
        let mut lines = Vec::new();
        for row in 0..len {
            let mut cur_row = Vec::new();
            for col in 0..len {
                let cur_cell = match self.get(row, col) {
                    Some(c) => c,
                    None => ' ',
                };
                cur_row.push(cur_cell);
            }
            lines.push(cur_row.into_iter().collect::<String>());
        }

        return lines.join("\n");
    }
}
