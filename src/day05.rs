// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{str::FromStr, num::ParseIntError, collections::HashMap};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Pos {
	x: u16,
	y: u16,
}

#[derive(Debug)]
pub(crate) struct Line {
	start: Pos,
	end: Pos,
}

impl Line {
	// TODO(bm-w): Implement custom iterator instead of `Box<dyn …>`?
	fn positions(&self, incl_diag: bool) -> Box<dyn Iterator<Item = Pos>> {
		match (self.start.x == self.end.x, self.start.y == self.end.y) {
			(true, _) => {
				let x = self.start.x;
				let r = if self.start.y >= self.end.y { self.end.y..=self.start.y }
					else { self.start.y..=self.end.y };
				Box::new(r.map(move |y| Pos { x, y }))
			}
			(_, true) => {
				let y = self.start.y;
				let r = if self.start.x >= self.end.x { self.end.x..=self.start.x }
					else { self.start.x..=self.end.x };
				Box::new(r.map(move |x| Pos { x, y }))
			}
			_ if incl_diag => {
				use itertools::Either;
				let rx = if self.start.x >= self.end.x {
					Either::Left((self.end.x..=self.start.x).rev())
				} else {
					Either::Right(self.start.x..=self.end.x)
				};
				let ry = if self.start.y >= self.end.y {
					Either::Left((self.end.y..=self.start.y).rev())
				} else {
					Either::Right(self.start.y..=self.end.y)
				};
				Box::new(std::iter::zip(rx, ry).map(|(x, y)| Pos { x, y }))
			},
			_ => Box::new(std::iter::empty()),
		}
	}
}


fn input_lines_from_str(s: &str) -> impl Iterator<Item = Line> + '_ {
	s.lines().map(|l| Line::from_str(l).unwrap())
}

fn input_lines() -> impl Iterator<Item = Line> {
	input_lines_from_str(include_str!("day05.txt"))
}


fn part1and2_impl(input_lines: impl Iterator<Item = Line>, incl_diag: bool) -> usize {
	let mut grid = HashMap::new();
	for line in input_lines {
		for pos in line.positions(incl_diag) {
			grid.entry(pos).and_modify(|v| *v += 1).or_insert(1);
		}
	}

	grid.values().filter(|v| **v >= 2).count()
}

pub(crate) fn part1() -> usize {
	part1and2_impl(input_lines(), false)
}

pub(crate) fn part2() -> usize {
	part1and2_impl(input_lines(), true)
}


#[derive(Debug)]
pub(crate) enum ParsePosError {
	InvalidFormat(String),
	InvalidX(ParseIntError),
	InvalidY(ParseIntError)
}

impl FromStr for Pos {
	type Err = ParsePosError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (x, y) = s.trim().split_once(",")
			.ok_or(ParsePosError::InvalidFormat(s.to_owned()))?;
		let x = u16::from_str(x)
			.map_err(|e| ParsePosError::InvalidX(e))?;
		let y = u16::from_str(y)
			.map_err(|e| ParsePosError::InvalidY(e))?;
		Ok(Pos { x, y })
	}
}

#[derive(Debug)]
pub(crate) enum ParseLineError {
	InvalidFormat(String),
	InvalidStart(ParsePosError),
	InvalidEnd(ParsePosError)
}

impl FromStr for Line {
	type Err = ParseLineError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (start, end) = s.split_once("->")
			.ok_or(ParseLineError::InvalidFormat(s.to_owned()))?;
		let start = Pos::from_str(start)
			.map_err(|e| ParseLineError::InvalidStart(e))?;
		let end = Pos::from_str(end)
			.map_err(|e| ParseLineError::InvalidEnd(e))?;
		Ok(Line { start, end })
	}
}


#[test]
fn tests() {
	const INPUT_LINES: &str = indoc::indoc! { "
		0,9 -> 5,9
		8,0 -> 0,8
		9,4 -> 3,4
		2,2 -> 2,1
		7,0 -> 7,4
		6,4 -> 2,0
		0,9 -> 2,9
		3,4 -> 1,4
		0,0 -> 8,8
		5,5 -> 8,2
	" };
	assert_eq!(part1and2_impl(input_lines_from_str(INPUT_LINES), false), 5);
	assert_eq!(part1and2_impl(input_lines_from_str(INPUT_LINES), true), 12);
}
