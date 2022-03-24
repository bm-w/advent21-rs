// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{collections::{VecDeque, HashSet}, num::ParseIntError, str::FromStr};


#[derive(Debug, Default, PartialEq, Eq, Hash)]
struct Pos {
	x: usize,
	y: usize,
}

#[derive(Debug)]
enum FoldAxis { X, Y }

#[derive(Debug)]
struct FoldInstr(FoldAxis, usize);

#[derive(Debug)]
struct Paper {
	/// The position to the right of and below any dots.
	dots: HashSet<Pos>,
	fold_instrs: VecDeque<FoldInstr>,
}

impl Paper {
	// Returns `true` if there was a fold instruction, or `false` otherwise.
	fn fold_once(&mut self) -> bool {
		macro_rules! fold_once {
			($axis:ident, $axis_pos:expr) => { {
				let mut folded_dots = Vec::new();
				self.dots.retain(|dot| {
					if dot.$axis > $axis_pos {
						folded_dots.push(Pos { $axis: 2 * $axis_pos - dot.$axis, ..*dot  });
						false
					} else {
						true
					}
				});
				self.dots.extend(folded_dots);
				true
			} }
		}
		match self.fold_instrs.pop_front() {
			Some(FoldInstr(FoldAxis::X, fold_x)) => fold_once!(x, fold_x),
			Some(FoldInstr(FoldAxis::Y, fold_y)) => fold_once!(y, fold_y),
			_ => false
		}
	}
}

impl ToString for Paper {
	fn to_string(&self) -> String {
		let Pos { x: width, y: height } = self.dots.iter().fold(Pos::default(), |mut extent, pos| {
			extent.x = extent.x.max(pos.x + 1);
			extent.y = extent.y.max(pos.y + 1);
			extent
		});
		let stride = width + 1;
		let mut grid = vec!('.'; stride * height - 1);
		for y in 1..height {
			grid[y * stride - 1] = '\n'
		}
		for Pos { x, y } in self.dots.iter() {
			grid[y * stride + x] = '#';
		}
		grid.into_iter().collect()
	}
}


fn input_paper_from_str(s: &str) -> Paper {
	s.parse().unwrap()
}

fn input_paper() -> Paper {
	input_paper_from_str(include_str!("day13.txt"))
}


fn part1_impl(mut input_paper: Paper) -> usize {
	input_paper.fold_once();
	input_paper.dots.len()
}

pub(crate) fn part1() -> usize {
	part1_impl(input_paper())
}


fn part2_impl(mut input_paper: Paper) -> String {
	while input_paper.fold_once() {};
	input_paper.to_string()
}

pub(crate) fn part2() -> String {
	part2_impl(input_paper())
}


#[derive(Debug)]
enum ParsePosError {
	InvalidFormat(String),
	InvalidX(ParseIntError),
	InvalidY(ParseIntError),
}

impl FromStr for Pos {
	type Err = ParsePosError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParsePosError::*;
		let (x, y) = s.split_once(',')
			.ok_or_else(|| InvalidFormat(s.to_owned()))?;
		let x = x.parse().map_err(InvalidX)?;
		let y = y.parse().map_err(InvalidY)?;
		Ok(Pos { x, y })
	}
}

#[derive(Debug)]
struct ParseFoldAxisError(String);

impl FromStr for FoldAxis {
	type Err = ParseFoldAxisError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"x" => Ok(Self::X),
			"y" => Ok(Self::Y),
			_ => Err(ParseFoldAxisError(s.to_owned())),
		}
	}
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum ParseFoldInstrError {
	InvalidFormat(String),
	InvalidAxis(ParseFoldAxisError),
	InvalidAmount(ParseIntError),
}

const FOLD_INSTR_PREFIX: &str = "fold along ";

impl FromStr for FoldInstr {
	type Err = ParseFoldInstrError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParseFoldInstrError::*;
		let (axis, pos) = if s.starts_with(FOLD_INSTR_PREFIX)  {
			s.strip_prefix(FOLD_INSTR_PREFIX).and_then(|s| s.split_once('='))
		} else {
			None
		}.ok_or_else(|| InvalidFormat(s.to_owned()))?;
		let axis = axis.parse().map_err(InvalidAxis)?;
		let pos = pos.parse().map_err(InvalidAmount)?;
		Ok(FoldInstr(axis, pos))
	}
}

#[allow(dead_code)]
#[derive(Debug)]
enum ParsePaperError {
	InvalidPos { line: usize, source: ParsePosError},
	InvalidFoldInstr { line: usize, source: ParseFoldInstrError },
}

impl FromStr for Paper {
	type Err = ParsePaperError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParsePaperError::*;
		let mut lines = s.lines().enumerate();
		let dots = lines.by_ref()
			.map_while(|(l, line)| if line.is_empty() { None } else {
				Some(line.parse().map_err(|e| InvalidPos { line: l, source: e }))
			})
			.collect::<Result<HashSet<Pos>, _>>()?;
		let fold_instrs = lines
			.map(|(l, line)| line.parse()
				.map_err(|e| InvalidFoldInstr { line: l, source: e }))
			.collect::<Result<VecDeque<FoldInstr>, _>>()?;
		Ok(Paper { dots, fold_instrs })
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		6,10
		0,14
		9,10
		0,3
		10,4
		4,11
		6,0
		6,12
		4,1
		0,13
		10,12
		3,4
		3,0
		8,4
		1,10
		2,14
		8,10
		9,0

		fold along y=7
		fold along x=5
	" };
	assert_eq!(part1_impl(input_paper_from_str(INPUT)), 17);
	assert_eq!(part1_impl({
		let mut paper = input_paper_from_str(INPUT);
		paper.fold_once();
		paper
	}), 16);
	assert_eq!(part1(), 837);

	assert_eq!(part2_impl(input_paper_from_str(INPUT)), indoc::indoc! { "
		#####
		#...#
		#...#
		#...#
		#####" });
	assert_eq!(part2(), indoc::indoc! { "
		####.###..####..##..#..#..##..#..#.#..#
		#....#..#....#.#..#.#.#..#..#.#..#.#..#
		###..#..#...#..#....##...#....####.#..#
		#....###...#...#.##.#.#..#....#..#.#..#
		#....#....#....#..#.#.#..#..#.#..#.#..#
		####.#....####..###.#..#..##..#..#..##." });
}
