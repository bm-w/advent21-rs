// Copyright (c) 2022 Bastiaan Marinus van de Weerd


fn input_strs_from_str(s: &str) -> impl Iterator<Item = &str>  {
	s.lines()
}

fn input_strs() -> impl Iterator<Item = &'static str>  {
	input_strs_from_str(include_str!("day03.txt"))
}


fn part1_impl<'a, I, const N: usize>(input_nums: I) -> u64
where I: Iterator<Item = &'a str> {
	let mut counts = [0i64; N];
	for input_num in input_nums {
		assert_eq!(input_num.len(), N);
		for (i, c) in input_num.chars().enumerate() {
			match c {
				'1' => { counts[i] += 1 }
				'0' => { counts[i] -= 1 }
				_ => unreachable!()
			}
		}
	}

	let gam = counts.iter().enumerate().fold(0u64, |mut accum, (i, &v)| {
		if v > 0 { accum += (1 << (N - i - 1)) as u64 }
		accum
	});
	let eps = !gam & ((1 << N) - 1);

	gam * eps
}

pub(crate) fn part1() -> u64 {
	part1_impl::<_, 12>(input_strs())
}


enum Part2Kind { Oxy, Co2 }

fn part2_kind(n: usize, mut input_nums: Vec<&str>, kind: Part2Kind) -> u64 {
	let mut count = 0i64;

	for input_num in input_nums.iter() {
		match input_num.chars().rev().nth(n - 1).unwrap() {
			'1' => { count += 1 }
			'0' => { count -= 1 }
			_ => unreachable!()
		}
	}
	let bit = match kind {
		Part2Kind::Oxy if count >= 0 => '1',
		Part2Kind::Oxy => '0',
		Part2Kind::Co2 if count < 0 => '1',
		Part2Kind::Co2 => '0',
	};

	input_nums.retain(move |input_num| input_num.chars().rev().nth(n - 1).unwrap() == bit);

	if input_nums.len() == 1 {
		u64::from_str_radix(input_nums[0],2).unwrap()
	} else if n > 1 {
		part2_kind(n - 1, input_nums, kind)
	} else {
		unreachable!()
	}
}

fn part2_impl<'a, I, const N: usize>(input_nums: I) -> u64
where I: Iterator<Item = &'a str> {
	let oxy_vec = input_nums.collect::<Vec<_>>();
	let co2_vec = oxy_vec.clone();

	let oxy = part2_kind(N, oxy_vec, Part2Kind::Oxy);
	let co2 = part2_kind(N, co2_vec, Part2Kind::Co2);

	oxy * co2
}

pub(crate) fn part2() -> u64 {
	part2_impl::<_, 12>(input_strs_from_str(include_str!("day03.txt")))
}


#[test]
fn tests() {
	const INPUT_NUMS: &str = indoc::indoc! { "
		00100
		11110
		10110
		10111
		10101
		01111
		00111
		11100
		10000
		11001
		00010
		01010
	" };
	assert_eq!(part1_impl::<_, 5>(input_strs_from_str(INPUT_NUMS)), 198);
	assert_eq!(part2_impl::<_, 5>(input_strs_from_str(INPUT_NUMS)), 230);
}
