use std::cmp::Ordering;

use crate::SOLUTIONS;

use color_eyre::eyre::Result;
use linkme::distributed_slice;
use nom::{
	branch::alt,
	character::complete::{char, digit1, line_ending},
	combinator::{map, map_res},
	multi::separated_list0,
	sequence::{delimited, pair, terminated},
	IResult,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum PacketData {
	Integer(u8),
	List(Vec<PacketData>),
}

impl PartialOrd for PacketData {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		use PacketData::*;
		match (self, other) {
			(Integer(left), Integer(right)) => Some(left.cmp(right)),
			(List(left), List(right)) => Some(left.cmp(right)),
			(List(left), Integer(right)) => Some(left.as_slice().cmp([Integer(*right)].as_slice())),
			(Integer(left), List(right)) => Some([Integer(*left)].as_slice().cmp(right.as_slice())),
		}
	}
}

impl Ord for PacketData {
	fn cmp(&self, other: &Self) -> Ordering {
		self.partial_cmp(other).expect("unreachable")
	}
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Packet(Vec<PacketData>);

fn parse_packet_data(i: &str) -> IResult<&str, PacketData, ()> {
	alt((
		map(parse_packet_data_list, PacketData::List),
		map_res(digit1, |s: &str| {
			s.parse().map(PacketData::Integer).map_err(|_| ())
		}),
	))(i)
}

fn parse_packet_data_list(i: &str) -> IResult<&str, Vec<PacketData>, ()> {
	delimited(
		char('['),
		separated_list0(char(','), parse_packet_data),
		char(']'),
	)(i)
}

fn parse_packet(i: &str) -> IResult<&str, Packet, ()> {
	map(terminated(parse_packet_data_list, line_ending), Packet)(i)
}

fn parse_input(i: &str) -> IResult<&str, Vec<(Packet, Packet)>, ()> {
	separated_list0(line_ending, pair(parse_packet, parse_packet))(i)
}

fn solution() -> Result<()> {
	let input = std::fs::read_to_string("inputs/day-13")?;
	let (_, packets) = parse_input(&input)?;
	let correct = packets
		.iter()
		.enumerate()
		.filter(|(_, (left, right))| left < right)
		.map(|(i, _)| i + 1)
		.sum::<usize>();
	println!("{correct}");

	let markers = [
		Packet(vec![PacketData::List(vec![PacketData::Integer(2)])]),
		Packet(vec![PacketData::List(vec![PacketData::Integer(6)])]),
	];
	let mut packets: Vec<_> = packets
		.into_iter()
		.flat_map(|(a, b)| [a, b])
		.chain(markers.clone())
		.collect();
	packets.sort();
	let key = markers
		.iter()
		.map(|m| packets.binary_search(m).expect("unreachable") + 1)
		.product::<usize>();
	println!("{key}");
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_13: (usize, fn() -> Result<()>) = (13, solution);
