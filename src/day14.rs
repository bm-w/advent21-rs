// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{str::FromStr, ops::Deref, collections::HashMap};
use itertools::Itertools;


const PART1_STEPS: usize = 10;
const PART2_STEPS: usize = 40;


struct Template<'a>(&'a [u8]);

impl<'a> Deref for Template<'a> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target { self.0 }
}

struct InsertRule {
	between: (u8, u8),
	insert: u8,
}

struct Formula<'a> {
	template: Template<'a>,
	insert_rules: HashMap<(u8, u8), u8>,
}

impl Formula<'_> {
	fn tick(&self, polymer: &mut Option<HashMap<(u8, u8), usize>>) -> usize {
		let polymer_ref = polymer.get_or_insert_with(|| {
			let mut polymer = HashMap::new();
			for (l, r) in self.template.iter().copied().tuple_windows() {
				polymer.entry((l, r)).and_modify(|c| *c += 1).or_insert(1);		
			}
			polymer
		});

		let mut new_polymer = polymer_ref.clone();
		let mut inserts = 0;
		for (&pair, &count) in polymer_ref.iter() {
			if count == 0 { continue }
			if let Some(&insert) = self.insert_rules.get(&pair) {
				let (a, b, c) =
					match (pair == (pair.0, insert), pair == (insert, pair.1)) {
						(true, true) | (true, false) => (false, false, true),
						(false, true) => (false, true, false),
						(false, false) => (true, true, true),
					};
				if a { new_polymer.entry(pair)
					.and_modify(|c| *c -= count); }
				if b { new_polymer.entry((pair.0, insert))
					.and_modify(|c| *c += count)
					.or_insert(count); }
				if c { new_polymer.entry((insert, pair.1))
					.and_modify(|c| *c += count)
					.or_insert(count); }
				inserts += count;
			}
		}
		*polymer = Some(new_polymer);
		inserts
	}
}


fn input_formula_from_str(s: &str) -> Formula {
	Formula::try_from(s).unwrap()
}

fn input_formula() -> Formula<'static> {
	input_formula_from_str(include_str!("day14.txt"))
}


fn part1and2_impl(input_formula: Formula, steps: usize) -> usize {
	let mut polymer = None;
	for _ in 0..steps {
		input_formula.tick(&mut polymer);
	}
	if let Some(polymer) = polymer {
		let mut el_counts = HashMap::new();
		for (pair, count) in polymer {
			el_counts.entry(pair.0).and_modify(|c| *c+= count).or_insert(count);
			el_counts.entry(pair.1).and_modify(|c| *c+= count).or_insert(count);
		}
		el_counts.entry(*input_formula.template.first().unwrap()).and_modify(|c| *c += 2);
		el_counts.entry(*input_formula.template.last().unwrap()).and_modify(|c| *c += 2);
		
		let most_el = el_counts.values().max().unwrap() / 2;
		let least_el = el_counts.values().min().unwrap() / 2;

		most_el - least_el
	} else {
		0
	}
}

pub(crate) fn part1() -> usize {
	part1and2_impl(input_formula(), PART1_STEPS)
}

pub(crate) fn part2() -> usize {
	part1and2_impl(input_formula(), PART2_STEPS)
}


impl<'a> From<&'a str> for Template<'a> {
	fn from(s: &'a str) -> Self {
		Template(s.as_bytes())
	}
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum ParseInsertRuleError {
	InvalidFormat(String),
	InvalidBetween(usize),
	InvalidInsert(usize),
}

impl FromStr for InsertRule {
	type Err = ParseInsertRuleError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use ParseInsertRuleError::*;
		let (between, insert) = s.split_once("->")
			.ok_or_else(|| InvalidFormat(s.to_owned()))?;
		let (between, insert) = (between.trim().as_bytes(), insert.trim().as_bytes());
		if between.len() != 2 { return Err(InvalidBetween(between.len())) }
		if insert.len() != 1 { return Err(InvalidInsert(insert.len())) }
		Ok(InsertRule { between: (between[0], between[1]), insert: insert[0] })
	}
}

#[allow(dead_code)]
#[derive(Debug)]
enum ParseFormulaError {
	InvalidFormat { line: usize },
	InvalidInsertRule { line: usize, source: ParseInsertRuleError },
}

impl<'a> TryFrom<&'a str> for Formula<'a> {
	type Error = ParseFormulaError;

	fn try_from(s: &'a str) -> Result<Self, Self::Error> {
		use ParseFormulaError::*;
		let mut lines = s.lines().enumerate();

		let template = lines.by_ref().next()
			.map(|(_, line)| Template::from(line))
			.ok_or(InvalidFormat { line: 1 })?;

		if !matches!(lines.by_ref().next(), Some((_, ""))) {
			return Err(InvalidFormat { line: 2 })
		}

		let insert_rules = lines
			.map(|(l, line)| line.parse::<InsertRule>()
				.map_err(|e| InvalidInsertRule { line: l + 1, source: e }))
			.map_ok(|ir| (ir.between, ir.insert))
			.collect::<Result<HashMap<_, _>, _>>()?;

		Ok(Formula { template, insert_rules })
	}
}


#[test]
fn tests() {
	const INPUT: &str = indoc::indoc! { "
		NNCB

		CH -> B
		HH -> N
		CB -> H
		NH -> C
		HB -> C
		HC -> B
		HN -> C
		NN -> C
		BH -> H
		NC -> B
		NB -> B
		BN -> B
		BB -> N
		BC -> B
		CC -> N
		CN -> C
	" };
	assert_eq!(part1and2_impl(input_formula_from_str(INPUT), PART1_STEPS), 1588);
	assert_eq!(part1(), 3555);
	assert_eq!(part1and2_impl(input_formula_from_str(INPUT), PART2_STEPS), 2188189693529);
	assert_eq!(part2(), 4439442043739);
}
