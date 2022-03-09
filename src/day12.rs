// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{cmp::Ordering, collections::{HashSet, HashMap}, hash::{Hash, Hasher}};


#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
enum Cave<'a> {
	Start,
	Named(&'a str),
	End,
}

impl Cave<'_> {
	fn is_small(&self) -> bool {
		matches!(self, Cave::Named(n) if !n.contains(|c: char| !c.is_lowercase()))
	}
}

#[derive(Debug, Clone)]
struct Conn<'a>(Cave<'a>, Cave<'a>);

impl<'a> PartialEq for Conn<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 && self.1 == other.1
			|| self.0 == other.1 && self.1 == other.0
	}
}

impl<'a> Eq for Conn<'a> {}

impl<'a> Hash for Conn<'a> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		let (h0, h1) = match self.0.cmp(&self.1) {
			Ordering::Less => (&self.0, &self.1),
			_ => (&self.1, &self.0),
		};
		h0.hash(state);
		h1.hash(state);
	}
}

impl Conn<'_> {
	fn other_cave(&self, than_cave: &Cave) -> &Cave {
		if &self.0 == than_cave { &self.1 } else { &self.0 }
	}
}

#[derive(Debug)]
struct Caves<'a> {
	caves: HashMap<Cave<'a>, HashSet<Conn<'a>>>,
}


fn input_caves_from_str(s: &str) -> Caves {
	Caves::try_from(s).unwrap()
}

fn input_caves() -> Caves<'static> {
	input_caves_from_str(include_str!("day12.txt"))
}


// TODO(bm-w): Use e.g. macro to avoid repeating this huge `fn` signature?
type OtherCavesFn = for<'result, 'inner> fn(
	&'result Caves<'result>,
	&'inner Cave<'result>,
	&'inner Vec<(Vec<&'result Cave<'result>>, usize)>
) -> Vec<&'result Cave<'result>>;

fn part1and2_other_caves<'inner, 'result>(
	input_caves: &'result Caves<'result>,
	cave: &'inner Cave<'result>,
	path: &'inner Vec<(Vec<&'result Cave<'result>>, usize)>,
	always_allow_small_cave: bool,
) -> Vec<&'result Cave<'result>> {
	input_caves.caves[cave]
		.iter()
		.map(|conn| conn.other_cave(cave))
		.filter(|other_cave| other_cave != &&Cave::Start)
		.filter(|other_cave| !other_cave.is_small()
			|| always_allow_small_cave
			|| !path.iter().any(|p| &p.0[p.1] == other_cave))
		.collect::<Vec<_>>()
}

fn part1_other_caves<'inner, 'result>(
	input_caves: &'result Caves<'result>,
	cave: &'inner Cave<'result>,
	path: &'inner Vec<(Vec<&'result Cave<'result>>, usize)>
) -> Vec<&'result Cave<'result>> {
	part1and2_other_caves(input_caves, cave, path, false)
}

fn part2_other_caves<'inner, 'result>(
	input_caves: &'result Caves<'result>,
	cave: &'inner Cave<'result>,
	path: &'inner Vec<(Vec<&'result Cave<'result>>, usize)>
) -> Vec<&'result Cave<'result>> {
	part1and2_other_caves(input_caves, cave, path, {
		let path_small_caves = path.iter()
			.map(|p| p.0[p.1])
			.filter(|c| c.is_small());
		let small_caves = path_small_caves.clone().collect::<HashSet<_>>().len();
		path_small_caves.count() == small_caves
	})
}

fn part1and2_impl<'a, 'b>(input_caves: Caves<'a>, other_caves_fn: OtherCavesFn) -> usize {
	let mut paths = 0;
	let mut path = vec![(vec![&Cave::Start], 0)];
	loop {
		let (ref caves, to_visit_idx) = path.last_mut().unwrap();
		if *to_visit_idx != caves.len() {
			let cave = caves[*to_visit_idx];
			if cave == &Cave::End {
				paths += 1;
				*to_visit_idx += 1;
			} else {
				let to_visit_idx_ptr: *mut usize = to_visit_idx;
				let other_caves = other_caves_fn(&input_caves, cave, &path);
				if !other_caves.is_empty() {
					path.push((other_caves, 0));
				} else {
					// SAFETY: `path` was not modified.
					unsafe { *to_visit_idx_ptr += 1; }
				}
			}
		} else {
			path.pop();
			if let Some((_, idx)) = path.last_mut() {
				*idx += 1;
			} else {
				break
			}
		}
	}
	paths
}

fn part1_impl<'a>(input_caves: Caves<'a>) -> usize {
	part1and2_impl(input_caves, part1_other_caves)
}

pub(crate) fn part1() -> usize {
	part1_impl(input_caves())
}

fn part2_impl<'a>(input_caves: Caves<'a>) -> usize {
	part1and2_impl(input_caves, part2_other_caves)
}

pub(crate) fn part2() -> usize {
	part2_impl(input_caves())
}



#[allow(dead_code)]
#[derive(Debug)]
struct ParseCaveError<'a> {
	invalid_format: &'a str,
}

impl<'a> TryFrom<&'a str> for Cave<'a> {
	type Error = ParseCaveError<'a>;
	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		match s {
			"start" => Ok(Cave::Start),
			"end" => Ok(Cave::End),
			s if !s.contains(|c: char| !c.is_ascii_alphabetic()) => Ok(Cave::Named(s)),
			_ => Err(ParseCaveError { invalid_format: s}),
		}
	}
}

#[derive(Debug)]
enum ParseConnError<'a> {
	InvalidFormat(&'a str),
	InvalidFrom(ParseCaveError<'a>),
	InvalidTo(ParseCaveError<'a>),
}

impl<'a> TryFrom<&'a str> for Conn<'a> {
	type Error = ParseConnError<'a>;
	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		use ParseConnError::*;
		let (from, to) = s.split_once('-')
			.ok_or(InvalidFormat(s))?;
		let from = Cave::try_from(from)
			.map_err(|e| InvalidFrom(e))?;
		let to = Cave::try_from(to)
			.map_err(|e| InvalidTo(e))?;
		Ok(Conn(from, to))
	}
}

#[allow(dead_code)]
#[derive(Debug)]
enum ParseCavesError<'a> {
	InvalidConn { line: usize, source: ParseConnError<'a> },
}

impl<'a> TryFrom<&'a str> for Caves<'a> {
	type Error = ParseCavesError<'a>;
	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		use ParseCavesError::*;

		let conns = s.lines()
			.enumerate()
			.map(|(l, line)|
				Conn::try_from(line).map_err(|e| InvalidConn { line: l, source: e }))
			.collect::<Result<Vec<_>, _>>()?;
			
		let mut caves = HashMap::<Cave<'a>, HashSet<Conn<'a>>>::new();
		for conn in conns.iter() {
			caves.entry(conn.0.clone())
				.and_modify(|s| { s.insert(conn.clone()); })
				.or_insert_with(|| HashSet::from([conn.clone()]));
			caves.entry(conn.1.clone())
				.and_modify(|s| { s.insert(conn.clone()); })
				.or_insert_with(|| HashSet::from([conn.clone()]));
		}

		Ok(Caves { caves })
	}
}

#[test]
fn tests() {
	const TINY_INPUT: &str = indoc::indoc! { "
		start-A
		start-b
		A-c
		A-b
		b-d
		A-end
		b-end
	" };
	const SLIGHTLY_LARGER_INPUT: &str = indoc::indoc! { "
		dc-end
		HN-start
		start-kj
		dc-start
		dc-HN
		LN-dc
		HN-end
		kj-sa
		kj-HN
		kj-dc
	" };
	const EVEN_LARGER_INPUT: &str = indoc::indoc! { "
		fs-end
		he-DX
		fs-he
		start-DX
		pj-DX
		end-zg
		zg-sl
		zg-pj
		pj-he
		RW-he
		fs-DX
		pj-RW
		zg-RW
		start-pj
		he-WI
		zg-he
		pj-fs
		start-RW
	" };

	assert_eq!(part1_impl(input_caves_from_str(TINY_INPUT)), 10);
	assert_eq!(part1_impl(input_caves_from_str(SLIGHTLY_LARGER_INPUT)), 19);
	assert_eq!(part1_impl(input_caves_from_str(EVEN_LARGER_INPUT)), 226);
	assert_eq!(part1(), 3761);

	assert_eq!(part2_impl(input_caves_from_str(TINY_INPUT)), 36);
	assert_eq!(part2_impl(input_caves_from_str(SLIGHTLY_LARGER_INPUT)), 103);
	assert_eq!(part2_impl(input_caves_from_str(EVEN_LARGER_INPUT)), 3509);
	assert_eq!(part2(), 99138);
}
