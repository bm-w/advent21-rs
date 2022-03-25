// Copyright (c) 2022 Bastiaan Marinus van de Weerd


#[derive(Debug, Clone, Copy)]
enum Reg { W, X, Y, Z }

#[derive(Debug, Clone, Copy)]
enum Operand { Reg(Reg), Val(i64) }

#[derive(Debug)]
enum Instr {
	Inp(Reg),
	Add(Reg, Operand),
	Mul(Reg, Operand),
	Div(Reg, Operand),
	Mod(Reg, Operand),
	Eql(Reg, Operand),
}


/// Assuming this much about the program structure feels a little like cheating,
/// but it seems that everybody does this. Let’s just take it that the program
/// structure is fixed and the actual input is the parameters within (hereafter
/// called the division denominator, push addend, & pop subtrahend).
mod analysis {
	use itertools::Itertools as _;
	use super::{Reg, Operand, Instr};

	const SUBPROGRAMS_LEN: usize = 14;
	const SUBPROGRAM_INSTRS_LEN: usize = 18;
	const SUBPROGRAM_DIV_OFFSET: usize = 4;
	const SUBPROGRAM_POP_SUBTRAHEND_OFFSET: usize = 5;
	const SUBPROGRAM_PUSH_ADDEND_OFFSET: usize = 15;

	/// Each subprogram will either push or pop a base-26 digit into the stack
	/// that is represented by the Z register across the whole program. A valid
	/// whole program consists of 7 pairs of push & pop subprograms, each of
	/// which exclusively determine the possible values for the two (base-10)
	/// digits of a serial number corresponding to the subprograms’ locations.
	#[derive(Debug)]
	enum Subprogram {
		Push { addend: u8 },
		Pop { subtrahend: u8 },
	}

	pub(super) struct Analysis([Subprogram; SUBPROGRAMS_LEN]);

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum AnalysisError {
		InvalidSubprogram { div_found: Option<Instr>, pop_found: Option<Instr>, push_found: Option<Instr> },
		InvalidSubprogramsLen(usize),
		UnbalancedSubprograms { pushes: usize, pops: usize },
	}

	impl FromIterator<Instr> for Result<Analysis, AnalysisError> {
		fn from_iter<T: IntoIterator<Item = Instr>>(iter: T) -> Self {
			use {Reg::*, Operand::Val, Instr::*, AnalysisError::*};
			Ok(Analysis(iter.into_iter()
				.chunks(SUBPROGRAM_INSTRS_LEN)
				.into_iter()
				.map(|mut chunk| {
					match (
						chunk.nth(SUBPROGRAM_DIV_OFFSET),
						chunk.next(), // Equiv. to `.nth(SUBPROGRAM_POP_SUBTRAHEND_OFFSET - SUBPROGRAM_DIV_OFFSET - 1)`
						chunk.nth(SUBPROGRAM_PUSH_ADDEND_OFFSET - SUBPROGRAM_POP_SUBTRAHEND_OFFSET - 1),
					) {
						(
							Some(Div(Z, Val(div_denom))),
							Some(Add(X, Val(pop_addend))), // “Addend” here, but negated below to become the “subtrahend”
							Some(Add(Y, Val(push_addend))),
						)
						if (div_denom == 1 || div_denom == 26 && (-16..=0).contains(&pop_addend))
						&& (0..=16).contains(&push_addend) =>
							Ok(
								if div_denom == 1 { Subprogram::Push { addend: push_addend as u8 } }
								else { Subprogram::Pop { subtrahend: -pop_addend as u8 } }
							),
						(d, ps, pa) =>
							Err(InvalidSubprogram {
								div_found: d,
								pop_found: ps,
								push_found: pa
							})
					}
				})
				.collect::<Result<Vec<_>, _>>()?
				.try_into()
				.map_err(|e: Vec<_>| InvalidSubprogramsLen(e.len()))
				.and_then(|subprograms: [Subprogram; SUBPROGRAMS_LEN]| {
					let (pushes, pops) = subprograms.iter()
						.fold((0, 0, ), |(pushes, pops), sp| match sp {
							Subprogram::Push { .. } => (pushes + 1, pops),
							Subprogram::Pop { .. } => (pushes, pops + 1),
						});
					if pushes != pops { Err(UnbalancedSubprograms { pushes, pops }) }
					else { Ok(subprograms) }
				})?))
		}
	}

	impl Analysis {
		fn push_pop_pairs(&self) -> [(usize, u8, usize, u8); SUBPROGRAMS_LEN / 2] {
			let mut pairs = Vec::with_capacity(SUBPROGRAMS_LEN / 2);
			let mut stack = Vec::with_capacity(SUBPROGRAMS_LEN / 2);
			for (i, subprogram) in self.0.iter().enumerate() {
				match subprogram {
					Subprogram::Push { addend } => stack.push((i, *addend)),
					Subprogram::Pop { subtrahend } =>
						if let Some((push_i, push_addend)) = stack.pop() {
							pairs.push((push_i, push_addend, i, *subtrahend))
						}
				}
			}
			pairs.sort_by(|l, r| l.0.cmp(&r.0));
			pairs.try_into().unwrap()
		}

		fn find_serial_number(&self, push: fn(u8, u8) -> u8) -> u64 {
			let mut number = 0;
			for (push_i, push_addend, pop_i, pop_subtrahend) in self.push_pop_pairs() {
				let push = push(push_addend, pop_subtrahend);
				let push_digit = push - push_addend;
				let pop_digit = push - pop_subtrahend;
				number += push_digit as u64 * 10u64.pow(13 - push_i as u32);
				number += pop_digit as u64 * 10u64.pow(13 - pop_i as u32);
			}
			number
		}

		pub(super) fn max_serial_number(&self) -> u64 {
			self.find_serial_number(|push_addend, pop_subtrahend|
				(9 + push_addend).min(9 + pop_subtrahend))
		}

		pub(super) fn min_serial_number(&self) -> u64 {
			self.find_serial_number(|push_addend, pop_subtrahend|
				(1 + push_addend).max(1 + pop_subtrahend))
		}
	}
}


fn input_instrs_from_str(s: &str) -> Vec<Instr> {
	parsing::try_instrs_from_str(s).unwrap()
}

fn input_instrs() -> Vec<Instr> {
	input_instrs_from_str(include_str!("day24.txt"))
}


fn part1_impl(input_instrs: Vec<Instr>) -> u64 {
	Result::<analysis::Analysis, _>::from_iter(input_instrs.into_iter()).unwrap()
		.max_serial_number()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_instrs())
}


fn part2_impl(input_instrs: Vec<Instr>) -> u64 {
	Result::<analysis::Analysis, _>::from_iter(input_instrs.into_iter()).unwrap()
		.min_serial_number()
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_instrs())
}


mod parsing {
	use std::{str::FromStr, num::ParseIntError};
	use super::{Reg, Operand, Instr};

	#[derive(Debug)]
	pub(super) struct InvalidRegError(Option<char>);

	impl TryFrom<char> for Reg {
		type Error = InvalidRegError;
		fn try_from(chr: char) -> Result<Self, Self::Error> {
			match chr {
				'x' => Ok(Reg::X),
				'y' => Ok(Reg::Y),
				'z' => Ok(Reg::Z),
				'w' => Ok(Reg::W),
				found => Err(InvalidRegError(Some(found))),
			}
		}
	}

	impl FromStr for Reg {
		type Err = InvalidRegError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			let mut chars = s.chars();
			match (chars.next(), chars.next()) {
				(Some(chr), None) => chr.try_into(),
				(found @ None, None) | (_, found) => Err(InvalidRegError(found)),
			}
		}
	}

	#[derive(Debug)]
	pub(super) enum OperandError {
		InvalidFormat,
		Invalid(InvalidRegError, ParseIntError),
	}

	impl FromStr for Operand {
		type Err = OperandError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use OperandError::*;
			if s.is_empty() { return Err(InvalidFormat) }
			s.parse().map_or_else(
				|reg_err| s.parse().map_or_else(
					|val_err| Err(Invalid(reg_err, val_err)),
					|val| Ok(Operand::Val(val))),
				|reg| Ok(Operand::Reg(reg))
			)
		}
	}

	#[allow(dead_code, clippy::enum_variant_names)]
	#[derive(Debug)]
	pub(super) enum InstrError {
		InvalidFormat,
		InvalidInstr { found: String },
		InvalidReg(InvalidRegError),
		InvalidArgs { found: String },
		InvalidOperand(OperandError),
	}

	impl FromStr for Instr {
		type Err = InstrError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use InstrError::*;
			let (instr, args) = s.split_once(' ')
				.ok_or(InvalidFormat)?;

			fn try_reg_arg(arg: &str) -> Result<Reg, InstrError> {
				arg.parse().map_err(InvalidReg)
			}

			fn try_args(args: &str) -> Result<(Reg, Operand), InstrError> {
				let (reg, operand) = args.split_once(' ')
					.ok_or_else(|| InvalidArgs { found: args.to_owned() })?;
				let reg = try_reg_arg(reg)?;
				let operand = operand.parse().map_err(InvalidOperand)?;
				Ok((reg, operand))
			}

			match instr {
				"inp" => try_reg_arg(args).map(Instr::Inp),
				"add" => try_args(args).map(|(reg, operand)| Instr::Add(reg, operand)),
				"mul" => try_args(args).map(|(reg, operand)| Instr::Mul(reg, operand)),
				"div" => try_args(args).map(|(reg, operand)| Instr::Div(reg, operand)),
				"mod" => try_args(args).map(|(reg, operand)| Instr::Mod(reg, operand)),
				"eql" => try_args(args).map(|(reg, operand)| Instr::Eql(reg, operand)),
				found => Err(InvalidInstr { found: found.to_owned() })
			}
		}
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) struct InstrsError {
		line: usize,
		source: InstrError,
	}

	pub(super) fn try_instrs_from_str(s: &str) -> Result<Vec<Instr>, InstrsError> {
		s.lines()
			.enumerate()
			.map(|(l, line)| line.parse()
				.map_err(|e| InstrsError { line: l + 1, source: e }))
			.collect::<Result<_, _>>()
	}

	#[test]
	fn reg() {
		assert!(matches!(Reg::try_from('a'), Err(InvalidRegError(Some('a')))));
		assert!(matches!(Reg::try_from('x'), Ok(Reg::X)));
		assert!(matches!(Reg::try_from('y'), Ok(Reg::Y)));
		assert!(matches!(Reg::try_from('z'), Ok(Reg::Z)));
		assert!(matches!(Reg::try_from('w'), Ok(Reg::W)));
		assert!(matches!(Reg::from_str("ab"), Err(InvalidRegError(Some('b')))));
	}

	#[test]
	fn operand() {
		use OperandError::*;
		assert!(matches!("".parse::<Operand>(), Err(InvalidFormat)));
		assert!(matches!("a".parse::<Operand>(), Err(Invalid(InvalidRegError(Some('a')), ParseIntError { .. }))));
		assert!(matches!("ab".parse::<Operand>(), Err(Invalid(InvalidRegError(Some('b')), ParseIntError { .. }))));
		assert!(matches!("x".parse::<Operand>(), Ok(Operand::Reg(Reg::X))));
		assert!(matches!("1337".parse::<Operand>(), Ok(Operand::Val(1337))));
	}

	#[test]
	fn instr() {
		use InstrError::*;
		assert!(matches!("".parse::<Instr>(), Err(InvalidFormat)));
		assert!(matches!("foo".parse::<Instr>(), Err(InvalidFormat)));
		assert!(matches!("foo bar".parse::<Instr>(), Err(InvalidInstr { found }) if found.as_str() == "foo"));
		assert!(matches!("inp bar".parse::<Instr>(), Err(InvalidReg(_))));
		assert!(matches!("inp x".parse::<Instr>(), Ok(Instr::Inp(Reg::X))));
		assert!(matches!("add bar".parse::<Instr>(), Err(InvalidArgs { found }) if found.as_str() == "bar"));
		assert!(matches!("add x bar".parse::<Instr>(), Err(InvalidOperand(_))));
		assert!(matches!("add x 1".parse::<Instr>(), Ok(Instr::Add(Reg::X, Operand::Val(1)))));
	}
}


#[test]
fn tests() {
	assert_eq!(part1(), 65984919997939);
	assert_eq!(part2(), 11211619541713);
}


#[cfg(test)]
/// Program execution is not actually required to solve this puzzle,
/// but I’d already implemented it so might as well leave it here.
mod execution {
	use std::ops::{Index, IndexMut};
	use super::{Reg, Operand, Instr};

	impl From<Reg> for usize {
		fn from(reg: Reg) -> Self {
			match reg {
				Reg::W => 0,
				Reg::X => 1,
				Reg::Y => 2,
				Reg::Z => 3,
			}
		}
	}

	impl Index<Reg> for [i64; 4] {
		type Output = i64;
		fn index(&self, index: Reg) -> &Self::Output {
			&self[index as usize]
		}
	}

	impl IndexMut<Reg> for [i64; 4] {
		fn index_mut(&mut self, index: Reg) -> &mut Self::Output {
			&mut self[index as usize]
		}
	}

	impl Operand {
		fn resolve(&self, state: &[i64; 4]) -> i64 {
			use Operand::*;
			match self {
				Reg(reg) => state[*reg],
				Val(val) => *val,
			}
		}
	}

	#[derive(Debug, PartialEq, Eq)]
	pub(super) enum InstrError { DivByZero, ModByZero }

	#[derive(Debug, PartialEq, Eq)]
	pub(super) struct ProgramError {
		instr: usize,
		source: InstrError
	}

	impl Instr {
		fn execute(&self, state: &mut [i64; 4], input: &mut impl Iterator<Item = i64>) -> Result<(), InstrError> {
			use Instr::*;
			match self {
				Inp(reg) =>
					state[*reg] = input.next().unwrap(),
				Add(reg, operand) =>
					state[*reg] += operand.resolve(state),
				Mul(reg, operand) =>
					state[*reg] *= operand.resolve(state),
				Div(reg, operand) => {
					let val = operand.resolve(state);
					if val == 0 { return Err(InstrError::DivByZero) }
					state[*reg] /= val;
				}
				Mod(reg, operand) => {
					let val = operand.resolve(state);
					if val == 0 { return Err(InstrError::ModByZero) }
					state[*reg] %= val;
				}
				Eql(reg, operand) =>
					state[*reg] = if state[*reg] == operand.resolve(state) { 1 } else { 0 },
			}
			Ok(())
		}

		pub(super) fn execute_program(
			instrs: impl IntoIterator<Item = Instr>,
			input: impl IntoIterator<Item = i64>
		) -> Result<[i64; 4], ProgramError> {
			let mut input = input.into_iter();
			let mut state = [0; 4];
			for (i, instr) in instrs.into_iter().enumerate() {
				instr.execute(&mut state, &mut input)
					.map_err(|e| ProgramError { instr: i, source: e })?;
			}
			Ok(state)
		}
	}

	#[test]
	fn tests() -> Result<(), ProgramError> {
		use {super::Reg::*, Operand::*, Instr::*};

		const NEGATE_PROGRAM: [Instr; 2] = [
			Inp(X),
			Mul(X, Val(-1))
		];
		const IS_TRIPLE_PROGRAM: [Instr; 4] = [
			Inp(Z),
			Inp(X),
			Mul(Z, Val(3)),
			Eql(Z, Reg(X)),
		];
		const BINARY_PROGRAM: [Instr; 11] = [
			Inp(W),
			Add(Z, Reg(W)),
			Mod(Z, Val(2)),
			Div(W, Val(2)),
			Add(Y, Reg(W)),
			Mod(Y, Val(2)),
			Div(W, Val(2)),
			Add(X, Reg(W)),
			Mod(X, Val(2)),
			Div(W, Val(2)),
			Mod(W, Val(2)),
		];

		assert_eq!(Instr::execute_program(NEGATE_PROGRAM, [1337])?[X], -1337);
		assert_eq!(Instr::execute_program(NEGATE_PROGRAM, [54321])?[X], -54321);
		assert_eq!(Instr::execute_program(IS_TRIPLE_PROGRAM, [1337, 4011])?[Z], 1);
		assert_eq!(Instr::execute_program(IS_TRIPLE_PROGRAM, [54321, 4011])?[Z], 0);
		assert_eq!(Instr::execute_program(BINARY_PROGRAM, [13])?, [1, 1, 0, 1]);
		assert_eq!(Instr::execute_program(BINARY_PROGRAM, [7])?, [0, 1, 1, 1]);
		assert_eq!(Instr::execute_program(BINARY_PROGRAM, [0])?, [0, 0, 0, 0]);

		const INVALID_MODEL_NUMBER_DIGITS: [i64; 14] = [1, 3, 5, 7, 9, 2, 4, 6, 8, 9, 9, 9, 9, 9];
		let mut digits_iter = INVALID_MODEL_NUMBER_DIGITS.into_iter();
		assert_eq!(Instr::execute_program(super::input_instrs(), digits_iter.by_ref())?[Z], 2695331544);
		assert_eq!(digits_iter.next(), None);

		/// Initially derived by hand before solving in the `analysis` module below

		const MAX_VALID_MODEL_NUMBER_DIGITS: [i64; 14] = [6, 5, 9, 8, 4, 9, 1, 9, 9, 9, 7, 9, 3, 9];
		let mut digits_iter = MAX_VALID_MODEL_NUMBER_DIGITS.into_iter();
		assert_eq!(Instr::execute_program(super::input_instrs(), digits_iter.by_ref())?[Z], 0);
		assert_eq!(digits_iter.next(), None);

		const MIN_VALID_MODEL_NUMBER_DIGITS: [i64; 14] = [1, 1, 2, 1, 1, 6, 1, 9, 5, 4, 1, 7, 1, 3];
		let mut digits_iter = MIN_VALID_MODEL_NUMBER_DIGITS.into_iter();
		assert_eq!(Instr::execute_program(super::input_instrs(), digits_iter.by_ref())?[Z], 0);
		assert_eq!(digits_iter.next(), None);

		Ok(())
	}
}
