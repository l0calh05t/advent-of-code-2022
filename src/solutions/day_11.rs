use std::str::{from_utf8, FromStr};

use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use linkme::distributed_slice;
use nom::{
	branch::alt,
	bytes::complete::tag,
	character::complete::{digit1, line_ending},
	combinator::{map, map_res, opt},
	multi::separated_list1,
	sequence::{preceded, terminated, tuple},
	IResult,
};
use num::Integer;

#[derive(Clone, Copy, Debug)]
enum Operation {
	Square,
	MulConst(u64),
	AddConst(u64),
}

use Operation::*;

impl Operation {
	pub fn evaluate(self, old: u64) -> u64 {
		match self {
			Square => old * old,
			MulConst(c) => old * c,
			AddConst(c) => old + c,
		}
	}
}

#[derive(Clone, Debug)]
struct Monkey {
	id: usize,
	items: Vec<u64>,
	operation: Operation,
	test_modulo: u64,
	target_ids: [usize; 2],
	items_inspected: usize,
}

fn parse_as_str<F: FromStr>(i: &[u8]) -> std::result::Result<F, ()> {
	from_utf8(i).map_err(|_| ())?.parse().map_err(|_| ())
}

fn parse_monkey_id(i: &[u8]) -> IResult<&[u8], usize, ()> {
	preceded(
		tag(b"Monkey "),
		terminated(
			map_res(digit1, parse_as_str),
			tuple((tag(b":"), line_ending)),
		),
	)(i)
}

fn parse_monkey_items(i: &[u8]) -> IResult<&[u8], Vec<u64>, ()> {
	preceded(
		tag(b"  Starting items: "),
		terminated(
			separated_list1(tag(b", "), map_res(digit1, parse_as_str)),
			line_ending,
		),
	)(i)
}

fn parse_operation(i: &[u8]) -> IResult<&[u8], Operation, ()> {
	preceded(
		tag(b"  Operation: new = old "),
		terminated(
			alt((
				map(tag(b"* old"), |_| Square),
				map(
					preceded(tag(b"* "), map_res(digit1, parse_as_str)),
					MulConst,
				),
				map(
					preceded(tag(b"+ "), map_res(digit1, parse_as_str)),
					AddConst,
				),
			)),
			line_ending,
		),
	)(i)
}

fn parse_test(i: &[u8]) -> IResult<&[u8], u64, ()> {
	preceded(
		tag(b"  Test: divisible by "),
		terminated(map_res(digit1, parse_as_str), line_ending),
	)(i)
}

fn parse_targets(i: &[u8]) -> IResult<&[u8], [usize; 2], ()> {
	map(
		tuple((
			preceded(
				tag(b"    If true: throw to monkey "),
				terminated(map_res(digit1, parse_as_str), line_ending),
			),
			preceded(
				tag(b"    If false: throw to monkey "),
				terminated(map_res(digit1, parse_as_str), line_ending),
			),
		)),
		|(t, f)| [t, f],
	)(i)
}

fn parse_monkey(i: &[u8]) -> IResult<&[u8], Monkey, ()> {
	map(
		terminated(
			tuple((
				parse_monkey_id,
				parse_monkey_items,
				parse_operation,
				parse_test,
				parse_targets,
			)),
			opt(line_ending),
		),
		|(id, items, operation, test_modulo, target_ids)| Monkey {
			id,
			items,
			operation,
			test_modulo,
			target_ids,
			items_inspected: 0,
		},
	)(i)
}

fn parse_monkeys() -> Result<Vec<Monkey>> {
	let input = std::fs::read_to_string("inputs/day-11")?;
	let mut input = input.as_bytes();
	let mut monkeys = Vec::new();
	while let Ok((remainder, monkey)) = parse_monkey(input) {
		if monkey.id != monkeys.len() {
			return Err(eyre!("expected sorted monkeys"));
		}
		monkeys.push(monkey);
		input = remainder;
	}
	if !input.is_empty() {
		return Err(eyre!("input could not be parsed completely"));
	}
	if monkeys.iter().enumerate().any(|(i, m)| {
		m.target_ids[0] == m.target_ids[1]
			|| m.target_ids.iter().any(|&j| j >= monkeys.len() || i == j)
	}) {
		return Err(eyre!("invalid target monkeys"));
	}
	Ok(monkeys)
}

fn pick_disjoint_mut<T, const N: usize>(items: &mut [T], indices: [usize; N]) -> [&mut T; N] {
	assert!(indices.iter().all(|&i| i < items.len()));
	assert!(indices.iter().tuple_combinations().all(|(&i, &j)| i != j));
	indices.map(|i| unsafe { &mut *items.as_mut_ptr().add(i) })
}

fn perform_round_1(monkeys: &mut [Monkey]) {
	for i in 0..monkeys.len() {
		let [monkey, true_monkey, false_monkey] = pick_disjoint_mut(
			monkeys,
			[i, monkeys[i].target_ids[0], monkeys[i].target_ids[1]],
		);

		monkey.items_inspected += monkey.items.len();
		for item in monkey.items.drain(..) {
			let item = monkey.operation.evaluate(item) / 3;
			if item % monkey.test_modulo == 0 {
				true_monkey.items.push(item);
			} else {
				false_monkey.items.push(item);
			}
		}
	}
}

fn part_1(mut monkeys: Vec<Monkey>) {
	(0..20).for_each(|_| perform_round_1(&mut monkeys));
	monkeys.select_nth_unstable_by(1, |a, b| b.items_inspected.cmp(&a.items_inspected));
	let score = monkeys[..2]
		.iter()
		.map(|m| m.items_inspected)
		.product::<usize>();
	println!("{score}");
}

fn perform_round_2(monkeys: &mut [Monkey], modulo: u64) {
	for i in 0..monkeys.len() {
		let [monkey, true_monkey, false_monkey] = pick_disjoint_mut(
			monkeys,
			[i, monkeys[i].target_ids[0], monkeys[i].target_ids[1]],
		);

		monkey.items_inspected += monkey.items.len();
		for item in monkey.items.drain(..) {
			let item = monkey.operation.evaluate(item) % modulo;
			if item % monkey.test_modulo == 0 {
				true_monkey.items.push(item);
			} else {
				false_monkey.items.push(item);
			}
		}
	}
}

fn part_2(mut monkeys: Vec<Monkey>) {
	let modulo = monkeys
		.iter()
		.map(|m| m.test_modulo)
		.reduce(|c, d| c.lcm(&d))
		.unwrap_or(1);
	(0..10000).for_each(|_| perform_round_2(&mut monkeys, modulo));
	monkeys.select_nth_unstable_by(1, |a, b| b.items_inspected.cmp(&a.items_inspected));
	let score = monkeys[..2]
		.iter()
		.map(|m| m.items_inspected)
		.product::<usize>();
	println!("{score}");
}

fn solution() -> Result<()> {
	let monkeys = parse_monkeys()?;
	part_1(monkeys.clone());
	part_2(monkeys);
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_11: (usize, fn() -> Result<()>) = (11, solution);
