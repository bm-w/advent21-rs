// Copyright (c) 2022 Bastiaan Marinus van de Weerd


#[derive(Clone, Copy)]
enum SeaCucumber { East, South }

struct Grid {
	spaces: Vec<Option<SeaCucumber>>,
	width: usize,
}


mod simulation {
	use super::{SeaCucumber, Grid};

	impl Grid {
		fn tick_east(&mut self) -> bool {
			let mut any_moved = false;
			for y in 0..self.spaces.len() / self.width {
				let iy = y * self.width;
				let first_none = self.spaces[iy].is_none();
				let mut just_moved_x = self.width;
				for (x0, x1) in (0..self.width - 1).zip(1..self.width) {
					if x0 == just_moved_x { continue }
					let (i0, i1) = (iy + x0 , iy + x1);
					if let (Some(SeaCucumber::East), None) = (self.spaces[i0], self.spaces[i1]) {
						// println!("Moving East: {i0}:({x0},{y}) -> {i1}:({x1},{y})");
						self.spaces.swap(i0, i1);
						just_moved_x = x1;
						any_moved = true;
					}
				}
				let last_x = self.width - 1;
				if first_none && just_moved_x != last_x {
					let last_i = iy + last_x;
					if let Some(SeaCucumber::East) = self.spaces[last_i] {
						// println!("Moving East: {last_i}:({last_x},{y}) -> {iy}:(0,{y}) (wrapped)");
						self.spaces.swap(last_i, iy);
						any_moved = true
					}
				}
			}
			any_moved
		}

		fn tick_south(&mut self) -> bool {
			let mut any_moved = false;
			let height = self.spaces.len() / self.width;
			for x in 0..self.width {
				let first_none = self.spaces[x].is_none();
				let mut just_moved_y = height;
				for (y0, y1) in (0..height - 1).zip(1..height) {
					if y0 == just_moved_y { continue }
					let (i0, i1) = (self.width * y0 + x , self.width * y1 + x);
					if let (Some(SeaCucumber::South), None) = (self.spaces[i0], self.spaces[i1]) {
						// println!("Moving South: {i0}:({x},{y0}) -> {i1}:({x},{y1})");
						self.spaces.swap(i0, i1);
						just_moved_y = y1;
						any_moved = true;
					}
				}
				let last_y = height - 1;
				if first_none && just_moved_y != last_y {
					let last_i = last_y * self.width + x;
					if let Some(SeaCucumber::South) = self.spaces[last_i] {
						// println!("Moving South: {last_i}:({x},{last_y}) -> {x}:({x},0) (wrapped)");
						self.spaces.swap(last_i, x);
						any_moved = true
					}
				}
			}
			any_moved
		}

		fn tick(&mut self) -> bool {
			let any_moved_east = self.tick_east();
			let any_moved_south = self.tick_south();
			any_moved_east || any_moved_south
		}

		pub(super) fn tick_until_stuck(&mut self) -> usize {
			for i in 0.. {
				if !self.tick() { return i + 1 }
			}
			unreachable!()
		}
	}


	#[cfg(test)]
	use std::fmt::{Display, Write};

	#[cfg(test)]
	impl Display for Grid {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			for (i, space) in self.spaces.iter().enumerate() {
				f.write_char(match space {
					None => '.',
					Some(super::SeaCucumber::East) => '>',
					Some(super::SeaCucumber::South) => 'v',
				})?;
				if i % self.width == self.width - 1 {
					f.write_char('\n')?;
				}
			}
			Ok(())
		}
	}
}


fn input_grid_from_str(s: &str) -> Grid {
	s.parse().unwrap()
}


fn part1_impl(mut input_grid: Grid) -> usize {
	input_grid.tick_until_stuck()
}

pub(crate) fn part1() -> usize {
	part1_impl(input_grid_from_str(include_str!("day25.txt")))
}


pub(crate) fn part2() -> &'static str {
	"Merry Christmas!"
}


mod parsing {
	use std::str::FromStr;
	use super::{SeaCucumber, Grid};

	#[derive(Debug)]
	pub(super) struct InvalidSeaCucumberError(Option<char>);

	impl TryFrom<char> for SeaCucumber {
		type Error = InvalidSeaCucumberError;
		fn try_from(value: char) -> Result<Self, Self::Error> {
			match value {
				'>' => Ok(SeaCucumber::East),
				'v' => Ok(SeaCucumber::South),
				found => Err(InvalidSeaCucumberError(Some(found))),
			}
		}
	}

	#[allow(dead_code, clippy::enum_variant_names)]
	#[derive(Debug)]
	pub(super) enum GridError {
		InvalidFormat { line: usize, column: usize, found: Option<char> },
		InvalidSeaCucumber { line: usize, column: usize, source: InvalidSeaCucumberError },
	}

	impl FromStr for Grid {
		type Err = GridError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use {std::iter::once, itertools::Either, GridError::*};

			let mut lines = s.lines();

			let (width, lines) = {
				let line = lines.next()
					.and_then(|line| if line.is_empty() { None } else { Some(line) })
					.ok_or(InvalidFormat { line: 1, column: 1, found: None })?;
				let width = line.len();
				let lines = once(line).chain(lines);
				(width, lines)
			};

			let spaces = lines
				.enumerate()
				.flat_map(|(l, line)| if line.len() != width {
					Either::Left(once(Err(InvalidFormat { line: l + 1, column: line.len().max(width) + 1, found: None })))
				} else {
					Either::Right(line.chars()
						.enumerate()
						.map(move |(c, chr)|
							if chr == '.' { Ok(None) }
							else { chr.try_into().map_or_else(
								|e| Err(InvalidSeaCucumber { line: l + 1, column: c + 1, source: e }),
								|sc| Ok(Some(sc))) }))
				})
				.collect::<Result<Vec<_>, _>>()?;

			Ok(Grid { spaces, width })
		}
	}


	#[test]
	fn sea_cucumber() {
		SeaCucumber::try_from('>').unwrap();
		SeaCucumber::try_from('v').unwrap();
		SeaCucumber::try_from('x').map(|_| ()).unwrap_err();
	}

	#[test] 
	fn grid() {
		super::TEST_INPUT.parse::<Grid>().unwrap();
	}
}


#[cfg(test)]
const TEST_INPUT: &str = indoc::indoc! { "
	v...>>.vv>
	.vv>>.vv..
	>>.>v>...v
	>>v>>.>.v.
	v>v.vv.v..
	>.>>..v...
	.vv..>.>v.
	v.v..>>v.v
	....v..v.>
" };

#[test]
fn tests() {
	assert_eq!(part1_impl(input_grid_from_str(TEST_INPUT)), 58);
	assert_eq!(part1(), 419);
}
