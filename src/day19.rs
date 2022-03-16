// Copyright (c) 2022 Bastiaan Marinus van de Weerd


#[derive(Debug, PartialEq, Eq, Hash)]
struct Pos([i32; 3]);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Component { First, Second, Third }

#[derive(Debug)]
struct Scanner {
	#[allow(dead_code)]
	id: usize,
	beacons: Vec<Pos>,
}


mod parsing {
	use std::{iter, num::ParseIntError, str::FromStr};
	use super::{Pos, Component, Scanner};


	#[allow(dead_code)]
	#[derive(Debug)]
	pub(crate) enum PosError {
		InvalidFormat { column: usize },
		InvalidComponent { component: Component, source: ParseIntError },
	}


	impl FromStr for Pos {
		type Err = PosError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use {PosError::*, Component::*};
			let (x, yz) = s.split_once(",")
				.ok_or(InvalidFormat { column: 1 })?;
			let (y, z) = yz.split_once(",")
				.ok_or(InvalidFormat { column: x.len() + 2 })?;
			let x = x.parse().map_err(|e| InvalidComponent { component: First, source: e })?;
			let y = y.parse().map_err(|e| InvalidComponent { component: Second, source: e })?;
			let z = z.parse().map_err(|e| InvalidComponent { component: Third, source: e })?;
			Ok(Pos([x, y, z]))
		}
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(crate) enum ScannerError {
		InvalidFormat { line: usize, column: usize },
		InvalidId { line: usize, column: usize, source: ParseIntError },
		InvalidBeacon { line: usize, source: PosError },
	}

	// TODO(bm-w): Remove the tuple (but something about conflicting implementations)
	impl<'a, I> TryFrom<(usize, I)> for Scanner 
	where I: Iterator<Item = &'a str> {
		type Error = ScannerError;
		fn try_from((line_offset, mut lines): (usize, I)) -> Result<Self, Self::Error> {
			use ScannerError::*;

			let header = lines.next().unwrap_or("");
			const HEADER_PREFIX: &str = "--- scanner ";
			const HEADER_SUFFIX: &str = " ---";
			if !header.starts_with(HEADER_PREFIX) {
				let c = header.chars().zip(HEADER_PREFIX.chars()).take_while(|(l, r)| l == r).count();
				return Err(InvalidFormat { line: line_offset, column: c + 1 })
			}
			let suffix_start = header.len() - HEADER_SUFFIX.len();
			if !header.ends_with(HEADER_SUFFIX) {
				let c = header[suffix_start..].chars().zip(HEADER_SUFFIX.chars()).take_while(|(l, r)| l == r).count();
				return Err(InvalidFormat { line: line_offset, column: suffix_start + c + 1 })
			}
			let id = header[HEADER_PREFIX.len()..suffix_start].parse().map_err(|e|
				InvalidId { line: line_offset, column: HEADER_PREFIX.len() + 1, source: e })?;

			let beacons = lines
				.take_while(|line| !line.is_empty())
				.enumerate()
				.map(|(l, line)| line.parse()
					.map_err(|e| InvalidBeacon { line: line_offset + 1 + l, source: e }))
				.collect::<Result<Vec<_>, _>>()?;
			if beacons.is_empty() {
				return Err(InvalidFormat { line: line_offset + 1, column: 1 })
			}

			Ok(Scanner { id, beacons })
		}
	}

	#[cfg(test)]
	impl FromStr for Scanner {
		type Err = ScannerError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			Scanner::try_from((1, s.lines()))
		}
	}

	pub(super) fn try_scanners_from_str(s: &str) -> Result<Vec<Scanner>, ScannerError> {
		let mut lines = s.lines();
		let mut scanners = Vec::new();
		let mut line_offset = 1;
		loop {
			let scanner_line_offset = line_offset;
			let lines = if let Some(line) = lines.next() {
				iter::once(line).chain(lines.by_ref()).inspect(|_| line_offset += 1)
			} else {
				break
			};
			match Scanner::try_from((scanner_line_offset, lines)) {
				Ok(scanner) => scanners.push(scanner),
				Err(err) => return Err(err)
			};
		}
		Ok(scanners)
	}


	#[test]
	fn pos() {
		use {PosError::*, Component::*};
		assert!(matches!("foo".parse::<Pos>(), Err(InvalidFormat { column: 1 })));
		assert!(matches!("foo,bar".parse::<Pos>(), Err(InvalidFormat { column: 5 })));
		assert!(matches!("foo,bar,daz".parse::<Pos>(), Err(InvalidComponent { component: First, .. })));
		assert!(matches!("-1337,bar,daz".parse::<Pos>(), Err(InvalidComponent { component: Second, .. })));
		assert!(matches!("-1337,0,daz".parse::<Pos>(), Err(InvalidComponent { component: Third, .. })));
		assert!(matches!("-1337,0,1337".parse(), Ok(Pos([-1337, 0, 1337]))));
	}

	#[test]
	fn scanner() {
		use ScannerError::*;
		assert!(matches!("".parse::<Scanner>(), Err(InvalidFormat { line: 1, column: 1 })));
		assert!(matches!("--- scanX".parse::<Scanner>(), Err(InvalidFormat { line: 1, column: 9 })));
		assert!(matches!("--- scanner X -X-".parse::<Scanner>(), Err(InvalidFormat { line: 1, column: 16 })));
		assert!(matches!("--- scanner X ---".parse::<Scanner>(), Err(InvalidId { line: 1, column: 13, .. })));
		assert!(matches!("--- scanner 1337 ---".parse::<Scanner>(), Err(InvalidFormat { line: 2, column: 1 })));
		assert!(matches!("--- scanner 1337 ---\nX".parse::<Scanner>(), Err(InvalidBeacon { line: 2, .. })));
		assert!(matches!("--- scanner 1337 ---\n-1337,0,1337".parse(), Ok(Scanner { id: 1337, beacons }) if beacons.len() == 1));
	}

	#[test]
	fn scanners() {
		assert!(matches!(try_scanners_from_str(indoc::indoc! { "
			--- scanner 1337 ---
			-1337,0,1337

			--- scanner 54321 ---
			4321,321,21
		" }), Ok(scanners) if scanners.len() == 2));
	}
}

mod analysis {
	use std::{collections::{HashMap, VecDeque, hash_map::Entry}, cell::Cell};
	use itertools::Itertools as _;
	use super::{Pos, Component, Scanner};

	const OVERLAP_COUNT: usize = 12;
	const OVERLAP_CONNECTIONS_COUNT: usize = OVERLAP_COUNT * (OVERLAP_COUNT - 1) / 2;


	impl Pos {
		fn square_distance(&self, to: &Self) -> u64 {
			let d0 = (to.0[0] - self.0[0]) as i64;
			let d1 = (to.0[1] - self.0[1]) as i64;
			let d2 = (to.0[2] - self.0[2]) as i64;
			(d0 * d0 + d1 * d1 + d2 * d2) as u64
		}
	}

	type PosFromTo<'a> = (&'a Pos, &'a Pos);
	type PosFromTos<'a> = Vec<PosFromTo<'a>>;
	type SquareDistances<'a> = HashMap<u64, PosFromTos<'a>>;

	struct AnalyzedScanner<'a> {
		scanner: &'a Scanner,
		square_distances: SquareDistances<'a>
	}

	type CommonSquareDistances<'a> = HashMap<u64, (&'a PosFromTos<'a>, &'a PosFromTos<'a>)>;
	type PosFromSquareDistanceTos<'a> = HashMap<&'a Pos, Vec<(u64, &'a Pos)>>;

	enum ComparisonStage<'a> {
		Pending,
		Potential { common_square_distances: CommonSquareDistances<'a> },
		Matched(geometry::Transform),// TODO: PosFromSquareDistanceTos<'a>, PosFromSquareDistanceTos<'a>),
		Rejected,
	}

	struct ScannersComparison<'a> {
		scanners: (&'a AnalyzedScanner<'a>, &'a AnalyzedScanner<'a>),
		_stage: Cell<ComparisonStage<'a>>,
	}


	impl<'a> From<&'a Scanner> for AnalyzedScanner<'a> {
		fn from(scanner: &'a Scanner) -> Self {
			AnalyzedScanner { scanner, square_distances: scanner.beacons.iter()
				.tuple_combinations()
				.map(|(from, to)| (from.square_distance(to), (from, to)))
				.into_group_map() }
		}
	}

	impl<'a> ScannersComparison<'a> {
		fn new(scanners: (&'a AnalyzedScanner<'a>, &'a AnalyzedScanner<'a>)) -> Self {
			ScannersComparison { scanners, _stage: Cell::new(ComparisonStage::Pending) }
		}

		fn _potential_common_square_distances(&'a self) -> Option<CommonSquareDistances<'a>> {
			let sqdd0 = &self.scanners.0.square_distances;
			Some(self.scanners.1.square_distances.iter()
				.filter_map(|(sqd, ftt1)|
					sqdd0.get(sqd).map(|ftt0| (*sqd, (ftt0, ftt1))))
				.collect::<CommonSquareDistances<'a>>())
				.filter(|csdd| csdd.len() >= OVERLAP_CONNECTIONS_COUNT)
		}

		fn is_potential(&'a self) -> bool {
			use ComparisonStage::*;
			match self._stage.replace(Rejected) {
				Pending => {
					self._potential_common_square_distances()
						.map(|common_square_distances|
							self._stage.set(Potential { common_square_distances }))
						.is_some()
				}
				s @ Potential { .. } | s @ Matched(_) => {
					self._stage.set(s);
					true
				}
				_ => false
			}
		}

		fn _max_connected_beacons(
			from_square_distance_tos: &PosFromSquareDistanceTos<'a>
		) -> Option<(PosFromSquareDistanceTos<'a>, u64)> {
			let ideal_conn_count = from_square_distance_tos.len();
			let mut max_conn: (usize, Option<(PosFromSquareDistanceTos<'a>, u64)>) = (0, None);
			for &start in from_square_distance_tos.keys() {
				if max_conn.1.as_ref()
					.map(|c| c.0.contains_key(start))
					.unwrap_or(false) { continue }
				let mut queue = VecDeque::from([start]);
				let mut conn = PosFromSquareDistanceTos::new();
				let mut total_sqd = 0;
				while let Some(from) = queue.pop_front() {
					let sqd_tos = match conn.entry(from) {
						Entry::Occupied(_) => continue,
						Entry::Vacant(entry) =>
							entry.insert(from_square_distance_tos[from].clone())
					};
					for &(sqd, to) in sqd_tos.iter() {
						total_sqd += sqd;
						queue.push_back(to);
					};
				}
				assert!(total_sqd % 2 == 0);
				total_sqd /= 2;
				let conn_len = conn.len();
				if conn_len == ideal_conn_count {
					return Some((conn, total_sqd))
				} else if conn_len > max_conn.0 {
					max_conn = (conn_len, Some((conn, total_sqd)));
				}
			}
			if max_conn.0 < OVERLAP_COUNT { None } else { max_conn.1 }
		}

		fn _matched_transform(&'a self, common_square_distances: CommonSquareDistances<'a>) -> Option<geometry::Transform> {
			let from_sqd_tos = common_square_distances.iter().fold(
				(PosFromSquareDistanceTos::new(), PosFromSquareDistanceTos::new()),
				|mut psqdd, (&sqd, &(ftt0, ftt1))| {
					for (from0, to0) in ftt0.iter().flat_map(|(f0, t0)| [(f0, t0), (t0, f0)].into_iter()) {
						psqdd.0.entry(&from0).or_default().push((sqd, to0));
					}
					for (from1, to1) in ftt1.iter().flat_map(|(f1, t1)| [(f1, t1), (t1, f1)].into_iter()) {
						psqdd.1.entry(from1).or_default().push((sqd, to1));
					}
					psqdd
				});

			Option::zip(
				Self::_max_connected_beacons(&from_sqd_tos.0),
				Self::_max_connected_beacons(&from_sqd_tos.1),
			).and_then(|(
				(from_sqd_tos0, _total_sqd0),
				(from_sqd_tos1, _total_sqd1),
			)| {
				//  1. Get some random beacon from `…0` and one of the squared distances it’s connected by
				//  2. Iterate over beacons from `…1` that are connected by the same squared distance (usually 2x, possibly 4x,6x…?)
				//      a. Iterate through orientations / rotations (6 * 4 = 24x) of `…1` relative to `…0`
				//          i. Iterate over other beacons from `…1`, transforming each into `…0` space and bailing if any aren’t in `…0` (11x)
				//  If no match, we probably bail early (2-4ish * 24 * 11 times).
				//  If match, we don’t need to keep iterating.
				from_sqd_tos0.iter().next()
					.map(|(&pos, sqd_tos)| (pos, sqd_tos[0].0 as u64))
					.into_iter()
					.flat_map(|(beacon, sqd)|
						common_square_distances[&sqd].1.iter()
							.flat_map(|(f, t)| [f, t].into_iter())
							.map(move |beacon1| (beacon, beacon1)))
					.cartesian_product(geometry::Orientation::all())
					.map(|((beacon0, beacon1), orientation)|
						geometry::Transform::from((beacon1, &orientation), &beacon0))
					.filter(|transform| {
						from_sqd_tos1.keys()
							.map(|beacon1| transform * *beacon1)
							.filter(|beacon1in0| from_sqd_tos0.contains_key(beacon1in0))
							.take(OVERLAP_COUNT)
							.count() == OVERLAP_COUNT
					})
					.next()
			})
		}

		fn matched_transform(&'a self) -> Option<geometry::Transform> {
			use ComparisonStage::*;
			if !self.is_potential() { return None }
			match self._stage.replace(Rejected) {
				Potential { common_square_distances } => {
					self._matched_transform(common_square_distances)
						// TODO(bm-w): `inspect`
						.map(|transform| {
							self._stage.set(Matched(transform.clone()));
							transform
						})
				}
				Matched(transform) => {
					self._stage.set(Matched(transform.clone()));
					Some(transform)
				}
				_ => unreachable!()
			}
		}
	}


	pub(super) mod geometry {
		use std::ops::{Index, Mul};
		use itertools::Itertools as _;
		use super::{Pos, Component};


		#[derive(Debug, PartialEq, Eq, Clone, Copy)]
		struct Axis {
			component: Component,
			positive: bool
		}

		#[derive(Debug, PartialEq, Eq, Clone)]
		/// Third axis follows from first two.
		pub(super) struct Orientation(Axis, Axis);

		#[derive(Debug, PartialEq, Eq, Clone)]
		pub struct Transform {
			orientation: Orientation,
			translation: [i32; 3],
		}


		impl Component {
			fn all() -> impl Iterator<Item = Component> + Clone {
				[Component::First, Component::Second, Component::Third].into_iter()
			}
		}

		impl Index<Component> for [i32; 3] {
			type Output = i32;
			fn index(&self, index: Component) -> &Self::Output {
				use Component::*;
				&self[match index { First => 0, Second => 1, _ => 2 }]
			}
		}

		impl Axis {
			fn from(components: [Option<bool>; 3]) -> Axis {
				use Component::*;
				match components {
					[Some(positive), None, None] => Axis { component: First, positive },
					[None, Some(positive), None] => Axis { component: Second, positive },
					[None, None, Some(positive)] => Axis { component: Third, positive },
					_ => panic!()
				}
			}

			fn all() -> impl Iterator<Item = Axis> + Clone {
				Component::all().cartesian_product([true, false].into_iter())
					.map(|(c, p)| Axis { component: c, positive: p })
			}

			fn sign(positive: bool) -> i32 {
				if positive { 1 } else { -1 }
			}

			fn cross(lhs: Axis, rhs: Axis) -> Axis {
				use Component::*;
				match (lhs.component, lhs.positive, rhs.component, rhs.positive) {
					(First, pl, Second, pr) => Axis { component: Third, positive: pl == pr },
					(First, pl, Third, pr) => Axis { component: Second, positive: pl != pr },
					(Second, pl, Third, pr) => Axis { component: First, positive: pl == pr },
					(Second, pl, First, pr) => Axis { component: Third, positive: pl != pr },
					(Third, pl, First, pr) => Axis { component: Second, positive: pl == pr },
					(Third, pl, Second, pr) => Axis { component: First, positive: pl != pr },
					_ => unreachable!(),
				}
			}
		}

		impl Mul<&Axis> for Axis {
			type Output = Option<bool>;
			fn mul(self, rhs: &Axis) -> Self::Output {
				use Component::*;
				match (self.component, rhs.component) {
					(First, First) | (Second, Second) | (Third, Third) => Some(self.positive == rhs.positive),
					_ => None
				} 
			}
		}

		impl Mul<&[i32; 3]> for Axis {
			type Output = i32;
			fn mul(self, rhs: &[i32; 3]) -> Self::Output {
				rhs[self.component] * Axis::sign(self.positive)
			}
		}

		impl Mul<&Pos> for Axis {
			type Output = i32;
			fn mul(self, rhs: &Pos) -> Self::Output {
				self * &rhs.0
			}
		}

		impl Orientation {
			pub(super) fn all() -> impl Iterator<Item = Orientation> + Clone {
				// TODO(bm-w): Tuple permutations to avoid `Vec` allocations?
				Axis::all().cartesian_product(Axis::all())
					.filter(|(a0, a1)| a0.component != a1.component)
					.map(|(a0, a1)| Orientation(a0, a1))
			}

			fn identity() -> Orientation {
				use Component::*;
				Orientation(Axis { component: First, positive: true }, Axis { component: Second, positive: true })
			}

			fn inverse(&self) -> Orientation {
				use Component::*;
				match (self.0.component, self.0.positive, self.1.component, self.1.positive) {
					(First, _, Second, _) =>
						self.clone(),
					(First, p0, Third, p1) =>
						Orientation(self.0, Axis { component: Third, positive: p0 != p1 }),
					(Second, p0, Third, p1) =>
						Orientation(Axis { component: Third, positive: p0 == p1 }, Axis { component: First, positive: p0 }),
					(Second, p0, First, p1) =>
						Orientation(Axis { component: Second, positive: p1 }, Axis { component: First, positive: p0 }),
					(Third, p0, First, p1) =>
						Orientation(Axis { component: Second, positive: p1 }, Axis { component: Third, positive: p0 == p1 }),
					(Third, p0, Second, p1) =>
						Orientation(Axis { component: Third, positive: p0 != p1 }, self.1),
					_ => unreachable!(),
				}
			}

			fn third_axis(&self) -> Axis {
				Axis::cross(self.0, self.1)
			}

			#[cfg(test)]
			fn determinant(&self) -> i32 {
				let sign = Axis::sign;
				let (p0, p1) = (self.0.positive, self.1.positive);
				sign(p0) * sign(p1) * sign(p0 == p1)
			}
		}

		impl Mul<&Pos> for &Orientation {
			type Output = Pos;
			fn mul(self, rhs: &Pos) -> Self::Output {
				Pos([self.0 * rhs, self.1 * rhs, self.third_axis() * rhs])
			}
		}

		impl Mul<&Orientation> for &Orientation {
			type Output = Orientation;
			fn mul(self, rhs: &Orientation) -> Self::Output {
				let rhs_inv = rhs.inverse();
				let rhs_inv_a2 = rhs_inv.third_axis();
				let a0 = Axis::from([self.0 * &rhs_inv.0, self.0 * &rhs_inv.1, self.0 * &rhs_inv_a2]);
				let a1 = Axis::from([self.1 * &rhs_inv.0, self.1 * &rhs_inv.1, self.1 * &rhs_inv_a2]);
				Orientation(a0, a1)
			}
		}

		impl Transform {
			/// Computes the transformation of positions in the source coordinate
			/// system (c.s.) to their corresponding positions in a target c.s.,
			/// assuming that the `source` position is in the source c.s. with
			/// `source` orientation relative to the target c.s. corresponds
			/// to the `pos_in_target` position in the target c.s.
			pub(super) fn from(source: (&Pos, &Orientation), pos_in_target: &Pos) -> Transform {
				let (pos_in_source, source_orientation) = source;
				let orientation = source_orientation.inverse();
				let pos_in_source_before_translation = &orientation * pos_in_source;
				let translation = [
					pos_in_target.0[0] - pos_in_source_before_translation.0[0],
					pos_in_target.0[1] - pos_in_source_before_translation.0[1],
					pos_in_target.0[2] - pos_in_source_before_translation.0[2],
				];
				Transform { orientation, translation }
			}

			pub(super) fn identity() -> Transform {
				Transform { orientation: Orientation::identity(), translation: [0, 0, 0] }
			}

			pub(super) fn inverse(&self) -> Transform {
				let orientation = self.orientation.inverse();
				let t = (&orientation * &Pos(self.translation)).0;
				Transform { orientation, translation: [-t[0], -t[1], -t[2]] }
			}
		}

		impl Mul<&Pos> for &Transform {
			type Output = Pos;
			fn mul(self, rhs: &Pos) -> Self::Output {
				let rot = (&self.orientation * rhs).0;
				Pos([rot[0] + self.translation[0], rot[1] + self.translation[1], rot[2] + self.translation[2]])
			}
		}

		impl Mul<&Transform> for &Transform {
			type Output = Transform;
			fn mul(self, rhs: &Transform) -> Self::Output {
				let orientation = &self.orientation * &rhs.orientation;
				let tr = rhs.translation;
				let tr = [
					self.orientation.0 * &tr,
					self.orientation.1 * &tr,
					self.orientation.third_axis() * &tr,
				];
				let t = self.translation;
				Transform { orientation, translation: [tr[0] + t[0], tr[1] + t[1], tr[2] + t[2]] }
			}
		}


		#[test]
		fn orientations() {
			use rand::Rng as _;
			let mut r = {
				let mut rng = rand::thread_rng();
				move || rng.gen_range(-100..=100)
			};
			assert_eq!(Axis::all().count(), 6);
			assert_eq!(Orientation::all().count(), 24);
			for (i, ori) in Orientation::all().enumerate() {
				assert_eq!(ori.determinant(), 1, "{i}: determinant of {ori:?}");
				let inv = ori.inverse();
				assert_eq!(inv.determinant(), 1, "{i}: determinant of {inv:?} (inverse of {ori:?})");
				assert_eq!(inv.inverse(), ori, "{i}: inverse of {inv:?} (inverse of {ori:?})");
				assert_eq!(&ori * &inv, Orientation::identity(), "{i}: {ori:?} times its inverse {inv:?}");
				for _ in 0..10 {
					let pos = Pos([r(), r(), r()]);
					assert_eq!(&inv * &(&ori * &pos), pos);
				}
			}
		}

		#[test]
		fn transforms() {
			use rand::Rng as _;
			let mut r = {
				let mut rng = rand::thread_rng();
				move || rng.gen_range(-100..=100)
			};
			for ori in Orientation::all() {
				let pos = (Pos([r(), r(), r()]), Pos([r(), r(), r()]));
				let transform = Transform::from((&pos.1, &ori), &pos.0);
				assert_eq!(&transform * &pos.1, pos.0);
				let inv = transform.inverse();
				assert_eq!(&inv * &pos.0, pos.1);
				assert_eq!(&transform * &inv, Transform::identity());
			}
		}
	}


	pub(super) fn match_scanners(scanners: &Vec<Scanner>, mut f: impl FnMut(&Scanner, &geometry::Transform)) {
		let transforms = {
			let ass = scanners.iter().map(|s| AnalyzedScanner::from(s)).collect::<Vec<_>>();
			ass.iter()
				.tuple_combinations()
				.filter_map(|(as0, as1)|
					ScannersComparison::new((&as0, &as1))
						.matched_transform()
						.map(|t| (as0.scanner, as1.scanner, t)))
				.flat_map(|(s0, s1, t)| [
					(s1 as *const _, (s0, t.inverse())),
					(s0 as *const _, (s1, t)),
				].into_iter())
				.into_group_map()
		};

		let mut queue = VecDeque::from([(&scanners[0], geometry::Transform::identity())]);
		let mut accum_transforms = HashMap::new();
		while let Some((s, accum_transform)) = queue.pop_front() {
			let key = s as *const _;
			let accum_transform = match accum_transforms.entry(key) {
				Entry::Occupied(_) => continue,
				Entry::Vacant(entry) => entry.insert(accum_transform) as &_,
			};
			f(s, accum_transform);
			for (scanner, transform) in transforms[&key].iter() {
				queue.push_back((scanner, accum_transform * transform));
			}
		}
	}
}


fn input_scanners_from_str(s: &str) -> Vec<Scanner> {
	parsing::try_scanners_from_str(s).unwrap()
}

fn input_scanners() -> Vec<Scanner> {
	input_scanners_from_str(include_str!("day19.txt"))
}


fn part1_impl(input_scanners: Vec<Scanner>) -> usize {
	use std::collections::HashSet;
	let mut all_beacons = HashSet::with_capacity(input_scanners.len() * input_scanners[0].beacons.len());
	analysis::match_scanners(&input_scanners, |scanner, transform| {
		all_beacons.extend(scanner.beacons.iter().map(|b| transform * b));
	});
	all_beacons.len()
}

pub(crate) fn part1() -> usize {
	part1_impl(input_scanners())
}


fn part2_impl(input_scanners: Vec<Scanner>) -> u32 {
	use std::collections::HashSet;
	use itertools::Itertools as _;
	let mut positions = HashSet::with_capacity(input_scanners.len());
	analysis::match_scanners(&input_scanners, |_, transform| {
		positions.insert(transform * &Pos([0, 0, 0]));
	});
	positions.iter().tuple_combinations().map(|(p0, p1)| {
		((p1.0[0] - p0.0[0]).abs() + (p1.0[1] - p0.0[1]).abs() + (p1.0[2] - p0.0[2]).abs()) as u32
	}).max().unwrap()
}

pub(crate) fn part2() -> u32 {
	part2_impl(input_scanners())
}


#[test]
fn tests() {
	const INPUT: &str = include_str!("day19_test.txt");
	assert_eq!(part1_impl(input_scanners_from_str(INPUT)), 79);
	assert_eq!(part1(), 398);
	assert_eq!(part2_impl(input_scanners_from_str(INPUT)), 3621);
	assert_eq!(part2(), 10965);
}
