// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{str::FromStr, collections::{HashSet, VecDeque}};


pub(crate) struct Grid {
	cells: Vec<u8>,
	stride: usize,
}

impl Grid {
	/// Returns adjacent cells, if they exist, in the order top,
	/// right, bottom, left; where each existing cell is `(idx, height)`.
	fn adjacent_cells(&self, idx: usize) -> [Option<(usize, u8)>; 4] {
		let mut pts = [None; 4];
		if idx >= self.stride { pts[0] = Some((idx - self.stride, self.cells[idx - self.stride])); }
		if idx % self.stride < self.stride - 1 { pts[1] = Some((idx + 1, self.cells[idx + 1])); }
		if idx < self.cells.len() - self.stride { pts[2] = Some((idx + self.stride, self.cells[idx + self.stride])); }
		if idx % self.stride > 0 { pts[3] = Some((idx - 1, self.cells[idx - 1])); }
		pts
	}

	fn existing_adjacent_cells(&self, idx: usize) -> impl Iterator<Item = (usize, u8)> {
		self.adjacent_cells(idx).into_iter().filter_map(|c| c) 
	}

	/// Returns the cell’s height if it’s a low point.
	fn low_point(&self, idx: usize) -> Option<u8> {
		let height = self.cells[idx];
		if self.adjacent_cells(idx).into_iter()
			.all(|c| c.map_or(true, |(_, h)| h > height))
		{
			Some(height)
		} else {
			None
		}
	}
	
	fn low_points(&self) -> impl Iterator<Item = (usize, u8)> + '_ {
		let mut idx = 0;
		std::iter::from_fn(move || {
			loop {
				if idx == self.cells.len() { return None }
				let low_pt = self.low_point(idx);
				idx += 1;
				if low_pt.is_some() { return low_pt.map(|c| (idx - 1, c)) }
			}
		})
	}

	/// Returns iterator where each element is `(idx, size)`,
	/// where `idx` is the cell index of the basin’s lowest point.
	fn basins(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
		self.low_points().map(|(low_pt_idx, _)| {
			let mut basin_idxs = HashSet::new();
			let mut idxs_to_process = VecDeque::from([low_pt_idx]);
			while let Some(cell_idx) = idxs_to_process.pop_front() {
				if basin_idxs.contains(&cell_idx) { continue }
				basin_idxs.insert(cell_idx);
				for (adj_idx, adj_h) in self.existing_adjacent_cells(cell_idx) {
					if adj_h == 9 { continue }
					idxs_to_process.push_back(adj_idx)
				}
			}
			(low_pt_idx, basin_idxs.len())
		})
	}
}


fn input_grid_from_str(s: &str) -> Grid {
	s.parse().unwrap()
}

fn input_grid() -> Grid {
	input_grid_from_str(include_str!("day09.txt"))
}


fn part1_impl(grid: Grid) -> u64 {
	grid.low_points().map(|(_, h)| h as u64 + 1).sum()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_grid())
}


fn part2_impl(grid: Grid) -> usize {
	let mut basins = grid.basins().collect::<Vec<_>>();
	basins.sort_by(|l, r| l.1.cmp(&r.1));
	basins[basins.len() - 3..].into_iter().map(|(_, s)| s).product()
}

pub(crate) fn part2() -> usize {
	part2_impl(input_grid())
}


#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum ParseGridError {
	Empty,
	InvalidLineLen { stride: usize, line: usize, line_len: usize },
	InvalidCell { line: usize, column: usize, found: char },
}

impl FromStr for Grid {
	type Err = ParseGridError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParseGridError::*;

		let mut stride = 0;
		let mut cells = Vec::new();
		for (i, line) in s.lines().enumerate() {
			match (stride, line.len()) {
				(0, ll) => {
					stride = ll;
					cells.reserve(stride * stride);
				}
				(s, ll) if s == ll => {
					let add_cap = (i * s).saturating_sub(cells.capacity());
					if add_cap > 0 { cells.reserve(add_cap) }
				}
				(prev_stride, line_len) => {
					return Err(InvalidLineLen { stride: prev_stride, line: i, line_len });
				}
			};

			for (j, c) in line.chars().enumerate() {
				match c.to_digit(10) {
					Some(cell) => { cells.push(cell as u8); }
					None => { return Err(InvalidCell { line: i, column: j, found: c }); }
				}
			}
		}

		Ok(Grid { cells, stride })
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		2199943210
		3987894921
		9856789892
		8767896789
		9899965678
	" };
	assert_eq!(part1_impl(input_grid_from_str(INPUT)), 15);
	assert_eq!(part2_impl(input_grid_from_str(INPUT)), 1134);
}
