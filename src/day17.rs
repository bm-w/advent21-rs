// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{ops::RangeInclusive, str::FromStr, num::ParseIntError};
use itertools::Itertools;


#[derive(Debug)]
struct TargetArea {
	x: RangeInclusive<i32>,
	y: RangeInclusive<i32>,
}

impl TargetArea {
	fn min_initial_x_velocity(&self) -> u32 {
		(1..).find(|v| v * (v + 1) / 2 >= *self.x.start() as u32).unwrap()
	}

	fn valid_initial_x_velocities(&self) -> RangeInclusive<u32> {
		let v_start = self.min_initial_x_velocity();
		v_start as u32..=*self.x.end() as u32 + 1
	}

	fn max_initial_y_velocity(&self) -> u32 {
		if *self.y.end() >= 0 { unimplemented!() }
		// If we shoot up, we’ll eventually end up back at `y == 0` with
		// with velocity `-ivy - 1`, so we need to make sure we don’t
		// ‘tunnel’ through the target area on the next step.
		self.y.start().abs() as u32 - 1
	}

	fn valid_initial_y_velocities(&self) -> RangeInclusive<i32> {
		*self.y.start()..=self.max_initial_y_velocity() as i32
	}

	fn max_steps(&self, initival_x_velocity: u32) -> Option<usize> {
		let mut x = 0;
		let mut vx = initival_x_velocity as i32;
		for i in 0.. {
			if vx <= 0 { return None }
			if x > *self.x.end() { return Some(i) }
			x += vx;
			vx -= 1;
		}
		unreachable!()
	}

	fn contains(&self, pos: (i32, i32)) -> bool {
		self.x.contains(&pos.0) && self.y.contains(&pos.1)
	}

	fn may_still_be_hit1(r: &RangeInclusive<i32>, pos: i32, velocity: i32, velocity_may_still_become_negative: bool) -> bool {
		match velocity {
			0 if !r.contains(&pos) => velocity_may_still_become_negative || false,
			1.. => velocity_may_still_become_negative || pos < *r.end(),
			vx if vx < 0 => pos > *r.start(), // TODO(bm-w): `..-1` once half-open ranges are stable
			_ => true,
		}
	}

	fn may_still_be_hit(&self, pos: (i32, i32), velocity: (i32, i32)) -> bool {
		Self::may_still_be_hit1(&self.x, pos.0, velocity.0, false)
			&& (Self::may_still_be_hit1(&self.y, pos.1, velocity.1, true))
	}

	fn will_be_hit(&self, initial_velocity: (i32, i32)) -> bool {
		let mut velocity = initial_velocity;
		let mut pos = (0, 0);
		loop {
			if !self.may_still_be_hit(pos, velocity) { break false }
			pos = (pos.0 + velocity.0, pos.1 + velocity.1);
			if self.contains(pos) { break true }
			velocity.0 -= i32_sign(velocity.0);
			velocity.1 -= 1;
		}
	}
}


fn input_targret_area_from_str(s: &str) -> TargetArea {
	s.parse().unwrap()
}

fn input_targret() -> TargetArea {
	input_targret_area_from_str(include_str!("day17.txt"))
}


fn part1_impl(input_target_area: TargetArea) -> u32 {
	let mut max_height = 0;
	let mut iv = (0, 0);
	for ivx in input_target_area.min_initial_x_velocity().. {
		if input_target_area.max_steps(ivx).is_some() {
			if max_height == 0 { unimplemented!() }
			// Assuming we’ll never shoot higher in a finite number of steps
			break
		} else {
			let ivy = input_target_area.max_initial_y_velocity();
			let height = ivy * (ivy + 1) / 2;
			max_height = max_height.max(height);
			iv = (ivx as i32, ivy as i32)
		}
	}
	assert!(input_target_area.will_be_hit(iv));
	max_height
}

pub(crate) fn part1() -> u32 {
	part1_impl(input_targret())
}


fn part2_impl(input_target_area: TargetArea) -> usize {
	let mut count = 0;
	for (ivx, ivy) in input_target_area.valid_initial_x_velocities()
		.cartesian_product(input_target_area.valid_initial_y_velocities())
	{
		if input_target_area.will_be_hit((ivx as i32, ivy)) {
			count += 1;
		}
	}
	count
}

pub(crate) fn part2() -> usize {
	part2_impl(input_targret())
}


#[derive(Debug)]
enum ParseRangeInclusiveError {
	InvalidFormat,
	InvalidFrom(ParseIntError),
	InvalidThrough(ParseIntError),
}

fn range_incl_from_str(s: &str) -> Result<RangeInclusive<i32>, ParseRangeInclusiveError> {
	use ParseRangeInclusiveError::*;
	let (from, through) = s.split_once("..")
		.ok_or(InvalidFormat)?;
	let from = from.parse().map_err(|e| InvalidFrom(e))?;
	let through = through.parse().map_err(|e| InvalidThrough(e))?;
	Ok(from..=through)
}

#[allow(dead_code)]
#[derive(Debug)]
enum ParseTargetAreaError {
	InvalidFormat { column: usize, found: String },
	InvalidXFormat { column: usize, found: String },
	InvalidX { column: usize, source: ParseRangeInclusiveError },
	InvalidYFormat { column: usize, found: String },
	InvalidY { column: usize, source: ParseRangeInclusiveError },
}

fn try_skip_prefix<'a>(s: &'a str, prefix: &str) -> Result<&'a str, usize> {
	if s.starts_with(prefix) {
		Ok(&s[prefix.len()..])
	} else {
		let c = s.chars().zip(prefix.chars())
			.enumerate()
			.find(|(_, (l, r))| l != r)
			.map(|(c, _)| c)
			.unwrap_or(0);
		return Err(c)
	}
}

impl FromStr for TargetArea {
	type Err = ParseTargetAreaError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::ParseTargetAreaError::*;

		const PREFIX: &str = "target area: ";
		const X_PREFIX: &str = "x=";
		const Y_PREFIX: &str = "y=";

		let s = s.lines().next()
			.ok_or(InvalidFormat { column: 1, found: s.to_owned() })?;
		let s = try_skip_prefix(s, PREFIX)
			.map_err(|c| InvalidFormat { column: c + 1, found: s.to_owned() })?;
		let (x, y) = s.split_once(", ")
			.ok_or(InvalidFormat { column: PREFIX.len() + 1, found: s.to_owned() })?;
		let y_offset = || PREFIX.len() + s.find(", ").unwrap() + 2;
		let x = try_skip_prefix(x, X_PREFIX)
			.map_err(|c| InvalidXFormat { column: PREFIX.len() + c + 1, found: x.to_owned() })?;
		let x = range_incl_from_str(x)
			.map_err(|e| InvalidX { column: PREFIX.len() + X_PREFIX.len() + 1, source: e })?;
		let y = try_skip_prefix(y, Y_PREFIX)
			.map_err(|c| InvalidYFormat { column: y_offset() + c + 1, found: y.to_owned() })?;
		let y = range_incl_from_str(y)
			.map_err(|e| InvalidY { column: y_offset() + Y_PREFIX.len() + 1, source: e })?;
		Ok(TargetArea { x, y, })
	}
}


#[test]
fn tests() {
	const INPUT: &str = "target area: x=20..30, y=-10..-5";
	assert_eq!(part1_impl(input_targret_area_from_str(INPUT)), 45);
	assert_eq!(part1(), 8646);
	assert_eq!(part2_impl(input_targret_area_from_str(INPUT)), 112);
	assert_eq!(part2(), 5945);
}


// Util

fn i32_sign(value: i32) -> i32 {
	match value {
		0 => 0,
		1.. => 1,
		_ => -1,
	}
}
