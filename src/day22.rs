// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::{ops::RangeInclusive, collections::HashSet};
use itertools::iproduct;


#[cfg_attr(test, derive(Debug))]
struct Step {
	on: bool,
	cuboid_ranges: [RangeInclusive<i32>; 3],
}

impl Step {
	fn cuboid_fully_inside_50x_unit_region(&self) -> bool {
		self.cuboid_ranges.iter()
			.all(|r| r.start().abs().max(r.end().abs()) <= 50)
	}
}


trait Intersection: Sized {
	fn intersection(&self, other: &Self) -> Option<Self>;
}

impl Intersection for RangeInclusive<i32> {
	fn intersection(&self, other: &Self) -> Option<Self> {
		if self.start() > other.end() || self.end() < other.start() { None }
		else { Some(*self.start().max(other.start())..=*self.end().min(other.end())) }
	}
}

impl Intersection for [RangeInclusive<i32>; 3] {
	fn intersection(&self, other: &Self) -> Option<Self> {
		let mut it = Iterator::zip(self.iter(), other.iter())
			.map(|(s, o)| s.intersection(o))
			.take_while(|r| r.is_some())
			.filter_map(|r| r);
		// TODO(bm-w): Collect into array (`util::cast` doesn’t work because of `Default` bound…)?
		if let (Some(r0), Some(r1), Some(r2)) = (it.next(), it.next(), it.next()) { Some([r0, r1, r2]) }
		else { None }
	}
}


trait Volume {
	fn volume(&self) -> usize;
}

impl Volume for RangeInclusive<i32> {
	fn volume(&self) -> usize {
		(*self.end() - *self.start()) as usize + 1
	}
}

impl Volume for [RangeInclusive<i32>; 3] {
	fn volume(&self) -> usize {
		self.iter().map(|r| r.volume()).product()
	}
}


fn input_steps_from_str(s: &str) -> Vec<Step> {
	parsing::steps_from_str(s).unwrap()
}

fn input_steps() -> Vec<Step> {
	input_steps_from_str(include_str!("day22.txt"))
}


#[allow(dead_code)]
fn part1_brute(input_steps: Vec<Step>) -> usize {
	let mut on = HashSet::new();
	for step in input_steps {
		if !step.cuboid_fully_inside_50x_unit_region() { continue }
		for cube in iproduct!(
			step.cuboid_ranges[0].clone(),
			step.cuboid_ranges[1].clone(),
			step.cuboid_ranges[2].clone()
		) {
			if step.on { on.insert(cube); }
			else { on.remove(&cube); }
		}
	}
	on.len()
}

fn part1and2_impl(input_steps: Vec<Step>, discard_cuboid_partly_outside_50x_unit_region: bool) -> usize {
	let mut cuboid_parts = {
		let steps_len = input_steps.len();
		Vec::with_capacity(steps_len * (steps_len + 1) / 2)
	};

	for step in input_steps {
		if discard_cuboid_partly_outside_50x_unit_region && !step.cuboid_fully_inside_50x_unit_region() { continue }

		for j in 0..cuboid_parts.len() {
			let &(prev_cuboid_part_on, ref prev_cuboid_part_ranges) = &cuboid_parts[j];
			if let Some(intersection) = step.cuboid_ranges.intersection(prev_cuboid_part_ranges) {
				cuboid_parts.push((!(prev_cuboid_part_on as bool), intersection));
			}
		}
		if step.on { cuboid_parts.push((true, step.cuboid_ranges)); }
	}

	cuboid_parts.into_iter()
		.map(|(on, ranges)|
			if on { 1 } else { -1 } * ranges.volume() as i64)
		.sum::<i64>() as usize
}

pub(crate) fn part1() -> usize {
	part1and2_impl(input_steps(), true)
}

pub(crate) fn part2() -> usize {
	part1and2_impl(input_steps(), false)
}


mod parsing {
	use std::{str::FromStr, ops::RangeInclusive, num::ParseIntError};
	use crate::util::cast::Cast as _;
	use super::Step;

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum RangeError {
		InvalidFormat,
		InvalidFrom(ParseIntError),
		InvalidThrough { column: usize, source: ParseIntError },
		Inverted,
	}

	fn range_from_str(s: &str) -> Result<RangeInclusive<i32>, RangeError> {
		use RangeError::*;
		let (from, through) = s.split_once("..")
			.ok_or(InvalidFormat)?;
		let through_column = from.len() + 3;
		let from = from.parse().map_err(|e| InvalidFrom(e))?;
		let through = through.parse().map_err(|e| InvalidThrough { column: through_column, source: e })?;
		if through < from { return Err(Inverted) }
		Ok(from..=through)
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum CuboidRangeError {
		InvalidFormat,
		InvalidRange(RangeError),
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum StepError {
		InvalidFormat { column: usize },
		InvalidOn { found: String },
		InvalidCuboid { range_name: char, column: usize, source: CuboidRangeError },
	}

	impl FromStr for Step {
		type Err = StepError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use StepError::*;

			let (on, cuboid) = s.split_once(' ')
				.ok_or(InvalidFormat { column: 1 })?;
			let (on, cuboid_column) = match on {
				"on" => Ok((true, 4)),
				"off" => Ok((false, 5)),
				on => Err(InvalidOn { found: on.to_owned() }),
			}?;

			let [x, y, z]: [&str; 3] = cuboid.split(",").cast()
				.map_err(|_| InvalidFormat { column: cuboid_column })?;
			let y_column = cuboid_column + x.len() + 1;
			let z_column = y_column + y.len() + 1;
			[(cuboid_column, x), (y_column, y), (z_column, z)]
				.into_iter().zip("xyz".chars())
				.filter(|((_, r), c)| !r.starts_with(&format!("{c}=")))
				.map(|((c, _), n)| Err(InvalidCuboid { range_name: n, column: c, source: CuboidRangeError::InvalidFormat }))
				.next().unwrap_or(Ok(()))?;
			let x = range_from_str(&x[2..]).map_err(|e| InvalidCuboid
				{ range_name: 'x', column: cuboid_column + 2, source: CuboidRangeError::InvalidRange(e) })?;
			let y = range_from_str(&y[2..]).map_err(|e| InvalidCuboid
				{ range_name: 'y', column: y_column + 2, source: CuboidRangeError::InvalidRange(e) })?;
			let z = range_from_str(&z[2..]).map_err(|e| InvalidCuboid
				{ range_name: 'z', column: z_column + 2, source: CuboidRangeError::InvalidRange(e) })?;

			Ok(Step { on, cuboid_ranges: [x, y, z] })
		}
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum StepsError {
		InvalidStep { line: usize, source: StepError },
	}

	pub(super) fn steps_from_str(s: &str) -> Result<Vec<Step>, StepsError> {
		use StepsError::*;
		s.lines()
			.enumerate()
			.map(|(l, line)| line.parse()
				.map_err(|e| InvalidStep { line: l + 1, source: e }))
			.collect::<Result<Vec<_>, _>>()
	}


	#[test]
	fn range() {
		use RangeError::*;
		assert!(matches!(range_from_str("foo"), Err(InvalidFormat)));
		assert!(matches!(range_from_str("foo..bar"), Err(InvalidFrom(_))));
		assert!(matches!(range_from_str("13..bar"), Err(InvalidThrough { column: 5, source: _ })));
		assert!(matches!(range_from_str("37..13"), Err(Inverted)));
		assert!(matches!(range_from_str("13..37"), Ok(r) if r.start() == &13 && r.end() == &37));
	}

	#[test]
	fn step() {
		use StepError::*;
		assert!(matches!("foo".parse::<Step>(), Err(InvalidFormat { column: 1 })));
		assert!(matches!("foo bar".parse::<Step>(), Err(InvalidOn { found }) if matches!(found.as_str(), "foo")));
		assert!(matches!("on foo".parse::<Step>(), Err(InvalidFormat { column: 4 })));
		assert!(matches!("on foo,bar,daz".parse::<Step>(), Err(InvalidCuboid { range_name: 'x', column: 4, source: CuboidRangeError::InvalidFormat })));
		assert!(matches!("on x=foo,bar,daz".parse::<Step>(), Err(InvalidCuboid { range_name: 'y', column: 10, source: CuboidRangeError::InvalidFormat })));
		assert!(matches!("on x=foo,y=bar,daz".parse::<Step>(), Err(InvalidCuboid { range_name: 'z', column: 16, source: CuboidRangeError::InvalidFormat })));
		assert!(matches!("on x=foo,y=bar,z=daz".parse::<Step>(), Err(InvalidCuboid { range_name: 'x', column: 6, source: CuboidRangeError::InvalidRange(_) })));
		assert!(matches!("on x=1..3,y=bar,z=daz".parse::<Step>(), Err(InvalidCuboid { range_name: 'y', column: 13, source: CuboidRangeError::InvalidRange(_) })));
		assert!(matches!("on x=1..3,y=3..7,z=daz".parse::<Step>(), Err(InvalidCuboid { range_name: 'z', column: 20, source: CuboidRangeError::InvalidRange(_) })));
		assert!(matches!("on x=1..3,y=3..7,z=13..37".parse::<Step>(), Ok(Step { on: true, cuboid_ranges })
			if cuboid_ranges[0].start() == &1 && cuboid_ranges[0].end() == &3
			&& cuboid_ranges[1].start() == &3 && cuboid_ranges[1].end() == &7
			&& cuboid_ranges[2].start() == &13 && cuboid_ranges[2].end() == &37));
	}

	#[test]
	fn steps() -> Result<(), StepsError> {
		const INPUT: &str = indoc::indoc! { "
			on x=10..12,y=10..12,z=10..12
			on x=11..13,y=11..13,z=11..13
			off x=9..11,y=9..11,z=9..11
			on x=10..10,y=10..10,z=10..10
		" };
		assert_eq!(steps_from_str(INPUT).map(|ss| ss.len())?, 4);
		Ok(())
	}
}


#[test]
fn tests() {
	const INPUT_PART1: &str = indoc::indoc! { "
		on x=-20..26,y=-36..17,z=-47..7
		on x=-20..33,y=-21..23,z=-26..28
		on x=-22..28,y=-29..23,z=-38..16
		on x=-46..7,y=-6..46,z=-50..-1
		on x=-49..1,y=-3..46,z=-24..28
		on x=2..47,y=-22..22,z=-23..27
		on x=-27..23,y=-28..26,z=-21..29
		on x=-39..5,y=-6..47,z=-3..44
		on x=-30..21,y=-8..43,z=-13..34
		on x=-22..26,y=-27..20,z=-29..19
		off x=-48..-32,y=26..41,z=-47..-37
		on x=-12..35,y=6..50,z=-50..-2
		off x=-48..-32,y=-32..-16,z=-15..-5
		on x=-18..26,y=-33..15,z=-7..46
		off x=-40..-22,y=-38..-28,z=23..41
		on x=-16..35,y=-41..10,z=-47..6
		off x=-32..-23,y=11..30,z=-14..3
		on x=-49..-5,y=-3..45,z=-29..18
		off x=18..30,y=-20..-8,z=-3..13
		on x=-41..9,y=-7..43,z=-33..15
		on x=-54112..-39298,y=-85059..-49293,z=-27449..7877
		on x=967..23432,y=45373..81175,z=27513..53682
	" };
	assert_eq!(part1_brute(input_steps_from_str(INPUT_PART1)), 590784);
	assert_eq!(part1and2_impl(input_steps_from_str(INPUT_PART1), true), 590784);
	assert_eq!(part1(), 580098);

	const INPUT_PART2: &str = indoc::indoc! { "
		on x=-5..47,y=-31..22,z=-19..33
		on x=-44..5,y=-27..21,z=-14..35
		on x=-49..-1,y=-11..42,z=-10..38
		on x=-20..34,y=-40..6,z=-44..1
		off x=26..39,y=40..50,z=-2..11
		on x=-41..5,y=-41..6,z=-36..8
		off x=-43..-33,y=-45..-28,z=7..25
		on x=-33..15,y=-32..19,z=-34..11
		off x=35..47,y=-46..-34,z=-11..5
		on x=-14..36,y=-6..44,z=-16..29
		on x=-57795..-6158,y=29564..72030,z=20435..90618
		on x=36731..105352,y=-21140..28532,z=16094..90401
		on x=30999..107136,y=-53464..15513,z=8553..71215
		on x=13528..83982,y=-99403..-27377,z=-24141..23996
		on x=-72682..-12347,y=18159..111354,z=7391..80950
		on x=-1060..80757,y=-65301..-20884,z=-103788..-16709
		on x=-83015..-9461,y=-72160..-8347,z=-81239..-26856
		on x=-52752..22273,y=-49450..9096,z=54442..119054
		on x=-29982..40483,y=-108474..-28371,z=-24328..38471
		on x=-4958..62750,y=40422..118853,z=-7672..65583
		on x=55694..108686,y=-43367..46958,z=-26781..48729
		on x=-98497..-18186,y=-63569..3412,z=1232..88485
		on x=-726..56291,y=-62629..13224,z=18033..85226
		on x=-110886..-34664,y=-81338..-8658,z=8914..63723
		on x=-55829..24974,y=-16897..54165,z=-121762..-28058
		on x=-65152..-11147,y=22489..91432,z=-58782..1780
		on x=-120100..-32970,y=-46592..27473,z=-11695..61039
		on x=-18631..37533,y=-124565..-50804,z=-35667..28308
		on x=-57817..18248,y=49321..117703,z=5745..55881
		on x=14781..98692,y=-1341..70827,z=15753..70151
		on x=-34419..55919,y=-19626..40991,z=39015..114138
		on x=-60785..11593,y=-56135..2999,z=-95368..-26915
		on x=-32178..58085,y=17647..101866,z=-91405..-8878
		on x=-53655..12091,y=50097..105568,z=-75335..-4862
		on x=-111166..-40997,y=-71714..2688,z=5609..50954
		on x=-16602..70118,y=-98693..-44401,z=5197..76897
		on x=16383..101554,y=4615..83635,z=-44907..18747
		off x=-95822..-15171,y=-19987..48940,z=10804..104439
		on x=-89813..-14614,y=16069..88491,z=-3297..45228
		on x=41075..99376,y=-20427..49978,z=-52012..13762
		on x=-21330..50085,y=-17944..62733,z=-112280..-30197
		on x=-16478..35915,y=36008..118594,z=-7885..47086
		off x=-98156..-27851,y=-49952..43171,z=-99005..-8456
		off x=2032..69770,y=-71013..4824,z=7471..94418
		on x=43670..120875,y=-42068..12382,z=-24787..38892
		off x=37514..111226,y=-45862..25743,z=-16714..54663
		off x=25699..97951,y=-30668..59918,z=-15349..69697
		off x=-44271..17935,y=-9516..60759,z=49131..112598
		on x=-61695..-5813,y=40978..94975,z=8655..80240
		off x=-101086..-9439,y=-7088..67543,z=33935..83858
		off x=18020..114017,y=-48931..32606,z=21474..89843
		off x=-77139..10506,y=-89994..-18797,z=-80..59318
		off x=8476..79288,y=-75520..11602,z=-96624..-24783
		on x=-47488..-1262,y=24338..100707,z=16292..72967
		off x=-84341..13987,y=2429..92914,z=-90671..-1318
		off x=-37810..49457,y=-71013..-7894,z=-105357..-13188
		off x=-27365..46395,y=31009..98017,z=15428..76570
		off x=-70369..-16548,y=22648..78696,z=-1892..86821
		on x=-53470..21291,y=-120233..-33476,z=-44150..38147
		off x=-93533..-4276,y=-16170..68771,z=-104985..-24507
	" };
	assert_eq!(part1and2_impl(input_steps_from_str(INPUT_PART2), false), 2758514936282235);
	assert_eq!(part2(), 1134725012490723);
}
