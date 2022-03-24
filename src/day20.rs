// Copyright (c) 2022 Bastiaan Marinus van de Weerd


const ALGORITHM_BITS_LEN: usize = 512;

struct EnhancementAlgorithm(Vec<bool>); // TODO(bm-w): Use more efficient storage?

use std::fmt::{Display, Write};

struct Image {
	bits: Vec<bool>,
	stride: usize,
}

struct Map {
	enhancement_algorithm: EnhancementAlgorithm,
	input_image: Image,
	step: usize,
}


impl Image {
	fn clone_expanded(&self, expanded_lit: bool) -> Image {
		let expanded_stride = self.stride + 2;
		let extra_capacity = 2 * self.stride + 2 * (self.bits.len() / self.stride + 2);
		let mut expanded_bits = vec![expanded_lit; self.bits.len() + extra_capacity];
		for (i, &b) in self.bits.iter().enumerate() {
			let (x, y) = (i % self.stride, i / self.stride);
			let expanded_i = (y + 1) * expanded_stride + x + 1;
			expanded_bits[expanded_i] = b;
		}
		Image { bits: expanded_bits, stride: expanded_stride }
	}

	fn enhancement_algorithm_index(&self, at: usize, expanded_lit: bool) -> usize { // Row-major
		let s = self.stride;
		let top = at < s;
		let left = at % s == 0;
		let right = at % s == s - 1;
		let bottom = at >= self.bits.len() - s;
		let b256 = if if top || left { expanded_lit } else { self.bits[at - s - 1] } { 256 } else { 0 };
		let b128 = if if top { expanded_lit } else { self.bits[at - s] } { 128 } else { 0 };
		let b64 = if if top || right { expanded_lit } else { self.bits[at - s + 1] } { 64 } else { 0 };
		let b32 = if if left { expanded_lit } else { self.bits[at - 1] } { 32 } else { 0 };
		let b16 = if self.bits[at] { 16 } else { 0 };
		let b8 = if if right { expanded_lit } else { self.bits[at + 1] } { 8 } else { 0 };
		let b4 = if if bottom || left { expanded_lit } else { self.bits[at + s - 1] } { 4 } else { 0 };
		let b2 = if if bottom { expanded_lit } else { self.bits[at + s] } { 2 } else { 0 };
		let b1 = if if bottom || right { expanded_lit } else { self.bits[at + s + 1] } { 1 } else { 0 };
		b256 + b128 + b64 + b32 + b16 + b8 + b4 + b2 + b1
	}

	fn count_lit_bits(&self) -> usize {
		self.bits.iter().filter(|&&b| b).count()
	}
}

impl Display for Image {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (i, &bit) in self.bits.iter().enumerate() {
			f.write_char(if bit { '#' } else { '.' })?;
			if i % self.stride == self.stride - 1 { f.write_char('\n')?; }
		}
		Ok(())
	}
}

impl Map {
	fn enhanced(self) -> Map {
		let expanded_lit = match (
			self.enhancement_algorithm.0[0],
			self.enhancement_algorithm.0[ALGORITHM_BITS_LEN - 1],
		) {
			(false, _) => false, // Dark outer pixels never light up
			(true, true) => self.step > 0, // Dark outer pixels light up on first step (but are not lit yet) and stay lit
			(true, false) => self.step % 2 == 1, // Outer pixels light up on first step, turn dark again on the second, ad infinitum
		};
		let mut expanded_image = self.input_image.clone_expanded(expanded_lit);
		expanded_image.bits = (0..expanded_image.bits.len())
				.map(|i| expanded_image.enhancement_algorithm_index(i, expanded_lit))
				.map(|k| self.enhancement_algorithm.0[k])
				.collect();
		Map { input_image: expanded_image, step: self.step + 1, ..self }
	}
}


mod parsing {
	use std::{iter, str::FromStr};
	use super::{EnhancementAlgorithm, Image, Map};


	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) struct BitsError {
		column: usize,
		found: char,
	}

	fn bits_from_str(s: &str) -> impl Iterator<Item = Result<bool, BitsError>> + '_ {
		s.chars().enumerate().map(|(c, chr)| match chr {
			'#' => Ok(true),
			'.' => Ok(false),
			b => Err(BitsError { column: c + 1, found: b })
		})
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum EnhancementAlgorithmError {
		InvalidFormat { column: usize },
		InvalidChar { column: usize, found: char },
	}

	impl FromStr for EnhancementAlgorithm {
		type Err = EnhancementAlgorithmError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use EnhancementAlgorithmError::*;
			let mut bits_iter = bits_from_str(s);
			let bits = bits_iter.by_ref()
				.take(super::ALGORITHM_BITS_LEN)
				.collect::<Result<Vec<_>, _>>()
				.map_err(|e| InvalidChar { column: e.column, found: e.found })?; 
			if bits.len() < 512 { Err(InvalidFormat { column: bits.len() + 1 }) }
			else if bits_iter.next().is_some() { Err(InvalidFormat { column: super::ALGORITHM_BITS_LEN + 1 }) }
			else { Ok(EnhancementAlgorithm(bits)) }
		}
	}
	
	#[allow(dead_code, clippy::enum_variant_names)]
	#[derive(Debug)]
	pub(super) enum ImageError {
		InvalidFormat,
		InvalidLineFormat { line: usize, column: usize },
		InvalidChar { line: usize, column: usize, found: char },
	}

	impl<'a, I> TryFrom<(usize, I)> for Image 
	where I: Iterator<Item = &'a str> {
		type Error = ImageError;
		fn try_from((line_offset, lines): (usize, I)) -> Result<Self, Self::Error> {
			use ImageError::*;
			let mut stride = 0;
			let bits = lines.enumerate()
				.flat_map(|(l, line)| {
					let line_len = line.len();
					if stride == 0 {
						stride = line_len;
					} else if line_len != stride {
						return itertools::Either::Left(iter::once(Err(
							InvalidLineFormat { line: line_offset + l, column: line_len + 1 }
						)))
					}
					itertools::Either::Right(bits_from_str(line).map(move |r|
						r.map_err(|e| InvalidChar { line: line_offset + l, column: e.column, found: e.found })))
				})
				.collect::<Result<Vec<_>, _>>()?;
			Ok(Image { bits, stride })
		}
	}

	#[allow(dead_code, clippy::enum_variant_names)]
	#[derive(Debug)]
	pub(super) enum MapError {
		InvalidFormat { line: usize },
		InvalidEnhancementAlgorithm(EnhancementAlgorithmError),
		InvalidImage(ImageError),
	}

	impl FromStr for Map {
		type Err = MapError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use MapError::*;
			let mut lines = s.lines();

			let enhancement_algorithm = lines.next()
				.map(|line| line.parse()
					.map_err(InvalidEnhancementAlgorithm))
				.unwrap_or(Err(InvalidFormat { line: 1 }))?;

			if lines.next() != Some("") { return Err(InvalidFormat { line: 2 }) }

			let input_image = (3, lines).try_into()
				.map_err(InvalidImage)?;

			Ok(Map { enhancement_algorithm, input_image, step: 0 })
		}
	}


	#[test]
	fn test() {
		assert!(super::TEST_INPUT.parse::<Map>().is_ok());
	}
}


fn input_map_from_str(s: &str) -> Map {
	s.parse().unwrap()
}

fn input_map() -> Map {
	input_map_from_str(include_str!("day20.txt"))
}


fn part1_impl(input_map: Map) -> usize {
	input_map.enhanced().enhanced().input_image.count_lit_bits()
}

pub(crate) fn part1() -> usize {
	part1_impl(input_map())
}


fn part2_impl(mut input_map: Map) -> usize {
	for _ in 0..50 {
		input_map = input_map.enhanced();
	}
	input_map.input_image.count_lit_bits()
}

pub(crate) fn part2() -> usize {
	part2_impl(input_map())
}


#[cfg(test)]
const TEST_INPUT: &str = indoc::indoc! { "
	..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

	...............
	...............
	...............
	...............
	...............
	.....#..#......
	.....#.........
	.....##..#.....
	.......#.......
	.......###.....
	...............
	...............
	...............
	...............
	...............
" };

#[test]
fn tests() {
	assert_eq!(part1_impl(input_map_from_str(TEST_INPUT)), 35);
	assert_eq!(part1(), 5379);
	assert_eq!(part2_impl(input_map_from_str(TEST_INPUT)), 3351);
	assert_eq!(part2(), 17917);
}
