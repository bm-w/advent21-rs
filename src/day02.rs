// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{str::FromStr, num::ParseIntError};


pub(crate) enum CommandDir { Forward, Down, Up }
pub(crate) struct Command(CommandDir, u64);


fn input_commands_from_str(s: &str) -> impl Iterator<Item = Command> + '_ {
	s.lines().map(|line| str::parse::<Command>(line).unwrap())
}

fn input_commands() -> impl Iterator<Item = Command> {
	input_commands_from_str(include_str!("day02.txt"))
}


fn part1_impl(input_commands: impl Iterator<Item = Command>) -> u64 {
	let mut pos = (0, 0); // Horz., vert.
	for command in input_commands {
		use CommandDir::*;
		match command {
			Command(Forward, amount) => { pos.0 += amount }
			Command(Down, amount) => { pos.1 += amount }
			Command(Up, amount) => { assert!(pos.1 >= amount); pos.1 -= amount }
		}
	}
	pos.0 * pos.1
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_commands())
}


fn part2_impl(input_commands: impl Iterator<Item = Command>) -> u64 {
	let mut pos = (0, 0); // Horz., vert.
	let mut aim = 0;
	for command in input_commands {
		use CommandDir::*;
		match command {
			Command(Forward, amount) => {
				let depth_amount = amount as i64 * aim;
				assert!(depth_amount >= 0 || -depth_amount as u64 <= pos.1);
				pos.0 += amount;
				pos.1 = (pos.1 as i64 + depth_amount) as u64;
			}
			Command(Down, amount) => { aim += amount as i64 }
			Command(Up, amount) => { aim -= amount as i64 }
		}
	}
	pos.0 * pos.1
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_commands())
}


#[derive(Debug)]
pub(crate) enum ParseCommandError {
	Empty,
	InvalidFormat(String),
	InvalidDir(String),
	InvalidAmount(ParseIntError),
}

impl FromStr for CommandDir {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, ()> {
		match s {
			"forward" => Ok(Self::Forward),
			"down" => Ok(Self::Down),
			"up" => Ok(Self::Up),
			_ => Err(()),
		}
	}
}

impl FromStr for Command {
	type Err = ParseCommandError;
	fn from_str(s: &str) -> Result<Self, ParseCommandError> {
		if s.is_empty() || s.trim().is_empty() { return Err(ParseCommandError::Empty); }
		let (dir, amount) = s.split_once(char::is_whitespace)
			.ok_or_else(|| ParseCommandError::InvalidFormat(s.to_owned()))?;
		let dir = dir.parse::<CommandDir>()
			.map_err(|_| ParseCommandError::InvalidDir(dir.to_owned()))?;
		let amount = amount.parse::<u64>()
			.map_err(ParseCommandError::InvalidAmount)?;
		Ok(Command(dir, amount))
	}
}


#[test]
fn tests() {
	const INPUT_COMMANDS: &str = indoc::indoc! { "
		forward 5
		down 5
		forward 8
		up 3
		down 8
		forward 2
	" };
	assert_eq!(part1_impl(input_commands_from_str(INPUT_COMMANDS)), 150);
	assert_eq!(part2_impl(input_commands_from_str(INPUT_COMMANDS)), 900);
}
