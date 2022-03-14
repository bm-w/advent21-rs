// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{ops::{Add, AddAssign}, iter::Sum};


#[derive(Debug, Clone)]
enum InnerNumber {
	Regular(u64),
	Pair(Box<Number>), // TODO(bm-w): References?
}

#[derive(Debug, Clone)]
struct Number(InnerNumber, InnerNumber);


#[cfg(test)]
mod formatting {
	use std::fmt::Display;
	use super::{InnerNumber, Number};

	impl Display for InnerNumber {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			use InnerNumber::*;
			match self {
				Regular(r) => write!(f, "{r}"),
				Pair(n) => write!(f, "{n}"),
			}
		}
	}

	impl Display for Number {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "[{},{}]", self.0, self.1)
		}
	}


	#[test]
	fn test() {
		use InnerNumber::*;
		assert_eq!(Number(Regular(1), Regular(2)).to_string(), "[1,2]");
		assert_eq!(Number(Regular(3), Pair(Box::new(Number(Regular(4), Regular(5))))).to_string(), "[3,[4,5]]");
		assert_eq!(Number(Pair(Box::new(Number(Regular(6), Regular(7)))), Regular(8)).to_string(), "[[6,7],8]");
	}
}


mod reducing {
	use {super::InnerNumber::{self, *}, super::Number};

	enum _Exploded { False, True(Option<u64>, Option<u64>) }

	impl _Exploded {
		fn is_true(&self) -> bool {
			matches!(self, _Exploded::True(_, _))
		}
	}

	enum _AddExplodedSide { Left, Right }

	impl _AddExplodedSide {
		fn is_left(&self) -> bool {
			matches!(self, _AddExplodedSide::Left)
		}
	}

	impl InnerNumber {
		fn _add_exploded(&mut self, value: u64, side: _AddExplodedSide) {
			match self {
				Regular(regular) => *regular += value,
				Pair(number) => number._add_exploded(value, side),
			}
		}

		fn _explode(&mut self, depth: usize) -> _Exploded {
			match self {
				Regular(_) => _Exploded::False,
				Pair(number) => match number._explode(depth + 1) {
					t @ _Exploded::True(Some(_), Some(_)) => {
						*self = Regular(0);
						t
					}
					e => e
				}
			}
		}

		fn _split(&mut self) -> bool {
			match self {
				Regular(ref val) if *val > 9 => {
					let l = *val / 2;
					let r = l + *val % 2;
					let number = Number(Regular(l), Regular(r));
					*self = Pair(Box::new(number));
					true
				}
				Pair(number) => {
					number._split()
				}
				_ => false
			}
		}
	}

	impl Number {
		fn _add_exploded(&mut self, value: u64, side: _AddExplodedSide) {
			if side.is_left() { &mut self.0 } else { &mut self.1 }
				._add_exploded(value, side)
		}

		fn _explode(&mut self, depth: usize) -> _Exploded {
			if depth >= 4 {
				match (&self.0, &self.1) {
					(Regular(l), Regular(r)) => _Exploded::True(Some(*l), Some(*r)),
					_ => unreachable!(),
				}
			} else {
				use _Exploded::*;
				match self.0._explode(depth) {
					True(l, Some(r)) => {
						self.1._add_exploded(r, _AddExplodedSide::Left);
						True(l, None)
					}
					t @ True(_, None) => t,
					False => match self.1._explode(depth) {
						True(Some(l), r) => {
							self.0._add_exploded(l, _AddExplodedSide::Right);
							True(None, r)
						}
						e => e
					}
				}
			}
		}

		fn _split(&mut self) -> bool {
			self.0._split() || self.1._split()
		}

		pub(super) fn reduce(&mut self) {
			loop {
				if self._explode(0).is_true() { continue }
				if self._split() { continue }
				break
			}
		}
	}


	#[test]
	fn test() -> Result<(), super::parsing::ParseNumberError>{
		const REDUCED: fn(Number) -> Number = |mut n: Number| { n.reduce(); n };
		assert_eq!(REDUCED("[[[[[9,8],1],2],3],4]".parse()?).to_string(), "[[[[0,9],2],3],4]");
		assert_eq!(REDUCED("[7,[6,[5,[4,[3,2]]]]]".parse()?).to_string(), "[7,[6,[5,[7,0]]]]");
		assert_eq!(REDUCED("[[6,[5,[4,[3,2]]]],1]".parse()?).to_string(), "[[6,[5,[7,0]]],3]");
		assert_eq!(REDUCED("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]".parse()?).to_string(), "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");
		Ok(())
	}
}


impl Add for Number {
	type Output = Number;
	fn add(self, rhs: Self) -> Self::Output {
		use InnerNumber::*;
		let mut res = Number(Pair(Box::new(self)), Pair(Box::new(rhs)));
		res.reduce();
		res
	}
}

impl AddAssign for Number {
	fn add_assign(&mut self, rhs: Self) {
		use InnerNumber::Regular;
		let old_self = std::mem::replace(self, Number(Regular(u64::MAX), Regular(u64::MAX)));
		*self = old_self + rhs
	}
}

impl Sum for Number {
	fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
		let mut l = iter.next().unwrap();
		while let Some(r) = iter.next() {
			l += r;
		}
		l
	}
}

impl InnerNumber {
	fn magnitude(&self) -> u64 {
		use InnerNumber::*;
		match self {
			Regular(r) => *r,
			Pair(n) => n.magnitude(),
		}
	}
}

impl Number {
	fn magnitude(&self) -> u64 {
		3 * self.0.magnitude() + 2 * self.1.magnitude()
	}
}


fn input_numbers_from_str(s: &str) -> Vec<Number> {
	parsing::numbers_from_str(s).unwrap()
}

fn input_numbers() -> Vec<Number> {
	input_numbers_from_str(include_str!("day18.txt"))
}


fn part1_impl(input_numbers: Vec<Number>) -> u64 {
	input_numbers.into_iter().sum::<Number>().magnitude()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_numbers())
}


fn part2_impl(input_numbers: Vec<Number>) -> u64 {
	let base = input_numbers.into_iter().enumerate();
	let rev = base.clone().rev();

	fn max_magnitude(input_iterator: impl Iterator<Item = (usize, Number)> + Clone) -> u64 {
		use itertools::*;
		input_iterator
			.tuple_combinations()
			.map(|((li, l), (ri, r))| (li, ri, l + r))
			.map(|(_, _, sum)| sum.magnitude())
			.max()
			.unwrap()
	}

	max_magnitude(base).max(max_magnitude(rev))
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_numbers_from_str(include_str!("day18.txt")))
}


mod parsing {
	use std::{num::ParseIntError, str::FromStr};
	use super::{InnerNumber, Number};

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(crate) enum ParseNumberError {
		InvalidFormat { column: usize, found: Option<char> },
		RegularAtRoot,
		InvalidRegular { column: usize, source: ParseIntError },
		MissingPairComma { column: usize, found: Option<char> },
		MissingPairClosingBracket { column: usize, found: Option<char> },
	}

	impl FromStr for Number {
		type Err = ParseNumberError;
		fn from_str(mut s: &str) -> Result<Self, Self::Err> {
			use ParseNumberError::*;

			let mut column = 1;
			if !s.starts_with("[") {
				Err(InvalidFormat { column, found: s.chars().next() })?
			}

			let mut state = vec![None];
			loop {
				let inner = match state.last_mut() {
					Some(found_pair @ None) => {
						if s.starts_with("[") {
							s = &s[1..];
							column += 1;
							*found_pair = Some(None);
							state.push(None);
							None
						} else {
							let end = s.find(|c: char| !c.is_numeric()).unwrap_or(s.len());
							let regular = s[..end].parse()
								.map_err(|e| InvalidRegular { column, source: e })?;
							s = &s[end..];
							column += end;
							Some(InnerNumber::Regular(regular))
						}
					}
					Some(Some(Some((_, found_comma @ None)))) => {
						if !s.starts_with(",") {
							return Err(MissingPairComma { column, found: s.chars().next() })
						}
						s = &s[1..];
						column += 1;
						*found_comma = Some(None);
						state.push(None);
						None
					}
					Some(inner) => {
						if !s.starts_with("]") {
							return Err(MissingPairClosingBracket { column, found: s.chars().next() })
						}
						s = &s[1..];
						column += 1;
						Some(match inner.take() {
							Some(Some((l, Some(Some(r))))) => {
								InnerNumber::Pair(Box::new(Number(l, r)))
							}
							_ => unreachable!()
						})
					}
					_ => unreachable!()
				};

				if let Some(inner) = inner {
					let removed = state.remove(state.len() - 1);
					assert!(removed.is_none());
					match state.last_mut() {
						Some(Some(state @ None)) => {
							*state = Some((inner, None))
						}
						Some(Some(Some((_, Some(state @ None))))) => {
							*state = Some(inner)
						}
						None => return match inner {
							InnerNumber::Pair(number) => {
								if let found @ Some(_) = s.chars().next() {
									Err(InvalidFormat { column, found })?
								}
								Ok(*number)
							}
							_ => Err(RegularAtRoot),
						},
						_ => unreachable!(),
					}
				}
			}
		}
	}
	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) struct ParseNumbersError {
		pub(super) line: usize,
		pub(super) source: ParseNumberError
	}

	pub(super) fn numbers_from_str(s: &str) -> Result<Vec<Number>, ParseNumbersError> {
		Ok(s.lines()
			.enumerate()
			.map(|(l, line)| line.parse::<Number>()
				.map_err(|e| ParseNumbersError { line: l + 1, source: e }))
			.collect::<Result<_, _>>()?)
	}


	#[test]
	fn test() {
		use {ParseNumberError::*, InnerNumber::*};
		assert!(matches!("".parse::<Number>(), Err(InvalidFormat { column: 1, found: None })));
		assert!(matches!("0".parse::<Number>(), Err(InvalidFormat { column: 1, found: Some('0') })));
		assert!(matches!("[".parse::<Number>(), Err(InvalidRegular { column: 2, .. })));
		assert!(matches!("[1".parse::<Number>(), Err(MissingPairComma { column: 3, found: None })));
		assert!(matches!("[1x".parse::<Number>(), Err(MissingPairComma { column: 3, found: Some('x') })));
		assert!(matches!("[1,2".parse::<Number>(), Err(MissingPairClosingBracket { column: 5, found: None })));
		assert!(matches!("[1,2x".parse::<Number>(), Err(MissingPairClosingBracket { column: 5, found: Some('x') })));
		assert!(matches!("[1,2]".parse::<Number>(), Ok(Number(Regular(1), Regular(2)))));
		assert!(matches!("[1,2]x".parse::<Number>(), Err(InvalidFormat { column: 6, found: Some('x') })));
		assert!(matches!("[[3,4],5]".parse::<Number>(), Ok(Number(Pair(num), Regular(5)))
			if matches!(*num, Number(Regular(3), Regular(4)))));
		assert!(matches!("[6,[7,8]]".parse::<Number>(), Ok(Number(Regular(6), Pair(num)))
			if matches!(*num, Number(Regular(7), Regular(8)))));
	}
}


#[test]
fn add() -> Result<(), parsing::ParseNumberError> {
	assert_eq!(("[1,2]".parse::<Number>()? + "[[3,4],5]".parse::<Number>()?).to_string(),
		"[[1,2],[[3,4],5]]");
	assert_eq!(("[[[[4,3],4],4],[7,[[8,4],9]]]".parse::<Number>()? + "[1,1]".parse::<Number>()?).to_string(),
		"[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
	assert_eq!(parsing::numbers_from_str(indoc::indoc! { "
		[1,1]
		[2,2]
		[3,3]
		[4,4]
	" }).map_err(|e| e.source)?.into_iter().sum::<Number>().to_string(),
		"[[[[1,1],[2,2]],[3,3]],[4,4]]");
	assert_eq!(parsing::numbers_from_str(indoc::indoc! { "
		[1,1]
		[2,2]
		[3,3]
		[4,4]
		[5,5]
	" }).map_err(|e| e.source)?.into_iter().sum::<Number>().to_string(),
		"[[[[3,0],[5,3]],[4,4]],[5,5]]");
	assert_eq!(parsing::numbers_from_str(indoc::indoc! { "
		[1,1]
		[2,2]
		[3,3]
		[4,4]
		[5,5]
		[6,6]
	" }).map_err(|e| e.source)?.into_iter().sum::<Number>().to_string(),
		"[[[[5,0],[7,4]],[5,5]],[6,6]]");
	assert_eq!(parsing::numbers_from_str(indoc::indoc! { "
		[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
		[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
		[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
		[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
		[7,[5,[[3,8],[1,4]]]]
		[[2,[2,2]],[8,[8,1]]]
		[2,9]
		[1,[[[9,3],9],[[9,0],[0,7]]]]
		[[[5,[7,4]],7],1]
		[[[[4,2],2],6],[8,7]]
	" }).map_err(|e| e.source)?.into_iter().sum::<Number>().to_string(),
		"[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]");
	Ok(())
}

#[test]
fn magnitude() -> Result<(), parsing::ParseNumberError> {
	assert_eq!("[9,1]".parse::<Number>()?.magnitude(), 29);
	assert_eq!("[1,9]".parse::<Number>()?.magnitude(), 21);
	assert_eq!("[[9,1],[1,9]]".parse::<Number>()?.magnitude(), 129);
	assert_eq!("[[1,2],[[3,4],5]]".parse::<Number>()?.magnitude(), 143);
	assert_eq!("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]".parse::<Number>()?.magnitude(), 1384);
	assert_eq!("[[[[1,1],[2,2]],[3,3]],[4,4]]".parse::<Number>()?.magnitude(), 445);
	assert_eq!("[[[[3,0],[5,3]],[4,4]],[5,5]]".parse::<Number>()?.magnitude(), 791);
	assert_eq!("[[[[5,0],[7,4]],[5,5]],[6,6]]".parse::<Number>()?.magnitude(), 1137);
	assert_eq!("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]".parse::<Number>()?.magnitude(), 3488);
	Ok(())
}

#[test]
fn tests() -> Result<(), parsing::ParseNumberError> {
	const INPUT: &str = indoc::indoc! { "
		[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
		[[[5,[2,8]],4],[5,[[9,9],0]]]
		[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
		[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
		[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
		[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
		[[[[5,4],[7,7]],8],[[8,3],8]]
		[[9,3],[[9,9],[6,[4,9]]]]
		[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
		[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]
	" };
	assert_eq!(part1_impl(input_numbers_from_str(INPUT)), 4140);
	assert_eq!(part1(), 4347);
	assert_eq!(part2_impl(input_numbers_from_str(INPUT)), 3993);
	assert_eq!(part2(), 4721);
	Ok(())
}
