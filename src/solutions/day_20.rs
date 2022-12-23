use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

#[derive(Copy, Clone, Debug)]
struct Entry<T> {
	value: T,
	previous: usize,
	next: usize,
}

fn decode(numbers: &[isize], key: isize, rounds: usize) -> Result<isize> {
	let length = numbers.len();
	let mut zero_index = None;
	let mut numbers: Vec<_> = numbers
		.iter()
		.copied()
		.enumerate()
		.map(|(index, value)| {
			if value == 0 {
				zero_index = Some(index);
			}
			let value = value * key;
			let previous = (index as isize - 1).rem_euclid(length as isize) as usize;
			let next = (index as isize + 1).rem_euclid(length as isize) as usize;
			Entry {
				value,
				previous,
				next,
			}
		})
		.collect();

	let zero_index = zero_index.ok_or_else(|| eyre!("zero entry missing!"))?;

	for _ in 0..rounds {
		for current in 0..length {
			let value = numbers[current].value;
			let d = value.rem_euclid(length as isize - 1);

			if d == 0 {
				continue;
			}

			let old_next = numbers[current].next;
			let old_previous = numbers[current].previous;

			let (next, previous) = if d < length as isize / 2 {
				let mut next = numbers[current].next;
				for _ in 0..d {
					next = numbers[next].next;
				}
				let previous = numbers[next].previous;
				(next, previous)
			} else {
				let mut previous = numbers[current].previous;
				let d = length as isize - d - 1;
				for _ in 0..d {
					previous = numbers[previous].previous;
				}
				let next = numbers[previous].next;
				(next, previous)
			};

			numbers[old_next].previous = old_previous;
			numbers[old_previous].next = old_next;

			numbers[current].next = next;
			numbers[current].previous = previous;

			numbers[next].previous = current;
			numbers[previous].next = current;
		}
	}

	let mut mixed = Vec::with_capacity(length);
	let mut next = zero_index;
	while {
		mixed.push(numbers[next].value);
		next = numbers[next].next;
		next != zero_index
	} {}

	let result = [1000usize, 2000, 3000]
		.into_iter()
		.map(|k| {
			let k = k.rem_euclid(length);
			mixed[k]
		})
		.sum();

	Ok(result)
}

fn solution() -> Result<()> {
	let mut numbers = vec![];
	try_for_each_line_in_file("inputs/day-20", |line| {
		let line = line.trim();
		let entry = line.parse::<isize>()?;
		numbers.push(entry);
		Ok(())
	})?;

	let result_1 = decode(&numbers, 1, 1)?;
	println!("{result_1}");

	let result_2 = decode(&numbers, 811_589_153, 10)?;
	println!("{result_2}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_20: (usize, fn() -> Result<()>) = (20, solution);
