// Copyright (c) 2022 Bastiaan Marinus van de Weerd


#[derive(Debug)]
struct Player {
	#[allow(dead_code)]
	id: usize,
	/// 1-based
	starting_position: usize,
}


fn input_players_from_string(s: &str) -> [Player; 2] {
	parsing::players_from_str(s).unwrap()
}

fn input_players() -> [Player; 2] {
	input_players_from_string(include_str!("day21.txt"))
}


fn part1_impl(input_players: [Player; 2]) -> u64 {
	let mut rolls = (1..=100).cycle();
	let mut positions = [
		input_players[0].starting_position - 1,
		input_players[1].starting_position - 1,
	];
	let mut scores = [0; 2];
	for i in 0.. {
		let i_mod = i % input_players.len();
		let moove = rolls.by_ref().take(3).sum::<usize>();
		assert!(moove > 0);
		let pos = (positions[i_mod] + moove) % 10;
		positions[i_mod] = pos;
		scores[i_mod] += pos as u64 + 1;
		if scores[i_mod] >= 1000 {
			let i_other = (i + 1) % input_players.len();
			let score_other = scores[i_other];
			return score_other * 3 * (i as u64 + 1)
		}
	}
	unreachable!()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_players())
}


mod quantum {
	use super::Player;

	const TARGET_SCORE: u64 = 21;

	/// In a player’s “worst-case” universe, where regardless of where
	/// they start, they land on space 3 and get 3 point on the first
	/// turn, then roll 9 to land on space 2, then roll 9 to land on
	/// space 1, etc. for 3+2+1+4+2+1+4+2+1+4=24 points after 10 turns.
	const NUM_TURNS: usize = 10;

	/// After one turn (3 die rolls), there will be one universe where
	/// we moved 3 spaces (we rolled 3x1), 3 universes where we moved 4
	/// spaces (we rolled 2x1 & 1x3, in 3 permutations), etc.
	const KERNEL: [usize; 10] = [0, 0, 0, 1, 3, 6, 7, 6, 3, 1];

	#[derive(Debug, Clone, Copy)]
	pub(super) struct Turn {
		#[allow(dead_code)]
		pub(super) seq: usize,
		pub(super) still_playing_universes: usize,
		pub(super) reached_target_universes: usize,
	}

	impl Player {
		pub(super) fn quantum_turns(&self) -> impl Iterator<Item = Turn> + '_ {
			// Was 1-based, need 0-based.
			let starting_position = self.starting_position - 1;
	
			// For each space, all possible scores, and for each score,
			// the number of universes in that space with that score. 
			let mut space_score_universes
				= [[0usize; TARGET_SCORE as usize]; 10];
	
			// Before the first turn, there is only one universe
			// at the starting point and it has a score of zero.
			space_score_universes[starting_position][0] = 1;

			(0..NUM_TURNS).map(move |turn| {
				let mut still_playing_universes = 0;
				let mut reached_target_universes = 0;
				let mut new_space_score_universes
					= [[0usize; TARGET_SCORE as usize]; 10];
				for (space, score_universes) in space_score_universes.iter().enumerate() {
					for (score, universes) in score_universes.iter().enumerate().map(|(s, &u)| (s as u64, u)) {
						if universes == 0 { continue }
						for (moove, &move_universes) in KERNEL.iter().enumerate() {
							if move_universes == 0 { continue }
							let new_universes = universes * move_universes;
							let new_space = (space + moove) % 10;
							let new_score = score + new_space as u64 + 1;
							if new_score < TARGET_SCORE {
								still_playing_universes += new_universes;
								new_space_score_universes[new_space][new_score as usize] += new_universes;
							} else {
								reached_target_universes += new_universes;
							}
						}
					}
				}
				space_score_universes = new_space_score_universes;
				Turn { seq: turn, still_playing_universes, reached_target_universes }
			})
		}
	}
}


fn part2_impl(input_players: [Player; 2]) -> usize {
	let turns = {
		let mut turns = [
			input_players[0].quantum_turns(),
			input_players[1].quantum_turns()
		];
		let mut prev_turn = turns[0].next();
		let mut t = 0;
		std::iter::from_fn(move || {
			let prev_t = t;
			t = (t + 1) % turns.len();
			let mut round_turns = [None, None];
			round_turns[prev_t] = prev_turn;
			round_turns[t] = turns[t].next();
			prev_turn = round_turns[t];
			round_turns[0].zip(round_turns[1])
				.map(|(t0, t1)| (t, [t0, t1]))
		})
	};

	let mut win_universes = [0, 0];
	for (t, turns) in turns {
		let reached_target_universes = turns[t].reached_target_universes;
		let other_still_playing_universes = turns[(t + 1) % turns.len()].still_playing_universes;
		win_universes[t] += reached_target_universes * other_still_playing_universes;
	}
	win_universes.into_iter().max().unwrap()
}

pub(crate) fn part2() -> usize {
	part2_impl(input_players())
}


mod parsing {
	use std::{str::FromStr, num::ParseIntError};
	use super::Player;

	#[derive(Debug)]
	pub(super) enum IntError<T> {
		Format,
		Parsing(ParseIntError),
		Invalid(T),
	}

	#[allow(dead_code, clippy::enum_variant_names)]
	#[derive(Debug)]
	pub(super) enum PlayerError {
		InvalidFormat { column: usize },
		InvalidId { column: usize, source: IntError<usize> },
		InvalidStartingPosition { column: usize, source: IntError<usize> },
	}

	impl FromStr for Player {
		type Err = PlayerError;
		fn from_str(s: &str) -> Result<Self, Self::Err> {
			use PlayerError::*;

			const ID_PREFIX: &str = "Player ";
			const STARTING_POSITION_PREFIX: &str = " starting position: ";

			if !s.starts_with(ID_PREFIX) {
				let c = s.chars().zip(ID_PREFIX.chars()).take_while(|(l, r)| l == r).count();
				return Err(InvalidFormat { column: c + 1 })
			}
			let id_start = ID_PREFIX.len();
			let id_end = id_start + s[id_start..].find(|c: char| !c.is_numeric())
				.ok_or(InvalidId { column: id_start + 1, source: IntError::Format })?;
			let id = s[id_start..id_end].parse()
				.map_err(|e| InvalidId { column: id_start + 1, source: IntError::Parsing(e) })?;

			if !s[id_end..].starts_with(STARTING_POSITION_PREFIX) {
				let c = s[id_end..].chars().zip(STARTING_POSITION_PREFIX.chars()).take_while(|(l, r)| l == r).count();
				return Err(InvalidFormat { column: c + 1 })
			}
			let starting_position_start = id_end + STARTING_POSITION_PREFIX.len();
			if s[starting_position_start..].contains(|c: char| !c.is_numeric()) {
				return Err(InvalidStartingPosition { column: starting_position_start + 1, source: IntError::Format })
			}
			let starting_position = s[starting_position_start..].parse()
				.map_err(|e| InvalidStartingPosition { column: starting_position_start + 1, source: IntError::Parsing(e) })?;
			if !(1..=10).contains(&starting_position) {
				return Err(InvalidStartingPosition { column: starting_position_start + 1, source: IntError::Invalid(starting_position) })
			}

			Ok(Player { id, starting_position })
		}
	}

	#[allow(dead_code)]
	#[derive(Debug)]
	pub(super) enum PlayersError {
		InvalidFormat { line: usize },
		InvalidPlayer { line: usize, source: PlayerError},
	}

	pub(super) fn players_from_str(s: &str) -> Result<[Player; 2], PlayersError> {
		use PlayersError::*;
		let mut lines = s.lines();
		let players = lines.by_ref()
			.enumerate()
			.map(|(l, line)| line.parse()
				.map_err(|e| InvalidPlayer { line: l + 1, source: e }))
			.take(2)
			.collect::<Result<Vec<_>, _>>()?;
		if players.len() < 2 { Err(InvalidFormat { line: players.len() + 1 }) }
		else if lines.next().is_some() { Err(InvalidFormat { line: 3 }) }
		else { Ok(players.try_into().unwrap()) }
	}
}


#[test]
fn tests() -> Result<(), parsing::PlayersError>{
	const INPUT: &str = indoc::indoc! { "
		Player 1 starting position: 4
		Player 2 starting position: 8
	" };
	assert_eq!(part1_impl(parsing::players_from_str(INPUT)?), 739785);
	assert_eq!(part1(), 920580);
	assert_eq!(part2_impl(parsing::players_from_str(INPUT)?), 444356092776315);
	assert_eq!(part2(), 647920021341197);
	Ok(())
}
