// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{collections::VecDeque, iter, str::FromStr};


const GRID_STRIDE: usize = 10;
const FLASH_ENERGY: u8 = 10;

struct Grid {
	cells: [u8; 100]
}

impl Grid {
	/// Returns adjacent cells, if they exist, in clockwise order,
	/// starting at the top; where each existing cell is `(idx, energy)`.
	fn adjacent_cells(&self, idx: usize) -> [Option<(usize, u8)>; 8] {
		let is_top_row = idx < GRID_STRIDE;
		let is_right_col = idx % GRID_STRIDE == GRID_STRIDE - 1;
		let is_bottom_row = idx >= 100 - GRID_STRIDE;
		let is_left_col = idx % GRID_STRIDE == 0;

		macro_rules! c { ( $i:expr ) => { Some(($i, self.cells[$i])) } }
		let mut cells = [None; 8];
		if !is_top_row { cells[0] = c!(idx - GRID_STRIDE); }
		if !is_top_row && !is_right_col { cells[1] = c!(idx - GRID_STRIDE + 1); }
		if !is_right_col { cells[2] = c!(idx + 1); }
		if !is_right_col && !is_bottom_row { cells[3] = c!(idx + GRID_STRIDE + 1); }
		if !is_bottom_row { cells[4] = c!(idx + GRID_STRIDE); }
		if !is_bottom_row && !is_left_col { cells[5] = c!(idx + GRID_STRIDE - 1); }
		if !is_left_col { cells[6] = c!(idx - 1); }
		if !is_left_col && !is_top_row { cells[7] = c!(idx - GRID_STRIDE - 1); }
		cells
	}

	fn existing_adjacent_cells(&self, idx: usize) -> impl Iterator<Item = (usize, u8)> {
		self.adjacent_cells(idx).into_iter().flatten()
	}

	/// Returns whether the octopus in the cell will flash.
	fn incr_cell_energy(&mut self, idx: usize) -> impl Iterator<Item = usize> {
		self.cells[idx] += 1;
		if self.cells[idx] == FLASH_ENERGY {
			itertools::Either::Left(self.existing_adjacent_cells(idx).map(|c| c.0))
		} else {
			itertools::Either::Right(iter::empty())
		}
	}

	/// Returns number of flashes.
	fn tick(&mut self) -> usize {
		let mut queue = VecDeque::from_iter(0..self.cells.len());
		while let Some(idx) = queue.pop_front() {
			queue.extend(self.incr_cell_energy(idx));
		}
		let mut flashed = 0;
		for idx in 0..self.cells.len() {
			if self.cells[idx] >= FLASH_ENERGY {
				flashed += 1;
				self.cells[idx] = 0;
			}
		}
		flashed
	}
}


fn input_grid_from_str(s: &str) -> Grid {
	s.parse::<Grid>().unwrap()
}

fn input_grid() -> Grid {
	input_grid_from_str(include_str!("day11.txt"))
}


fn part1_impl(mut input_grid: Grid) -> usize {
	let mut accum_flashed = 0;
	for _ in 0..100 {
		accum_flashed += input_grid.tick();
	}
	accum_flashed
}

pub(crate) fn part1() -> usize {
	part1_impl(input_grid())
}


fn part2_impl(mut input_grid: Grid) -> usize {
	for i in 0.. {
		if input_grid.tick() == input_grid.cells.len() {
			return i + 1
		}
	}
	unreachable!()
}

pub(crate) fn part2() -> usize {
	part2_impl(input_grid())
}


#[allow(dead_code, clippy::enum_variant_names)]
#[derive(Debug)]
pub(crate) enum ParseGridError {
	InvalidFormat { found_lines: usize },
	InvalidRowFormat { line: usize, found: String },
	InvalidCell  { line: usize, column: usize, found: char },
}

impl FromStr for Grid {
	type Err = ParseGridError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParseGridError::*;

		if s.is_empty() { return Err(InvalidFormat { found_lines: 0 }) }

		let mut cells = [0u8; 100];
		for (l, line) in s.lines().enumerate() {
			if l >= cells.len() / GRID_STRIDE {
				return Err(InvalidFormat { found_lines: l + 1 })
			} else if line.len() != GRID_STRIDE {
				return Err(InvalidRowFormat { line: l + 1, found: line.to_owned() })
			}
			for (c, chr) in line.chars().enumerate() {
				cells[l * GRID_STRIDE + c] = chr.to_digit(10)
					.ok_or(InvalidCell { line: l + 1, column: c + 1, found: chr})? as u8;
			}
		}
		
		Ok(Grid { cells })
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		5483143223
		2745854711
		5264556173
		6141336146
		6357385478
		4167524645
		2176841721
		6882881134
		4846848554
		5283751526
	" };
	assert_eq!(part1_impl(input_grid_from_str(INPUT)), 1656);
	assert_eq!(part1(), 1665);
	assert_eq!(part2_impl(input_grid_from_str(INPUT)), 195);
	assert_eq!(part2(), 235);
}
