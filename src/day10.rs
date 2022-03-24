// Copyright (c) 2022 Bastiaan Marinus van de Weerd


enum ChunkDelim { Open, Close }

#[derive(Debug, PartialEq, Eq)]
enum ChunkKind { Paren, Square, Curly, Angle }

impl ChunkKind {
	/// Returns corruption & completion scores.
	fn scores(&self) -> (u64, u64) {
		use ChunkKind::*;
		match self {
			Paren => (3, 1),
			Square => (57, 2),
			Curly => (1197, 3),
			Angle => (25137, 4),
		}
	}
}


fn input_lines_from_str(s: &str) -> impl Iterator<Item = &str> + '_ {
	s.lines()
}


fn part1_impl<'a>(input_lines: impl Iterator<Item = &'a str>) -> u64 {
	let mut score = 0;
	'lines: for line in input_lines {
		let mut stack = vec![];
		for chr in line.chars() {
			let (delim, kind) = char_to_chunk_tok(chr).unwrap();
			if matches!(delim, ChunkDelim::Open) {
				stack.push(kind);
			} else if !matches!(stack.last(), Some(k) if k == &kind) {
				if !stack.is_empty() { score += kind.scores().0; }
				continue 'lines;
			} else {
				stack.pop();
			}
		}
	}
	score
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_lines_from_str(include_str!("day10.txt")))
}


fn part2_impl<'a>(input_lines: impl Iterator<Item = &'a str>) -> u64 {
	let mut scores = input_lines.filter_map(|line| {
		let mut stack = vec![];
		for chr in line.chars() {
			let (delim, kind) = char_to_chunk_tok(chr).unwrap();
			if matches!(delim, ChunkDelim::Open) {
				stack.push(kind);
			} else if !matches!(stack.last(), Some(k) if k == &kind) {
				return None;
			} else {
				stack.pop();
			}
		}
		Some(stack.iter().rev().fold(0,|s, k| s * 5 + k.scores().1))
	}).collect::<Vec<_>>();
	scores.sort_unstable();
	scores[scores.len() / 2]
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_lines_from_str(include_str!("day10.txt")))
}


fn char_to_chunk_tok(c: char) -> Option<(ChunkDelim, ChunkKind)> {
	use {ChunkDelim::*, ChunkKind::*};
	match c {
		'(' => Some((Open, Paren)),
		')' => Some((Close, Paren)),
		'[' => Some((Open, Square)),
		']' => Some((Close, Square)),
		'{' => Some((Open, Curly)),
		'}' => Some((Close, Curly)),
		'<' => Some((Open, Angle)),
		'>' => Some((Close, Angle)),
		_ => None
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		[({(<(())[]>[[{[]{<()<>>
		[(()[<>])]({[<{<<[]>>(
		{([(<{}[<>[]}>{[]{[(<()>
		(((({<>}<{<{<>}{[]{[]{}
		[[<[([]))<([[{}[[()]]]
		[{[{({}]{}}([{[{{{}}([]
		{<[[]]>}<{[{[{[]{()[[[]
		[<(<(<(<{}))><([]([]()
		<{([([[(<>()){}]>(<<{{
		<{([{{}}[<[[[<>{}]]]>[]]
	" };
	assert_eq!(part1_impl(input_lines_from_str(INPUT)), 26397);
	assert_eq!(part1(), 464991);
	assert_eq!(part2_impl(input_lines_from_str(INPUT)), 288957);
	assert_eq!(part2(), 3662008566);
}
