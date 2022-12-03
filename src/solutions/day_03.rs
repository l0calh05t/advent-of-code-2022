use std::{
	collections::HashSet,
	fs::File,
	io::{BufRead, BufReader},
};

use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

fn solution() -> Result<()> {
	let file = File::open("inputs/day-03")?;
	let mut file = BufReader::new(file);
	let mut line = String::new();
	let mut items = Vec::new();
	let mut total_rucksacks = 0;
	let mut total_groups = 0;
	let mut elf = 0usize;
	let mut group_set = HashSet::new();
	while file.read_line(&mut line)? != 0 {
		let rucksack = line.trim().as_bytes();
		for priority in rucksack.iter().map(|c| match c {
			b'a'..=b'z' => Ok(c - b'a' + 1),
			b'A'..=b'Z' => Ok(c - b'A' + 27),
			_ => Err(eyre!("unexpected rucksack item")),
		}) {
			items.push(priority?);
		}

		// part 1
		let size = items.len();
		if size % 2 != 0 {
			return Err(eyre!("unexpected rucksack size"));
		}
		let (left, right) = items.split_at(size / 2);

		let left: HashSet<_> = left.iter().copied().collect();
		let right: HashSet<_> = right.iter().copied().collect();

		total_rucksacks += left.intersection(&right).map(|&p| p as u64).sum::<u64>();

		// part 2
		let rucksack: HashSet<_> = items.iter().copied().collect();

		let group_elf = elf % 3;
		if group_elf == 0 {
			group_set = rucksack;
		} else {
			let temp = group_set.intersection(&rucksack).copied().collect();
			group_set = temp;
		}
		if group_elf == 2 {
			total_groups += group_set.iter().map(|&p| p as u64).sum::<u64>();
		}

		line.clear();
		items.clear();
		elf += 1;
	}
	println!("{total_rucksacks}");
	println!("{total_groups}");
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_03: (usize, fn() -> Result<()>) = (3, solution);
