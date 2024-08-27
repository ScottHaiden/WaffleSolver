mod constraints;

use std::path::Path;
use std::fs;
use constraints::ConstraintBoard;

type Coord = (usize, usize);

fn try_word(board: ConstraintBoard, word: &str, indices: &[Coord]) -> Option<ConstraintBoard> {
    assert!(word.len() == indices.len(), "Word and indices are different lengths!");
    if word.len() == 0 { return Some(board); }
    let c: char = word.chars().next().unwrap();
    let (row, col) = indices[0];

    return match board.with(row, col, c) {
        Some(next) => try_word(next, &word[1..], &indices[1..]),
        None => None,
    };
}

fn find_solutions(board: ConstraintBoard, wordlist: &[&str]) {
    let words = board.get_all_words();
    if words.len() == 0 {
        println!("{}", board.to_string());
        println!();
        return;
    }

    let (constraint, indices) = words.into_iter().next().unwrap();
    let possible_words = wordlist.iter()
        .filter(|word| constraint.matches(word))
        .collect::<Vec<_>>();

    for possible_word in possible_words {
        let next = match try_word(board.clone(), &possible_word, &indices) {
            None => continue,
            Some(next) => next,
        };
        find_solutions(next, wordlist);
    }
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Expected 2 command line arguments but got {}", args.len() - 1);
        std::process::exit(1);
    }

    let wordlist: Vec<String> = fs::read_to_string(&Path::new(&args[1]))?
        .lines()
        .map(str::trim)
        .map(String::from)
        .collect();
    let source = ConstraintBoard::from_file(Path::new(&args[2])).expect("Failed to parse board");
    find_solutions(source, &wordlist.iter().map(String::as_str).collect::<Vec<&str>>());
    return Ok(());
}
