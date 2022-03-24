// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::ops::Range;

const BURROW_FOLDED: &str = indoc::indoc! { "
	#############
	#...........#
	###a#b#c#d###
	  #a#b#c#d#
	  #########
" };
const BURROW_FOLDED_NUM_SPACES: usize = 19;
const BURROW_UNFOLDED_NUM_SPACES: usize = 27;
const BURROW_FOLDED_NUM_AMPHIPODS: usize = 8;
const BURROW_UNFOLDED_NUM_AMPHIPODS: usize = 16;
const BURROW_ALL_LOCS: [[usize; 2]; BURROW_UNFOLDED_NUM_SPACES] = [
	[1, 1], [2, 1], [3, 1], [4, 1], [5, 1], [6, 1], [7, 1], [8, 1], [9, 1], [10, 1], [11, 1],
	                [3, 2],         [5, 2],         [7, 2],         [9, 2],
	                [3, 3],         [5, 3],         [7, 3],         [9, 3],
	                [3, 4],         [5, 4],         [7, 4],         [9, 4],
	                [3, 5],         [5, 5],         [7, 5],         [9, 5]];
const BURROW_SPACES_IN_FOLD: Range<usize> = 15..23;
const BURROW_AMPHIPOD_KINDS_IN_FOLD: [AmphipodKind; 8] = [
	AmphipodKind::Desert, AmphipodKind::Copper, AmphipodKind::Bronze, AmphipodKind::Amber,
	AmphipodKind::Desert, AmphipodKind::Bronze, AmphipodKind::Amber, AmphipodKind::Copper];
const BURROW_FOLDED_STEPS: [usize; 36] = [
	 1,  2,  0,  3, 11,  1,  4,  2, 5, 12, 3, 6, 4, 7, 13, 5, 8, 6, 9, 14, 7, 10, 8, 9,
	 2, 15,  4, 16,  6,  17, 8, 18,
	11, 12, 13, 14];
const BURROW_UNFOLDED_STEPS: [usize; 52] = [
	 1,  2,  0,  3, 11,  1,  4,  2, 5, 12, 3, 6, 4, 7, 13, 5, 8, 6, 9, 14, 7, 10, 8, 9,
	 2, 15,  4, 16,  6,  17, 8, 18,
	11, 19, 12, 20, 13, 21, 14, 22, 
	15, 23, 16, 24, 17, 25, 18, 26,
	19, 20, 21, 22];
const BURROW_FOLDED_SPACE_STEPS: [usize; BURROW_FOLDED_NUM_SPACES + 1] = [
	 0,  1,  3,  6,  8, 11, 13, 16, 18, 21, 23,
	24, 26, 28, 30,
	32, 33, 34, 35,
	36];
const BURROW_UNFOLDED_SPACE_STEPS: [usize; BURROW_UNFOLDED_NUM_SPACES + 1] = [
	 0,  1,  3,  6,  8, 11, 13, 16, 18, 21, 23,
	24, 26, 28, 30,
	32, 34, 36, 38,
	40, 42, 44, 46,
	48, 49, 50, 51,
	52];
const BURROW_MOVE_STEP_COUNTS: [[usize; BURROW_ALL_LOCS.len()]; BURROW_ALL_LOCS.len()] = [
	//                                           1A   B   C   D  2A   B   C   D  3A   B   C   D  4A   B   C   D
	[ 0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10,  3,  5,  7,  9,  4,  6,  8, 10,  5,  7,  9, 11,  6,  8, 10, 12],
	[ 1,  0,  1,  2,  3,  4,  5,  6,  7,  8,  9,  2,  4,  6,  8,  3,  5,  7,  9,  4,  6,  8, 10,  5,  7,  9, 11],
	[ 2,  1,  0,  1,  2,  3,  4,  5,  6,  7,  8,  1,  3,  5,  7,  2,  4,  6,  8,  3,  5,  7,  9,  4,  6,  8, 10],
	[ 3,  2,  1,  0,  1,  2,  3,  4,  5,  6,  7,  2,  2,  4,  6,  3,  3,  5,  7,  4,  4,  6,  8,  5,  5,  7,  9],
	[ 4,  3,  2,  1,  0,  1,  2,  3,  4,  5,  6,  3,  1,  3,  5,  4,  2,  4,  6,  5,  3,  5,  7,  6,  4,  6,  8],
	[ 5,  4,  3,  2,  1,  0,  1,  2,  3,  4,  5,  4,  2,  2,  4,  5,  3,  3,  5,  6,  4,  4,  6,  7,  5,  5,  7],
	[ 6,  5,  4,  3,  2,  1,  0,  1,  2,  3,  4,  5,  3,  1,  3,  6,  4,  2,  4,  7,  5,  3,  5,  8,  6,  4,  6],
	[ 7,  6,  5,  4,  3,  2,  1,  0,  1,  2,  3,  6,  4,  2,  2,  7,  5,  3,  3,  8,  6,  4,  4,  9,  7,  5,  5],
	[ 8,  7,  6,  5,  4,  3,  2,  1,  0,  1,  2,  7,  5,  3,  1,  8,  6,  4,  2,  9,  7,  5,  3, 10,  8,  6,  4],
	[ 9,  8,  7,  6,  5,  4,  3,  2,  1,  0,  1,  8,  6,  4,  2,  9,  7,  5,  3, 10,  8,  6,  4, 11,  9,  7,  5],
	[10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,  9,  7,  5,  3, 10,  8,  6,  4, 11,  8,  7,  5, 12, 10,  8,  6],
/**/[ 3,  2,  1,  2,  3,  4,  5,  6,  7,  8,  9,  0,  4,  6,  8,  1,  5,  7,  9,  2,  6,  8, 10,  3,  7,  9, 11],
	[ 5,  4,  3,  2,  1,  2,  3,  4,  5,  6,  7,  4,  0,  4,  6,  5,  1,  5,  7,  6,  2,  6,  8,  7,  3,  7,  9],
	[ 7,  6,  5,  4,  3,  2,  1,  2,  3,  4,  5,  6,  4,  0,  4,  7,  5,  1,  5,  9,  6,  2,  6,  9,  7,  3,  7],
	[ 9,  8,  7,  6,  5,  4,  3,  2,  1,  2,  3,  8,  6,  4,  0,  9,  7,  5,  1, 10, 10,  6,  2, 11,  9,  7,  3],
/**/[ 4,  3,  2,  3,  4,  5,  6,  7,  8,  9, 10,  1,  5,  7,  9,  0,  6,  8, 10,  1,  7,  9, 11,  2,  8, 10, 12],
	[ 6,  5,  4,  3,  2,  3,  4,  5,  6,  7,  8,  5,  1,  5,  7,  6,  0,  6,  8,  7,  1,  7,  9,  8,  2,  8, 10],
	[ 8,  7,  6,  5,  4,  3,  2,  3,  4,  5,  6,  7,  5,  1,  5,  8,  6,  0,  6,  9,  7,  1,  7, 10,  8,  2,  8], 
	[10,  9,  8,  7,  6,  5,  4,  3,  2,  3,  4,  9,  7,  5,  1, 10,  8,  6,  0, 11,  9,  7,  1, 12, 10,  8,  2],
/**/[ 5,  4,  3,  4,  5,  6,  7,  8,  9, 10, 11,  2,  6,  8, 10,  1,  7,  9, 11,  0,  8, 10, 12,  1,  9, 11, 13],
	[ 7,  6,  5,  4,  3,  4,  5,  6,  7,  8,  9,  6,  2,  6,  8,  7,  1,  7,  9,  8,  0,  8, 10,  9,  1,  9, 11],
	[ 9,  8,  7,  6,  5,  4,  3,  4,  5,  6,  7,  8,  6,  2,  6,  9,  7,  1,  7, 10,  8,  0,  8, 11,  9,  1,  9],
	[11, 10,  9,  8,  7,  6,  5,  4,  3,  4,  5, 10,  8,  8,  2, 11,  9,  7,  1, 12, 10,  8,  0, 13, 11,  9,  1],
/**/[ 6,  5,  4,  5,  6,  7,  8,  9, 10, 11, 12,  3,  7,  9, 11,  2,  8, 10, 12,  1,  9, 11, 13,  0, 10, 12, 14],
	[ 8,  7,  6,  5,  4,  5,  6,  7,  8,  9, 10,  7,  3,  7,  9,  8,  2,  8, 10,  9,  1,  9, 11, 10,  0, 10, 12],
	[10,  9,  8,  7,  6,  5,  4,  5,  6,  7,  8,  9,  7,  3,  7, 10,  8,  2,  8, 11,  9,  1,  9, 12, 10,  0, 10],
	[12, 11, 10,  9,  8,  7,  6,  5,  4,  5,  6, 11,  9,  7,  3, 12, 10,  8,  2, 13, 11,  9,  1, 14, 12, 10,  0]];
const BURROW_HALLWAY_SPACES: Range<usize> = 0..11;
const BURROW_OUTSIDE_SIDE_ROOM_SPACES: [usize; 4] = [2, 4, 6, 8];
const BURROW_TARGET_ROOMS: [(AmphipodKind, [usize; BURROW_UNFOLDED_NUM_AMPHIPODS / 4]); 4] = [
	(AmphipodKind::Amber, [11, 15, 19, 23]),
	(AmphipodKind::Bronze, [12, 16, 20, 24]),
	(AmphipodKind::Copper, [13, 17, 21, 25]),
	(AmphipodKind::Desert, [14, 18, 22, 26]),
];
const BURROW_FOLDED_TARGET_ROOM_SPACE_PERMUTATIONS: [[usize; BURROW_FOLDED_NUM_AMPHIPODS / 4]; 2] = [
	[0, 1], [1, 0]];
const BURROW_UNFOLDED_TARGET_ROOM_SPACE_PERMUTATIONS: [[usize; BURROW_UNFOLDED_NUM_AMPHIPODS / 4]; 24] = [
	[0, 1, 2, 3], [0, 2, 3, 1], [0, 3, 1, 2], [0, 1, 3, 2], [0, 3, 2, 1], [0, 2, 1, 3],
	[1, 2, 3, 0], [1, 3, 0, 2], [1, 0, 2, 3], [1, 2, 0, 3], [1, 0, 3, 2], [1, 3, 2, 0],
	[2, 3, 0, 1], [2, 0, 1, 3], [2, 1, 3, 0], [2, 3, 1, 0], [2, 1, 0, 3], [2, 0, 3, 1],
	[3, 0, 1, 2], [3, 1, 2, 0], [3, 2, 0, 1], [3, 0, 2, 1], [3, 2, 1, 0], [3, 1, 0, 2]];


#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
enum AmphipodKind { Amber, Bronze, Copper, Desert }

/// `.0` is the location’s index in `BURROW_LOCS`.
type BurrowAmphipods<const N: usize> = [(usize, AmphipodKind); N];

#[cfg_attr(test, derive(Debug))]
struct Burrow<const NS: usize, const NA: usize> {
	amphipods: BurrowAmphipods<NA>
}


mod moves {
	use std::{collections::{VecDeque, HashSet}, iter, rc::Rc};
	use itertools::Itertools;
	use super::{
		BURROW_FOLDED_NUM_SPACES,
		BURROW_ALL_LOCS,
		BURROW_FOLDED_STEPS, BURROW_UNFOLDED_STEPS,
		BURROW_FOLDED_SPACE_STEPS, BURROW_UNFOLDED_SPACE_STEPS,
		BURROW_HALLWAY_SPACES, BURROW_OUTSIDE_SIDE_ROOM_SPACES,
		BURROW_TARGET_ROOMS,
		AmphipodKind, Burrow};

	struct BurrowState<const NS: usize> {
		space_amphipods: [Option<AmphipodKind>; NS],
	}

	impl AmphipodKind {
		pub(super) fn step_cost(&self) -> u32 {
			use AmphipodKind::*;
			match self {
				Amber => 1,
				Bronze => 10,
				Copper => 100,
				Desert => 1000,
			}
		}
	}

	impl<const NS: usize, const NA: usize> Burrow<NS, NA> {
		fn state(&self) -> BurrowState<NS> {
			let mut space_amphipods = [None; NS];
			for (space, amphipod) in self.amphipods {
				space_amphipods[space] = Some(amphipod);
			}
			BurrowState { space_amphipods }
		}

		fn steps(from: usize) -> impl Iterator<Item = usize> {
			if NS == BURROW_FOLDED_NUM_SPACES {
				BURROW_FOLDED_STEPS[BURROW_FOLDED_SPACE_STEPS[from]..BURROW_FOLDED_SPACE_STEPS[from + 1]].iter().copied()
			} else {
				BURROW_UNFOLDED_STEPS[BURROW_UNFOLDED_SPACE_STEPS[from]..BURROW_UNFOLDED_SPACE_STEPS[from + 1]].iter().copied()
			}
		}

		fn in_filled_spaces_of_target_room(amphipod: AmphipodKind, at: usize, state: &BurrowState<NS>) -> bool {
			if BURROW_HALLWAY_SPACES.contains(&at) { return false }
			let &(target, ref spaces) =
				&BURROW_TARGET_ROOMS[(at - BURROW_HALLWAY_SPACES.end) % BURROW_TARGET_ROOMS.len()];
			if amphipod != target { return false }
			spaces[0..NA / 4].iter().rev().copied()
				.take_while(move |s| state.space_amphipods[*s] == Some(amphipod))
				.contains(&at)
		}

		fn entering_unavailable_target_room_from_hallway(amphipod: AmphipodKind, from: usize, to: usize, state: &BurrowState<NS>) -> bool {
			if !BURROW_HALLWAY_SPACES.contains(&from) || BURROW_HALLWAY_SPACES.contains(&to) { return false }
			let &(target, ref spaces) =
				&BURROW_TARGET_ROOMS[(to - BURROW_HALLWAY_SPACES.end) % BURROW_TARGET_ROOMS.len()];
			assert_eq!(to, spaces[0]);
			if amphipod != target { return true }
			spaces[0..NA / 4].iter().rev().copied()
				.skip_while(move |s| state.space_amphipods[*s] == Some(target))
				.any(|s| state.space_amphipods[s].is_some())
		}

		/// Assuming neither `initial_from` nor `to` in hallway
		fn same_target_room(initial_from: usize, to: usize) -> bool {
			BURROW_ALL_LOCS[initial_from][0] == BURROW_ALL_LOCS[to][0]
		}

		/// Assumes this is the right target room
		fn in_target_room_but_not_deepest(to: usize, state: &BurrowState<NS>) -> bool {
			if BURROW_HALLWAY_SPACES.contains(&to) { return false }
			let &(_, ref spaces) =
				&BURROW_TARGET_ROOMS[(to - BURROW_HALLWAY_SPACES.end) % BURROW_TARGET_ROOMS.len()];
			spaces[0..NA / 4].iter().rev().copied()
				.skip_while(move |s| state.space_amphipods[*s].is_some())
				.skip(1)
				.contains(&to)
		}

		fn amphipod_moves<'a>(&'a self, amphipod: usize, state: Option<&'_ Rc<BurrowState<NS>>>) -> impl Iterator<Item = (usize, u32)> + 'a {
			let a = amphipod;
			let (initial_from, amphipod) = self.amphipods[a];
			let initial_from_hallway = BURROW_HALLWAY_SPACES.contains(&initial_from);
			let step_cost = amphipod.step_cost();

			let state = state.cloned().unwrap_or_else(|| Rc::new(self.state()));

			let mut seen = HashSet::with_capacity(NS);
			let mut queue = VecDeque::from([(None, initial_from, 0)]);
			iter::from_fn(move || {
				while let Some((from, to, cost)) = queue.pop_front() {
					if !seen.insert(to) { continue }

					if to == initial_from && Self::in_filled_spaces_of_target_room(amphipod, to, state.as_ref()) {
						return None
					}

					if let Some(from) = from {
						// Can’t step into a space that already contains an amphipod
						if state.space_amphipods[to].is_some() {
							 continue
						}

						// Can’t step from hallway into side room that’s not this amphipod’s target room,
						// or that’s not the target room of any contains amphipods it already contains.
						if Self::entering_unavailable_target_room_from_hallway(amphipod, from, to, state.as_ref()) {
							continue
						}
					}

					// Enqueue next steps from this step
					{
						let from = to;
						queue.extend(Self::steps(from).map(|to| (Some(from), to, cost + step_cost)));
					}

					if to == initial_from { continue }

					let to_hallway = BURROW_HALLWAY_SPACES.contains(&to);

					// Don’t stop if initially in hallway and not moving into target room
					if initial_from_hallway && to_hallway { continue }

					// Don’t linger in the room where we started
					if !initial_from_hallway && !to_hallway && Self::same_target_room(initial_from, to) { continue }

					// Don’t stop right outside a side room
					if BURROW_OUTSIDE_SIDE_ROOM_SPACES.iter().contains(&to) { continue }

					// Don’t stop halfway into the target room
					if Self::in_target_room_but_not_deepest(to, state.as_ref()) { continue }

					return Some((to, cost))
				}
				None
			})
		}

		pub(super) fn all_moves(&self) -> impl Iterator<Item = (usize, usize, u32)> + '_ {
			let state = Rc::new(self.state());
			(0..self.amphipods.len())
				.flat_map(move |amphipod| self.amphipod_moves(amphipod, Some(&state))
					.map(move |(space, cost)| (amphipod, space, cost)))
		}
	}

	#[test]
	fn test() -> Result<(), super::parsing::BurrowError> {
		const NS: usize = BURROW_FOLDED_NUM_SPACES;
		const NA: usize = super::BURROW_FOLDED_NUM_AMPHIPODS;
		itertools::assert_equal(Burrow::<NS, NA>::steps(11), [2, 15]);
		let mut burrow = super::TEST_INPUT.parse::<Burrow<NS, NA>>()?;
		itertools::assert_equal(burrow.all_moves(), [
			(0, 3, 20), (0, 1, 20), (0, 0, 30), (0, 5, 40), (0, 7, 60), (0, 9, 80), (0, 10, 90),
			(1, 5, 200), (1, 3, 200), (1, 7, 400), (1, 1, 400), (1, 0, 500), (1, 9, 600), (1, 10, 700),
			(2, 7, 20), (2, 5, 20), (2, 9, 40), (2, 3, 40), (2, 10, 50), (2, 1, 60), (2, 0, 70),
			(3, 9, 2000), (3, 7, 2000), (3, 10, 3000), (3, 5, 4000), (3, 3, 6000), (3, 1, 8000), (3, 0, 9000)]);
		burrow.amphipods[2].0 = 3;
		itertools::assert_equal(burrow.amphipod_moves(2, None), []);
		itertools::assert_equal(burrow.amphipod_moves(1, None), [(5, 200), (7, 400), (13, 400), (9, 600), (10, 700)]);
		burrow.amphipods[1].0 = 13;
		itertools::assert_equal(burrow.amphipod_moves(1, None), []);
		itertools::assert_equal(burrow.amphipod_moves(6, None), []);
		itertools::assert_equal(burrow.amphipod_moves(2, None), []);
		itertools::assert_equal(burrow.amphipod_moves(5, None), [(5, 3000), (7, 5000), (9, 7000), (10, 8000)]);
		burrow.amphipods[5].0 = 5;
		itertools::assert_equal(burrow.amphipod_moves(5, None), []);
		itertools::assert_equal(burrow.amphipod_moves(2, None), [(16, 30)]);
		burrow.amphipods[2].0 = 16;
		itertools::assert_equal(burrow.amphipod_moves(0, None), [(3, 20), (1, 20), (0, 30), (12, 40)]);
		burrow.amphipods[0].0 = 12;
		// TODO(bm-w): `itertools::assert_equal(burrow.moves(2), []);`
		itertools::assert_equal(burrow.amphipod_moves(3, None), [(9, 2000), (7, 2000), (10, 3000)]);
		burrow.amphipods[3].0 = 7;
		itertools::assert_equal(burrow.amphipod_moves(3, None), []);
		itertools::assert_equal(burrow.amphipod_moves(7, None), [(9, 3), (10, 4)]);
		burrow.amphipods[7].0 = 9;
		itertools::assert_equal(burrow.amphipod_moves(7, None), []);
		itertools::assert_equal(burrow.amphipod_moves(3, None), [(18, 3000)]);
		burrow.amphipods[3].0 = 18;
		itertools::assert_equal(burrow.amphipod_moves(5, None), [(14, 4000)]);
		burrow.amphipods[5].0 = 14;
		itertools::assert_equal(burrow.amphipod_moves(3, None), []);
		itertools::assert_equal(burrow.amphipod_moves(5, None), []);
		itertools::assert_equal(burrow.amphipod_moves(7, None), [(11, 8)]);
		burrow.amphipods[7].0 = 11;
		itertools::assert_equal(burrow.all_moves(), []);
		Ok(())
	}
}


mod organization {
	use std::collections::{BinaryHeap, HashMap, hash_map::Entry};
	use super::{
		BURROW_FOLDED_NUM_AMPHIPODS,
		BURROW_TARGET_ROOMS, BURROW_MOVE_STEP_COUNTS,
		BURROW_FOLDED_TARGET_ROOM_SPACE_PERMUTATIONS, BURROW_UNFOLDED_TARGET_ROOM_SPACE_PERMUTATIONS,
		Burrow, BurrowAmphipods, AmphipodKind};

	#[cfg_attr(test, derive(Debug))]
	#[derive(PartialEq, Eq)]
	struct BurrowState<const NA: usize>{
		unorganizedness: Option<u32>,
		cost: u32,
		tiebreaker: usize,
		amphipods: BurrowAmphipods<NA>,
	}

	impl<const NA: usize> BurrowState<NA> {
		fn heuristic(&self) -> u32 {
			self.cost + self.unorganizedness.unwrap_or(0)
		}
	}

	impl<const NA: usize> PartialOrd for BurrowState<NA> {
		fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
			Some(self.cmp(other))
		}
	}

	impl<const NA: usize> Ord for BurrowState<NA> {
		fn cmp(&self, other: &Self) -> std::cmp::Ordering {
			// Inversely comparing cost so that lowest heuristic puts `self` at the top of a `BinaryHeap`
			other.heuristic().cmp(&self.heuristic())
				.then(self.tiebreaker.cmp(&other.tiebreaker))
		}
	}

	impl<const NA: usize> BurrowState<NA> {
		fn unorganizedness(amphipods: &BurrowAmphipods<NA>) -> Option<u32> {
			let amphipods = {
				let mut copied = *amphipods;
				copied.sort_by(|&(_, la), (_, ra)| la.cmp(ra));
				copied
			};

			let permutated_steps_count = |spaces: &[usize], amphipods: &[(usize, AmphipodKind)], a0: usize| {
				if NA == BURROW_FOLDED_NUM_AMPHIPODS {
					BURROW_FOLDED_TARGET_ROOM_SPACE_PERMUTATIONS.iter()
						.map(|permutated_idxs|
							permutated_idxs.iter().enumerate()
								.map(|(a, &s)| BURROW_MOVE_STEP_COUNTS[spaces[s]][amphipods[a0 + a].0])
								.sum::<usize>())
						.min().unwrap()
				} else {
					BURROW_UNFOLDED_TARGET_ROOM_SPACE_PERMUTATIONS.iter()
						.map(|permutated_idxs|
							permutated_idxs.iter().enumerate()
								.map(|(a, &s)| BURROW_MOVE_STEP_COUNTS[spaces[s]][amphipods[a0 + a].0])
								.sum::<usize>())
						.min().unwrap()
				}
			};

			// Basically underestimated hamming distance around the room entrances (considering step costs)
			let unorganizedness = BURROW_TARGET_ROOMS.iter().enumerate().map(|(r, (target, spaces))| {
				target.step_cost() * permutated_steps_count(&spaces[0..NA / 4], &amphipods[..], r * NA / 4) as u32
			}).sum();

			if unorganizedness > 0 { Some(unorganizedness) } else { None }
		}

		fn with_move(amphipods: &BurrowAmphipods<NA>, amphipod: usize, space: usize, cost: u32, tiebreaker: usize) -> BurrowState<NA> {
			let mut amphipods = *amphipods;
			amphipods[amphipod].0 = space;
			BurrowState { unorganizedness: BurrowState::unorganizedness(&amphipods), cost, tiebreaker, amphipods }
		}
	}

	impl<const NS: usize, const NA: usize> Burrow<NS, NA> {
		pub(super) fn organization_cost(&self) -> Option<u32> {
			let mut seen = HashMap::new();
			let mut tiebreaker = 0;
			let mut heap = BinaryHeap::from([BurrowState::<NA> {
				unorganizedness: BurrowState::unorganizedness(&self.amphipods),
				cost: 0,
				tiebreaker,
				amphipods: self.amphipods
			}]);

			while let Some(burrow_state) = heap.pop() {
				tiebreaker += 1;

				if burrow_state.unorganizedness.is_none() {
					return Some(burrow_state.cost)
				}

				if match seen.entry(burrow_state.amphipods) {
					Entry::Occupied(entry) => *entry.get() < burrow_state.heuristic(),
					Entry::Vacant(entry) => { entry.insert(burrow_state.heuristic()); false },
				} {
					continue
				}


				for (amphipod, space, add_cost) in (Burrow::<NS, NA> { amphipods: burrow_state.amphipods }).all_moves() {
					let next_state = BurrowState::with_move(&burrow_state.amphipods, amphipod, space,
						burrow_state.cost + add_cost, tiebreaker);
					let next_heuristic = next_state.heuristic();

					let (inserted, known_heuristic) = match seen.entry(next_state.amphipods) {
						Entry::Occupied(entry) => (false, entry.into_mut()),
						Entry::Vacant(entry) => (true, entry.insert(next_heuristic)),
					};
					if !inserted && *known_heuristic <= next_heuristic { continue }
					*known_heuristic = next_heuristic;

					heap.push(next_state);
				}
			}

			None
		}
	}

	#[test]
	fn test() -> Result<(), super::parsing::BurrowError> {
		use super::AmphipodKind::*;
		assert_eq!(BurrowState::unorganizedness(&[(11, Amber), (12, Bronze), (13, Copper), (14, Desert), (15, Amber), (16, Bronze), (17, Copper), (18, Desert)]), None);

		const NS: usize = super::BURROW_FOLDED_NUM_SPACES;
		const NA: usize = super::BURROW_FOLDED_NUM_AMPHIPODS;
		let mut burrow = super::TEST_INPUT.parse::<Burrow<NS, NA>>()?;
		// Just testing the last few steps (the full test is below in `day23::test`)
		for (amphipod, space) in [(2, 3), (1, 13), (5, 5), (2, 16), (0, 12), (3, 7), (7, 9), (3, 18)] {
			burrow.amphipods[amphipod].0 = space;
		}
		assert_eq!(burrow.organization_cost(), Some(4008));
		Ok(())
	}
}


fn input_burrow_from_str<const NS: usize, const NA: usize>(s: &str) -> Burrow<NS, NA> {
	s.parse().unwrap()
}

fn input_burrow<const NS: usize, const NA: usize>() -> Burrow<NS, NA> {
	input_burrow_from_str::<NS, NA>(include_str!("day23.txt"))
}


fn part1and2_impl<const NS: usize, const NA: usize>(input_burrow: Burrow<NS, NA>) -> u32 {
	input_burrow.organization_cost().unwrap()
}

pub(crate) fn part1() -> u32 {
	const NS: usize = BURROW_FOLDED_NUM_SPACES;
	const NA: usize = BURROW_FOLDED_NUM_AMPHIPODS;
	part1and2_impl(input_burrow::<NS, NA>())
}

#[allow(dead_code)]
pub(crate) fn part2() -> u32 {
	const NS: usize = BURROW_UNFOLDED_NUM_SPACES;
	const NA: usize = BURROW_UNFOLDED_NUM_AMPHIPODS;
	part1and2_impl(input_burrow::<NS, NA>())
}


mod parsing {
	use std::{iter, str::FromStr};
	use super::{
		BURROW_FOLDED, BURROW_ALL_LOCS,
		BURROW_SPACES_IN_FOLD, BURROW_AMPHIPOD_KINDS_IN_FOLD,
		BURROW_UNFOLDED_NUM_AMPHIPODS,
		AmphipodKind, Burrow};

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(crate) struct InvalidAmphipodKindError { found: char }

	impl TryFrom<char> for AmphipodKind {
		type Error = InvalidAmphipodKindError;
		fn try_from(value: char) -> Result<Self, Self::Error> {
			use AmphipodKind::*;
			match value {
				'A' => Ok(Amber),
				'B' => Ok(Bronze),
				'C' => Ok(Copper),
				'D' => Ok(Desert),
				found => Err(InvalidAmphipodKindError { found })
			}
		}
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum BurrowErrorKind {
		InvalidFormat { found: Option<char> },
		InvalidAmphipod(InvalidAmphipodKindError),
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum BurrowError {
		InvalidLine { line: usize, column: usize, kind: BurrowErrorKind },
		InvalidAmphipodsCount(AmphipodKind, usize),
	}

	impl<const NS: usize, const NA: usize> FromStr for Burrow<NS, NA> {
		type Err = BurrowError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use BurrowErrorKind::*;
			fn bur_line_err(l: usize, c: usize, kind: BurrowErrorKind) -> BurrowError {
				BurrowError::InvalidLine { line: l + 1, column: c + 1, kind }
			}

			let mut lines = s.lines().enumerate();
			let mut exp_lines = BURROW_FOLDED.lines().enumerate();
			let mut amphipods = iter::from_fn(move || match (lines.next(), exp_lines.next()) {
				(Some((l, line)), Some((_, exp_line))) => {
					let mut chars = line.chars().enumerate();
					let mut exp_chars = exp_line.chars().enumerate();
					Some(itertools::Either::Left(iter::from_fn(move || match (chars.next(), exp_chars.next()) {
						(Some((c, chr)), Some((_, 'a' | 'b' | 'c' | 'd'))) => {
							Some(Some(chr.try_into()
								.map(|amphipod| (
									BURROW_ALL_LOCS.iter().position(|&[x, y]| x == c && y == l).unwrap(),
									amphipod))
								.map_err(|e| bur_line_err(l, c, InvalidAmphipod(e)))))
						}
						(Some((c, chr)), exp) => match exp {
							Some((_, exp_chr)) if chr == exp_chr => Some(None),
							_ => Some(Some(Err(bur_line_err(l, c, InvalidFormat { found: Some(chr) })))),
						}
						(None, Some((c, _))) => {
							Some(Some(Err(bur_line_err(l, c, InvalidFormat { found: None }))))
						}
						(None, None) => None,
					}).flatten()))
				}
				(Some((l, line)), _) => {
					Some(itertools::Either::Right(iter::once(Err(bur_line_err(l, 0, InvalidFormat { found: line.chars().next() })))))
				}
				(None, Some((l, _))) => {
					Some(itertools::Either::Right(iter::once(Err(bur_line_err(l, 0, InvalidFormat { found: None })))))
				}
				(None, None) => None,
			}).flatten().collect::<Result<Vec<_>, _>>()?;

			for amphipod in [AmphipodKind::Amber, AmphipodKind::Bronze, AmphipodKind::Copper, AmphipodKind::Desert].into_iter() {
				let count = amphipods.iter().filter(|(_, a)| *a == amphipod).count();
				if count != 2 { return Err(BurrowError::InvalidAmphipodsCount(amphipod, count)) }
			}
			
			if NA == BURROW_UNFOLDED_NUM_AMPHIPODS {
				for (amphipod_space, _) in amphipods.iter_mut().skip(4) {
					*amphipod_space += 8;
				}

				_ = amphipods
					.splice(4..4, Iterator::zip(BURROW_SPACES_IN_FOLD, BURROW_AMPHIPOD_KINDS_IN_FOLD))
					.collect::<Vec<_>>();
			}

			Ok(Burrow { amphipods: amphipods.try_into().unwrap() })
		}
	}

	#[test]
	fn test() {
		use {BurrowError::*, BurrowErrorKind::*};
		const NSF: usize = super::BURROW_FOLDED_NUM_SPACES;
		const NAF: usize = super::BURROW_FOLDED_NUM_AMPHIPODS;
		assert!(matches!("".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 1, column: 1, kind: InvalidFormat { found: None } })));
		assert!(matches!("#".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 1, column: 2, kind: InvalidFormat { found: None } })));
		assert!(matches!("##x".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 1, column: 3, kind: InvalidFormat { found: Some('x') } })));
		assert!(matches!("##############x".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 1, column: 14, kind: InvalidFormat { found: Some('#') } })));
		assert!(matches!("#############".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 2, column: 1, kind: InvalidFormat { found: None } })));
		assert!(matches!("#############\n#.".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 2, column: 3, kind: InvalidFormat { found: None } })));
		assert!(matches!("#############\n#..x".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 2, column: 4, kind: InvalidFormat { found: Some('x') } })));
		assert!(matches!("#############\n#...........##x".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 2, column: 14, kind: InvalidFormat { found: Some('#') } })));
		assert!(matches!("#############\n#...........#\n".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 3, column: 1, kind: InvalidFormat { found: None } })));
		assert!(matches!("#############\n#...........#\n#".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 3, column: 2, kind: InvalidFormat { found: None } })));
		assert!(matches!("#############\n#...........#\n##x".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 3, column: 3, kind: InvalidFormat { found: Some('x') } })));
		assert!(matches!("#############\n#...........#\n###x".parse::<Burrow<NSF, NAF>>(), Err(InvalidLine { line: 3, column: 4, kind: InvalidAmphipod(_) })));
		assert!(matches!(super::TEST_INPUT.replace('A', "B").parse::<Burrow<NSF, NAF>>(), Err(BurrowError::InvalidAmphipodsCount(AmphipodKind::Amber, 0))));
		assert!(matches!(super::TEST_INPUT.parse::<Burrow<NSF, NAF>>(), Ok(_)));
		const NSU: usize = super::BURROW_UNFOLDED_NUM_SPACES;
		const NAU: usize = super::BURROW_UNFOLDED_NUM_AMPHIPODS;
		assert!(matches!(super::TEST_INPUT.parse::<Burrow<NSU, NAU>>(), Ok(_)));
	}
}


#[cfg(test)]
const TEST_INPUT: &str = indoc::indoc! { "
	#############
	#...........#
	###B#C#B#D###
	  #A#D#C#A#
	  #########
" };

#[test]
fn tests() {
	const NSF: usize = BURROW_FOLDED_NUM_SPACES;
	const NAF: usize = BURROW_FOLDED_NUM_AMPHIPODS;
	assert_eq!(part1and2_impl(input_burrow_from_str::<NSF, NAF>(TEST_INPUT)), 12521);
	assert_eq!(part1(), 13066);
	const NSU: usize = BURROW_UNFOLDED_NUM_SPACES;
	const NAU: usize = BURROW_UNFOLDED_NUM_AMPHIPODS;
	assert_eq!(part1and2_impl(input_burrow_from_str::<NSU, NAU>(TEST_INPUT)), 44169);
	assert_eq!(part2(), 47328);
}
