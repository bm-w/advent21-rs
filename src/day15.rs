// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{str::FromStr, iter, collections::BinaryHeap, cmp::Ordering};


const EXT_LEN: usize = 5;

struct Grid {
	cell_risks: Vec<u8>,
	width: usize,
}

impl Grid {
	fn cells_len(&self, extended: bool) -> usize {
		let unext_cells_len = self.cell_risks.len();
		if !extended { unext_cells_len }
		else { unext_cells_len * EXT_LEN * EXT_LEN }
	}

	/// Returns the “unextended” index into `self.cells`, along with the non-
	/// modulo’d additional risk that applies for `idx`’s part of the extended grid.
	fn unextended_index(&self, idx: usize, extended: bool) -> (usize, u8) {
		if !extended {
			(idx, 0)
		} else {
			let unext_height = self.cell_risks.len() / self.width;
			let ext_width = EXT_LEN * self.width;
			let ext_x = idx % ext_width;
			let ext_y = (idx - ext_x) / ext_width;
			(
				(ext_y % unext_height) * self.width + (ext_x % self.width),
				(ext_x / self.width + ext_y / unext_height) as u8
			)
		}
	}

	/// Returns adjacent cells, in order top,
	/// right, bottom, left, as `(idx, risk)`.
	fn adjacent_cells(&self, idx: usize, extended: bool) -> [Option<(usize, u8)>; 4] {

		let width = self.width;
		let height = self.cell_risks.len() / width;
		let (width, height) =
			if !extended { (width, height) }
			else { (EXT_LEN * width, EXT_LEN * height) };

		let mut cells = [None; 4];
		macro_rules! c { ($k: expr, $idx:expr) => {
			let (true_idx, add_risk) = self.unextended_index($idx, extended);
			cells[$k] = Some(($idx, 1 + (self.cell_risks[true_idx] + add_risk - 1) % 9));
		} }
		if idx > width { c!(0, idx - width); }
		if idx % width < width - 1 { c!(1, idx + 1); }
		if idx < width * height - width { c!(2, idx + width); }
		if idx % width > 0 { c!(3, idx - 1); }
		cells
	}

	fn existing_adjacent_cells(&self, idx: usize, extended: bool) -> impl Iterator<Item = (usize, u8)> {
		self.adjacent_cells(idx, extended).into_iter().filter_map(|c| c)
	}
}


fn input_grid_from_str(s: &str) -> Grid {
	s.parse().unwrap()
}

fn input_grid() -> Grid {
	input_grid_from_str(include_str!("day15.txt"))
}


fn part1_impl(input_grid: Grid, extended: bool) -> u64 {
	// Dijkstra

	#[derive(Debug, PartialEq, Eq)]
	struct State { idx: usize, total_risk: u64 }

	impl Ord for State {
		fn cmp(&self, other: &Self) -> Ordering {
			other.total_risk.cmp(&self.total_risk).then(self.idx.cmp(&other.idx))
		}
	}

	impl PartialOrd for State {
		fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
			Some(self.cmp(&other))
		}
	}

	let cells_len = input_grid.cells_len(extended);
	let mut total_risks = (0..cells_len).map(|_| u64::MAX).collect::<Vec<_>>();
	let mut heap = BinaryHeap::new();

	total_risks[0] = 0;
	heap.push(State { idx: 0, total_risk: 0 });

	while let Some(State { idx, total_risk }) = heap.pop() {
		if idx == cells_len - 1 { return total_risk }

		if total_risk > total_risks[idx] { continue }

		for (adj_idx, adj_risk) in input_grid.existing_adjacent_cells(idx, extended) {
			let adj_total_risk = total_risk + adj_risk as u64;

			if adj_total_risk < total_risks[adj_idx] {
				total_risks[adj_idx] = adj_total_risk;
				heap.push(State { idx: adj_idx, total_risk: adj_total_risk })
			}
		}
	}

	unreachable!()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_grid(), false)
}

pub(crate) fn part2() -> u64 {
	part1_impl(input_grid(), true)
}


#[allow(dead_code)]
#[derive(Debug)]
enum ParseGridError {
	InvalidFormat { line: usize },
	InvalidWidth { line: usize, found: usize},
	InvalidCell { line: usize, column: usize, found: char },
}

impl FromStr for Grid {
	type Err = ParseGridError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParseGridError::*;

		let mut width = 0;
		let mut height = 0;
		let mut cell_risks = Vec::new();
		enum El { CellRisk(u8), Width }
		for res in s.lines()
			.enumerate()
			.flat_map(|(l, line)|
				line.chars()
					.enumerate()
					.map(move |(c, chr)| chr.to_digit(10)
						.map(|cr| El::CellRisk(cr as u8))
						.ok_or(InvalidCell { line: l + 1, column: c + 1, found: chr }))
					.chain(iter::once(Ok(El::Width))))
		{
			let crl = cell_risks.len();
			match res? {
				El::CellRisk(cell_risk) => {
					if width > 0 && crl > width * width {
						return Err(InvalidFormat { line: height + 1 })
					}
					cell_risks.push(cell_risk);
				}
				El::Width if width == 0 => {
					width = cell_risks.len();
					height += 1;
				}
				El::Width if crl % width != 0 => {
					let line = height + 1;
					let found = crl - width * height;
					return Err(InvalidWidth { line, found })
				}
				El::Width => cell_risks.reserve(width * width - cell_risks.len()),
			}
		}

		if height == 0 {
			return Err(InvalidFormat { line: 1 })
		}

		Ok(Grid { cell_risks, width })
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		1163751742
		1381373672
		2136511328
		3694931569
		7463417111
		1319128137
		1359912421
		3125421639
		1293138521
		2311944581
	" };
	assert_eq!(part1_impl(input_grid_from_str(INPUT), false), 40);
	assert_eq!(part1(), 388);
	assert_eq!(part1_impl(input_grid_from_str(INPUT), true), 315);
	assert_eq!(part2(), 2819);
}
