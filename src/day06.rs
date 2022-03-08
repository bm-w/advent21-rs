// Copyright (c) 2022 Bastiaan Marinus van de Weerd


fn input_lanternfishes_from_str(s: &str) -> impl Iterator<Item = u8> + '_ {
	s.lines().next().unwrap()
		.split(',').map(|num| num.parse::<u8>().unwrap())
}

fn input_lanternfishes() -> impl Iterator<Item = u8> {
	input_lanternfishes_from_str(include_str!("day06.txt"))
}


#[allow(dead_code)]
fn part1_brute(lanternfishes: impl Iterator<Item = u8>, n: usize) -> usize {
	let mut lanternfishes = lanternfishes.collect::<Vec<_>>();

	for _ in 0..n {
		let mut new_count = 0;
		for lanternfish in &mut lanternfishes {
			if lanternfish == &0 {
				*lanternfish = 6;
				new_count += 1;
			} else {
				*lanternfish -= 1;
			}
		}
		for _ in 0..new_count {
			lanternfishes.push(8);
		}
	}
	lanternfishes.len()
}

fn part1and2_impl(lanternfishes: impl Iterator<Item = u8>, n_days: usize) -> usize {
	let mut breeders_per_day = [0usize; 9];

	for lanternfish in lanternfishes {
		breeders_per_day[lanternfish as usize] += 1;
	}

	for day in 0..n_days {
		// Babies born today ‘remain’ in today’s `breeders_per_day` index, so they’ll
		// start breeding in `breeders_per_day.len()` (9) days. Adults who bred today
		// are ‘moved’ 7 indices (days) ahead so that they’ll breed again then.
		let breeders = breeders_per_day[day % breeders_per_day.len()];
		let next_breeding_idx = (day + 7) % breeders_per_day.len();
		breeders_per_day[next_breeding_idx] += breeders;
	}

	breeders_per_day.into_iter().sum()
}

pub(crate) fn part1() -> usize {
	part1and2_impl(input_lanternfishes(), 80)
}

pub(crate) fn part2() -> usize {
	part1and2_impl(input_lanternfishes(), 256)
}


#[test]
fn tests() {
	const INPUT: &str = "3,4,3,1,2";
	assert_eq!(part1_brute(input_lanternfishes_from_str(INPUT), 80), 5934);
	assert_eq!(part1and2_impl(input_lanternfishes_from_str(INPUT), 80), 5934);
	assert_eq!(part1and2_impl(input_lanternfishes_from_str(INPUT), 256), 26984457539);
}
