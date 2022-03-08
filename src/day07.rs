// Copyright (c) 2022 Bastiaan Marinus van de Weerd


fn input_positions_from_str(s: &str) -> impl Iterator<Item = u16> + '_ {
	s.lines().next().unwrap()
		.split(',').map(|num| num.parse::<u16>().unwrap())
}

fn input_positions() -> impl Iterator<Item = u16> {
	input_positions_from_str(include_str!("day07.txt"))
}


// TODO(bm-w): Use more optimal minimization alg., or deterministic method (e.g. part 1 is just the median position)
fn part1and2_naive(input_positions: impl Iterator<Item = u16>, cost_fn: fn(u64) -> u64) -> u64 {
	let input_positions = input_positions.collect::<Vec<_>>();
	let max_input_pos = *input_positions.iter().max().unwrap();
	let mut cheapest_cost = u64::MAX;
	for target in 0..=max_input_pos {
		let mut cost = 0;
		for &pos in input_positions.iter() {
			cost += cost_fn(u16_abs_diff(pos, target) as u64);
		}
		if cost < cheapest_cost {
			cheapest_cost = cost
		}
	}
	cheapest_cost
}

fn cost_fn_identity(dist: u64) -> u64 {
	dist
}

fn cost_fn_quasisquared(dist: u64) -> u64 {
	(dist * dist + dist) / 2
}

pub(crate) fn part1() -> u64 {
	part1and2_naive(input_positions(), cost_fn_identity)
}

pub(crate) fn part2() -> u64 {
	part1and2_naive(input_positions(), cost_fn_quasisquared)
}


// TODO(bm-w): Drop in favor of `u16::abs_diff` once stabilized
fn u16_abs_diff(l: u16, r: u16) -> u16 {
	if l > r { l - r } else { r - l }
}


#[test]
fn tests() {
	const INPUT: &str = "16,1,2,0,4,2,7,1,2,14";
	assert_eq!(part1and2_naive(input_positions_from_str(INPUT), cost_fn_identity), 37);
	assert_eq!(part1and2_naive(input_positions_from_str(INPUT), cost_fn_quasisquared), 168);
}
