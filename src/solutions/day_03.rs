use std::{
	fs::File,
	io::{BufRead, BufReader},
};

use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use linkme::distributed_slice;

fn solution() -> Result<()> {
	let file = File::open("inputs/day-03")?;
	let mut file = BufReader::new(file);
	let mut line = String::new();
	let mut total_rucksacks = 0;
	let mut total_groups = 0;
	let mut elf = 0usize;
	let mut group_set = u64::MAX;
	while file.read_line(&mut line)? != 0 {
		let rucksack = line.trim().as_bytes();
		let size = rucksack.len();
		if size % 2 != 0 {
			return Err(eyre!("unexpected rucksack size"));
		}

		let (left, right) = rucksack.split_at(size / 2);
		let [left, right] = [left, right].map(|pocket| {
			pocket
				.iter()
				.map(|&c| match c {
					b'a'..=b'z' => Ok(c - b'a'),
					b'A'..=b'Z' => Ok(c - b'A' + 26),
					_ => Err(eyre!("unexpected rucksack item")),
				})
				.fold_ok(0u64, |set, item| set | 1 << item)
		});
		// could avoid this with try_map (nightly only #![feature(array_try_map)] so far)
		let left = left?;
		let right = right?;

		let evaluate = |set: u64| -> u64 {
			(0..52)
				.map(|i| if set & (1 << i) != 0 { i + 1 } else { 0 })
				.sum()
		};

		// part 1
		total_rucksacks += evaluate(left & right);

		// part 2
		group_set &= left | right;
		if elf % 3 == 2 {
			total_groups += evaluate(group_set);
			group_set = u64::MAX;
		}

		line.clear();
		elf += 1;
	}
	println!("{total_rucksacks}");
	println!("{total_groups}");
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_03: (usize, fn() -> Result<()>) = (3, solution);
