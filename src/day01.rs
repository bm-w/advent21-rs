// Copyright (c) 2022 Bastiaan Marinus van de Weerd


fn input_nums_from_str(s: &str) -> impl Iterator<Item = u64> + '_ {
	s.lines().map(|line| str::parse::<u64>(line).unwrap())
}

fn input_nums() -> impl Iterator<Item = u64> {
	input_nums_from_str(include_str!("day01.txt"))
}


fn part1_impl(mut input_nums: impl Iterator<Item = u64>) -> usize {
	let mut prev = input_nums.next().unwrap();
	let mut incrs = 0;
	for num in input_nums {
		if num > prev { incrs += 1 }
		prev = num;
	}
	incrs
}

pub(crate) fn part1() -> usize {
	part1_impl(input_nums())
}


fn part2_impl(mut input_nums: impl Iterator<Item = u64>) -> usize {
	let mut prevs = [
		input_nums.next().unwrap(),
		input_nums.next().unwrap(),
		input_nums.next().unwrap(),
	];
	let mut prevs_idx = 0;
	let mut prev_sum = prevs.iter().sum::<u64>();
	let mut incrs = 0;
	for num in input_nums {
		prevs[prevs_idx] = num;
		let sum = prevs.iter().sum::<u64>();
		if sum > prev_sum { incrs += 1 }
		prev_sum = sum;
		prevs_idx = (prevs_idx + 1) % prevs.len();
	}
	incrs
}

pub(crate) fn part2() -> usize {
	part2_impl(input_nums())
}


#[test]
fn tests() {
	const INPUT_NUMS: &[u64] = &[199, 200, 208, 210, 200, 207, 240, 269, 260, 263];
	assert_eq!(part1_impl(INPUT_NUMS.iter().copied()), 7);
	assert_eq!(part2_impl(INPUT_NUMS.iter().copied()), 5);
}
