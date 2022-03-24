// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{str::FromStr, num::ParseIntError};


#[derive(Debug)]
pub(crate) struct Game {
	nums: Vec<u8>,
	boards: Vec<[u8; 25]>
}


fn input_game_from_str(s: &str) -> Game {
	Game::from_str(s).unwrap()
}

fn input_game() -> Game {
	input_game_from_str(include_str!("day04.txt"))
}


fn part1_impl(game: Game) -> u64 {

	// Per board, no. of cells marked per row (first 5) & per column (last 5)
	let mut boards_marked = (0..game.boards.len()).map(|_| [0u8; 10]).collect::<Vec<_>>();
	let mut board_rem_sums = game.boards.iter()
		.map(|board| board.iter().map(|c| *c as u64).sum::<u64>())
		.collect::<Vec<_>>();

	for num in game.nums.iter() {
		for (k, board) in game.boards.iter().enumerate() {
			if let Some(idx) = board.iter().position(|c| c == num) {
				let num = *num as u64;
				board_rem_sums[k] -= num;
				let (i, j) = (idx / 5, idx % 5);

				boards_marked[k][i] += 1;
				boards_marked[k][5 + j] += 1;

				if boards_marked[k][i] == 5 || boards_marked[k][5 + j] == 5 {
					return num * board_rem_sums[k];
				}
			}
		}
	}
	unreachable!()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_game())
}


fn part2_impl(game: Game) -> u64 {

	// Per board, no. of cells marked per row (first 5) & per column (last 5)
	let mut boards_marked = (0..game.boards.len()).map(|_| [0u8; 10]).collect::<Vec<_>>();
	let mut board_rem_sums = game.boards.iter()
		.map(|board| board.iter().map(|c| *c as u64).sum::<u64>())
		.collect::<Vec<_>>();
	let mut board_scores = (0..game.boards.len()).map(|_| 0u64).collect::<Vec<_>>();

	for num in game.nums.iter() {
		for (k, board) in game.boards.iter().enumerate() {
			if board_scores[k] > 0 { continue }
			if let Some(idx) = board.iter().position(|c| c == num) {
				let num = *num as u64;
				board_rem_sums[k] -= num;
				let (i, j) = (idx / 5, idx % 5);

				boards_marked[k][i] += 1;
				boards_marked[k][5 + j] += 1;

				if boards_marked[k][i] == 5 || boards_marked[k][5 + j] == 5 {
					board_scores[k] = num * board_rem_sums[k];

					if !board_scores.iter().any(|score| *score == 0) {
						return board_scores[k];
					}
				}
			}
		}
	}
	unreachable!()
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_game())
}


#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum ParseGameError {
	Empty,
	InvalidNum { i: usize, e: ParseIntError },
	InvalidBoardSeparator { k: usize, found: String },
	IncompleteBoard { k: usize, i: usize },
	InvalidBoardNum { e: ParseIntError, k: usize, i: usize, j: usize },
	IncompleteBoardRow { k: usize, i: usize, j: usize },
	MissingBoards,
}

impl FromStr for Game {
	type Err = ParseGameError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lines = s.lines();

		let nums = lines.next()
			.ok_or(ParseGameError::Empty)?
			.split(',')
			.enumerate()
			.map(|(i, num)| u8::from_str(num)
				.map_err(|e| ParseGameError::InvalidNum { i, e }))
			.collect::<Result<Vec<_>, _>>()?;

		let mut boards = Vec::new();
		while let Some(blank_line) = lines.next() {
			let k = boards.len();

			if !blank_line.is_empty() {
				return Err(ParseGameError::InvalidBoardSeparator { k, found: String::from(blank_line) });
			}

			let mut board = [0u8; 25];
			for i in 0..5 {
				let mut j = 0;
				for num in lines.next()
					.ok_or(ParseGameError::IncompleteBoard { k, i })?
					.split(' ')
					.filter(|num| !num.is_empty())
				{
					board[5 * i + j] = u8::from_str(num).map_err(|e| ParseGameError::InvalidBoardNum { k, i , j, e })?;
					j += 1;
				}
				if j < 5 {
					return Err(ParseGameError::IncompleteBoardRow { k, i, j });
				}
			}
			boards.push(board);
		}
		if boards.is_empty() {
			return Err(ParseGameError::MissingBoards);
		}

		Ok(Game { nums, boards })
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

		22 13 17 11  0
		8  2 23  4 24
		21  9 14 16  7
		6 10  3 18  5
		1 12 20 15 19
		
		3 15  0  2 22
		9 18 13 17  5
		19  8  7 25 23
		20 11 10 24  4
		14 21 16 12  6
		
		14 21 17 24  4
		10 16 15  9 19
		18  8 23 26 20
		22 11 13  6  5
		2  0 12  3  7
	" };
	assert_eq!(part1_impl(input_game_from_str(INPUT)), 4512);
	assert_eq!(part2_impl(input_game_from_str(INPUT)), 1924);
}
