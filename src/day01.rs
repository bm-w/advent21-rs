// Copyright (c) 2022 Bastiaan Marinus van de Weerd


pub(crate) fn input_nums() -> impl Iterator<Item = u64> {
	include_str!("day01.txt")
		.lines()
		.map(|line| str::parse::<u64>(line).unwrap())
}

pub(crate) fn part1(mut input_nums: impl Iterator<Item = u64>) -> usize {
	let mut prev = input_nums.next().unwrap();
	let mut incrs = 0;
	for num in input_nums {
		if num > prev { incrs += 1 }
		prev = num;
	}
	incrs
}

pub(crate) fn part2(mut input_nums: impl Iterator<Item = u64>) -> usize {
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


#[test]
fn tests() {
	const INPUT_NUMS: &[u64] = &[199, 200, 208, 210, 200, 207, 240, 269, 260, 263];
	assert_eq!(part1(INPUT_NUMS.iter().copied()), 7);
	assert_eq!(part2(INPUT_NUMS.iter().copied()), 5);
}
