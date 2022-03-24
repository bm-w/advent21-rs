// Copyright (c) 2022 Bastiaan Marinus van de Weerd

use std::str::FromStr;


#[derive(Debug)]
enum Operator {
	Sum,
	Product,
	Minimum,
	Maximum,
	Descending,
	Ascending,
	Equal,
}

#[derive(Debug)]
enum PacketKind {
	Literal(u64),
	Operator(Operator, Vec<Packet>),
}

impl PacketKind {
	fn total_version(&self) -> u64 {
		if let PacketKind::Operator(_, packets) = self {
			packets.iter().map(|p| p.total_version()).sum()
		} else {
			0
		}
	}

	fn evaluate(&self) -> u64  {
		use {PacketKind::{Literal as Lit, Operator as Op}, Operator::*};
		match self {
			Lit(value) => *value,
			Op(Sum, packets) =>
				packets.iter().map(|p| p.evaluate()).sum(),
			Op(Product, packets) =>
				packets.iter().map(|p| p.evaluate()).product(),
			Op(Minimum, packets) =>
				packets.iter().map(|p| p.evaluate()).min().unwrap(),
			Op(Maximum, packets) =>
				packets.iter().map(|p| p.evaluate()).max().unwrap(),
			Op(Descending, packets) if packets.len() == 2 =>
				if packets[0].evaluate() > packets[1].evaluate() { 1 } else { 0 },
			Op(Ascending, packets) if packets.len() == 2 =>
				if packets[0].evaluate() < packets[1].evaluate(){ 1 } else { 0 },
			Op(Equal, packets) if packets.len() == 2 =>
				if packets[0].evaluate() == packets[1].evaluate(){ 1 } else { 0 },
			_ => unreachable!(),
		}
	}
}

#[derive(Debug)]
struct Packet {
	version: u8,
	kind: PacketKind,
}

impl Packet {
	fn total_version(&self) -> u64 {
		self.version as u64 + self.kind.total_version()
	}

	fn evaluate(&self) -> u64  {
		self.kind.evaluate()
	}
}


fn input_packet_from_str(s: &str) -> Packet {
	s.parse().unwrap()
}

fn input_packet() -> Packet {
	input_packet_from_str(include_str!("day16.txt"))
}


fn part1_impl(input_packet: Packet) -> u64 {
	input_packet.total_version()
}

pub(crate) fn part1() -> u64 {
	part1_impl(input_packet())
}


fn part2_impl(input_packet: Packet) -> u64 {
	input_packet.evaluate()
}

pub(crate) fn part2() -> u64 {
	part2_impl(input_packet())
}


#[allow(dead_code)]
#[derive(Debug)]
enum ParsePacketError {
	InvalidChar { column: usize, found: char },
	IncompleteVersion { column: usize, found: usize },
	IncompleteTypeId { column: usize, found: usize },
	MissingLiteralGroupPrefix { column: usize, found: usize },
	IncompleteLiteralGroup { column: usize, found: usize },
	InvalidOperator { column: usize, found: u8 },
	MissingOperatorMode { column: usize },
	IncompleteBitsOperatorMode { column: usize, found: usize },
	IncompletePacketsOperatorMode { column: usize, found: usize },
	TrailingNonZeroBit { column: usize },
}

mod parsing {
	use std::iter;
	use super::{Operator, PacketKind, Packet, ParsePacketError};

	enum OperatorMode {
		Bits(usize),
		Packets(usize),
	}

	impl TryFrom<u8> for Operator {
		type Error = ();
		fn try_from(value: u8) -> Result<Self, Self::Error> {
			use Operator::*;
			match value {
				0 => Ok(Sum),
				1 => Ok(Product),
				2 => Ok(Minimum),
				3 => Ok(Maximum),
				5 => Ok(Descending),
				6 => Ok(Ascending),
				7 => Ok(Equal),
				_ => Err(()),
			}
		}
	}

	impl TryFrom<&str> for Packet {
		type Error = super::ParsePacketError;
		fn try_from(s: &str) -> Result<Self, Self::Error> {
			use ParsePacketError::*;

			let mut bits = s
				.lines()
				.take(1)
				.flat_map(|line| line.chars())
				.enumerate()
				.flat_map(|(i, chr)| chr.to_digit(16)
					.map_or(
						itertools::Either::Left(
							iter::once(Err(InvalidChar { column: i + 1, found: chr }))),
						|d| itertools::Either::Right([
							Ok((i + 1, false, d & 0x8 > 0)),
							Ok((i + 1, false, d & 0x4 > 0)),
							Ok((i + 1, false, d & 0x2 > 0)),
							Ok((i + 1, true, d & 0x1 > 0)),
						].into_iter())));

			let mut column = 1;
			let mut bits_seen = 0;

			macro_rules! try_bit {
				( $err:expr ) => { {
					let (c, l, b) = bits.by_ref().next()
						.unwrap_or_else(|| Err($err()))?;
					column = if l { c + 1 } else { c };
					bits_seen += 1;
					b
				} }
			}

			macro_rules! try_bits_u {
				( $len:expr , $err:expr ) => { {
					let mut val = 0;
					for i in 0..$len {
						if try_bit!(|| $err(column, i)) {
							val += 1 << ($len - i - 1)
						}
					}
					val
				} }
			}

			const VERSION_LEN: usize = 3;
			const TYPE_ID_LEN: usize = 3;
			const LITERAL_GROUP_LEN: usize = 4;

			let mut state = vec![(None, None, bits_seen)];
			let packet = loop {
				let packet = match state.last_mut() {
					Some((state @ None, op_mode @ None, _)) => {
						let version = try_bits_u!(VERSION_LEN, |c, f| IncompleteVersion { column: c, found: f });
						let type_id = try_bits_u!(TYPE_ID_LEN, |c, f| IncompleteTypeId { column: c, found: f });
						if type_id == 4u8 {
							let mut value = 0;
							for i in 0.. {
								let last = !try_bit!(|| MissingLiteralGroupPrefix { column, found: i });
								let group = try_bits_u!(LITERAL_GROUP_LEN, |c, f| IncompleteLiteralGroup { column: c, found: f });
								value = (value << LITERAL_GROUP_LEN) + group;
								if last { break }
							}
							Some(Packet { version, kind: PacketKind::Literal(value) })
						} else {
							let op = type_id.try_into()
								.map_err(|_| InvalidOperator { column, found: type_id })?;
							*op_mode = Some(if !try_bit!(|| MissingOperatorMode { column }) {
								OperatorMode::Bits(try_bits_u!(15, |c, f| IncompleteBitsOperatorMode { column: c, found: f }))
							} else {
								OperatorMode::Packets(try_bits_u!(11, |c, f| IncompletePacketsOperatorMode { column: c, found: f }))
							});
							*state = Some(Packet { version, kind: PacketKind::Operator(op, Vec::new()) });
							None
						}
					}
					Some((operator_packet, ref mode, ref bits_start)) => {
						use {PacketKind::*, OperatorMode::*};
						let complete = match (&operator_packet, mode) {
							(Some(Packet { kind: Operator(_, _), .. }), Some(Bits(packets_bits_len))) =>
								bits_seen - *bits_start - 22 == *packets_bits_len,
							(Some(Packet { kind:  Operator(_, packets), .. }), Some(Packets(packets_len))) =>
								&packets.len() == packets_len,
							_ => unreachable!(),
						};

						if complete {
							operator_packet.take()
						} else {
							state.push((None, None, bits_seen));
							None
						}
					}
					_ => unreachable!()
				};

				if let Some(packet) = packet {
					let state_len = state.len();
					if state_len == 1 {
						break packet
					} else {
						let popped_none = state.remove(state_len - 1).0;
						assert!(popped_none.is_none());
						match &mut state[state_len - 2].0 {
							Some(Packet { kind: PacketKind::Operator(_, packets), .. }) =>
								packets.push(packet),
							_ => unreachable!(),
						}
					}
				}
			};

			bits
				.map(|r| r.and_then(|(c, _, b)| if !b { Ok(()) }
					else { Err(TrailingNonZeroBit { column: c }) }))
				.find(|r| r.is_err())
				.transpose()?;
			Ok(packet)
		}
	}
}

impl FromStr for Packet {
	type Err = ParsePacketError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.try_into()
	}
}


#[test]
fn tests() {
	assert_eq!(part1_impl(input_packet_from_str("D2FE28")), 6);
	assert_eq!(part1_impl(input_packet_from_str("38006F45291200")), 9);
	assert_eq!(part1_impl(input_packet_from_str("EE00D40C823060")), 14);
	assert_eq!(part1_impl(input_packet_from_str("8A004A801A8002F478")), 16);
	assert_eq!(part1_impl(input_packet_from_str("620080001611562C8802118E34")), 12);
	assert_eq!(part1_impl(input_packet_from_str("C0015000016115A2E0802F182340")), 23);
	assert_eq!(part1(), 895);

	assert_eq!(part2_impl(input_packet_from_str("C200B40A82")), 3);
	assert_eq!(part2_impl(input_packet_from_str("04005AC33890")), 54);
	assert_eq!(part2_impl(input_packet_from_str("880086C3E88112")), 7);
	assert_eq!(part2_impl(input_packet_from_str("CE00C43D881120")), 9);
	assert_eq!(part2_impl(input_packet_from_str("D8005AC2A8F0")), 1);
	assert_eq!(part2_impl(input_packet_from_str("F600BC2D8F")), 0);
	assert_eq!(part2_impl(input_packet_from_str("9C005AC2F8F0")), 0);
	assert_eq!(part2_impl(input_packet_from_str("9C0141080250320F1802104A08")), 1);
	assert_eq!(part2(), 1148595959144);
}
