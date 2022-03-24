// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::collections::HashSet;
use crate::util::cast::Cast as _;


fn input_entries_from_str(s: &str) -> impl Iterator<Item = ([&str; 10], [&str; 4])> + '_ {
	s.lines()
		.scan(None, |incomplete_entry_signal_patterns, line| {
			Some(if let Some(signal_patterns) = incomplete_entry_signal_patterns.take() {
				Some((signal_patterns, line))
			} else {
				let (signal_patterns, output_patterns) = line.split_once('|').unwrap();
				if output_patterns.is_empty() {
					*incomplete_entry_signal_patterns = Some(signal_patterns);
					None
				} else {
					Some((signal_patterns, output_patterns))
				}
			} )
		})
		.flatten()
		.map(|(s, o)| (
			s.trim().split_whitespace().cast().unwrap(),
			o.trim().split_whitespace().cast().unwrap(),
		))
}

fn input_entries() -> impl Iterator<Item = ([&'static str; 10], [&'static str; 4])> {
	input_entries_from_str(include_str!("day08.txt"))
}


/// For digits 1, 4, 7, & 8, resp.
const BASIC_PATTERN_LENS: [usize; 4] = [2, 4, 3, 7];

fn part1_impl<'a>(input_entries: impl Iterator<Item = ([&'a str; 10], [&'a str; 4])>) -> usize {
	let mut counts = [0usize; 4];
	for (_, output_patterns) in input_entries {
		for pattern in output_patterns.into_iter() {
			let pattern_len = pattern.len();
			for (i, &len) in BASIC_PATTERN_LENS.iter().enumerate() {
				if pattern_len == len {
					counts[i] += 1;
				}
			}
		}
	}
	counts.into_iter().sum()
}

pub(crate) fn part1() -> usize {
	part1_impl(input_entries())
}


fn part2_impl<'a>(input_entries: impl Iterator<Item = ([&'a str; 10], [&'a str; 4])>) -> u64 {
	let mut outputs_sum = 0;
	for (signal_patterns, output_patterns) in input_entries {
		let all_sets: [HashSet<char>; 10] = signal_patterns.into_iter()
			.map(|p| p.chars().collect::<HashSet<_>>())
			.cast().unwrap();

		let [one_idx, four_idx, seven_idx, eight_idx]: [usize; 4] =
			BASIC_PATTERN_LENS.iter().map(|&len|
				all_sets.iter().position(|p| p.len() == len).unwrap())
			.cast().unwrap();

		let (two_idx, three_idx, five_idx) = {
			let len5_idxs: [usize; 3] = all_sets.iter()
				.enumerate()
				.filter(|(_, s)| s.len() == 5)
				.map(|(i, _)| i)
				.cast().unwrap();
			let [two_idx, three_idx]: [usize; 2] =
				[(&all_sets[four_idx], 2), (&all_sets[seven_idx], 3)]
					.into_iter().map(|(s, n)|
						*len5_idxs.iter().find(|i|
							all_sets[**i].intersection(s).count() == n).unwrap())
					.cast().unwrap();
			let five_idx = len5_idxs.into_iter()
				.find(|&i| i != two_idx && i != three_idx)
				.unwrap();
			(two_idx, three_idx, five_idx)
		};

		let (zero_idx, six_idx, nine_idx) = {
			let len6_idxs: [usize; 3] = all_sets.iter()
				.enumerate()
				.filter(|(_, s)| s.len() == 6)
				.map(|(i, _)| i)
				.cast().unwrap();
			let [six_idx, nine_idx]: [usize; 2] =
				[(&all_sets[seven_idx], 2), (&all_sets[four_idx], 4)]
					.into_iter().map(|(s, n)|
						*len6_idxs.iter().find(|i|
							all_sets[**i].intersection(s).count() == n).unwrap())
					.cast().unwrap();
			let zero_idx = len6_idxs.into_iter()
				.find(|&i| i != six_idx && i != nine_idx)
				.unwrap();
			(zero_idx, six_idx, nine_idx)
		};

		let all_idxs = [zero_idx, one_idx, two_idx, three_idx,
			four_idx, five_idx, six_idx, seven_idx, eight_idx, nine_idx];

		outputs_sum += output_patterns.into_iter()
			.enumerate()
			.map(|(i, p)| {
				let set = p.chars().collect::<HashSet<_>>();
				for (digit, set_idx) in all_idxs.iter().enumerate() {
					if set == all_sets[*set_idx] {
						return digit as u64 * 10u64.pow(3 - i as u32);
					}
				}
				unreachable!()
			})
			.sum::<u64>();
	}
	outputs_sum
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_entries())
}


#[test]
fn tests() {
	const SINGLE_INPUT: &str = indoc::indoc! { "
		acedgfb cdfbe gcdfa fbcad dab cefabd cdfgeb eafb cagedb ab |
		cdfeb fcadb cdfeb cdbaf
	" };
	const MULTI_INPUT: &str = indoc::indoc! { "
		be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb |
		fdgacbe cefdb cefbgd gcbe
		edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec |
		fcgedb cgb dgebacf gc
		fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef |
		cg cg fdcagb cbg
		fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega |
		efabcd cedba gadfec cb
		aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga |
		gecf egdcabf bgf bfgea
		fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf |
		gebdcfa ecba ca fadegcb
		dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf |
		cefg dcbef fcge gbcadfe
		bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd |
		ed bcgafe cdgba cbgef
		egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg |
		gbdfcae bgc cg cgb
		gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc |
		fgae cfgab fg bagce
	" };
	assert_eq!(part1_impl(input_entries_from_str(SINGLE_INPUT)), 0);
	assert_eq!(part1_impl(input_entries_from_str(MULTI_INPUT)), 26);
	assert_eq!(part2_impl(input_entries_from_str(SINGLE_INPUT)), 5353);
	assert_eq!(part2_impl(input_entries_from_str(MULTI_INPUT)), 61229);
}
