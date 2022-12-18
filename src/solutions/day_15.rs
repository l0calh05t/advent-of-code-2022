use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use gcollections::ops::{Bounded, Cardinality, Difference, Empty, IsEmpty, Union};
use interval::{interval_set::ToIntervalSet, IntervalSet};
use linkme::distributed_slice;

fn read_input() -> Result<Vec<([i64; 2], [i64; 2])>> {
	let mut result = Vec::new();
	try_for_each_line_in_file("inputs/day-15", |line| {
		let (sx, line) = line
			.trim()
			.strip_prefix("Sensor at x=")
			.ok_or_else(|| eyre!("invalid input prefix"))?
			.split_once(", y=")
			.ok_or_else(|| eyre!("can't split sensor position components"))?;
		let (sy, line) = line
			.split_once(": closest beacon is at x=")
			.ok_or_else(|| eyre!("can't split sensor and beacon positions"))?;
		let (bx, by) = line
			.split_once(", y=")
			.ok_or_else(|| eyre!("can't split beacon components"))?;
		let [sx, sy, bx, by] = [sx, sy, bx, by].map(str::parse::<i64>);
		let [sx, sy, bx, by] = [sx?, sy?, bx?, by?];
		result.push(([sx, sy], [bx, by]));
		Ok(())
	})?;
	Ok(result)
}

fn intervals_in_line(
	input: &[([i64; 2], [i64; 2])],
	y_ref: i64,
	subtract_existing: bool,
) -> IntervalSet<i64> {
	let mut intervals = IntervalSet::<i64>::empty();

	// add intervals within reach (#)
	input
		.iter()
		.copied()
		.filter_map(|([sx, sy], [bx, by])| {
			let d = sx.abs_diff(bx) + sy.abs_diff(by);
			let d_ref = sy.abs_diff(y_ref);
			if d_ref > d {
				return None;
			}
			let dd = (d - d_ref) as i64;
			Some((sx - dd, sx + dd))
		})
		.for_each(|interval| {
			intervals = intervals.union(&interval.to_interval_set());
		});

	// subtract existing beacons (B)
	if subtract_existing {
		input
			.iter()
			.copied()
			.filter_map(|(_, [bx, by])| (by == y_ref).then_some((bx, bx)))
			.for_each(|interval| {
				intervals = intervals.difference(&interval.to_interval_set());
			});
	}

	intervals
}

fn solution() -> Result<()> {
	let input = read_input()?;
	let y_ref = 2000000;
	println!("{}", intervals_in_line(&input, y_ref, true).size());

	let mut found = false;
	for y_ref in 0..=4000000 {
		let intervals = intervals_in_line(&input, y_ref, false);
		let remaining = (0, 4000000).to_interval_set().difference(&intervals);
		if remaining.is_empty() {
			continue;
		}
		if remaining.size() != 1 || found {
			return Err(eyre!("more than one possible sensor position"));
		}

		let interval = remaining.into_iter().next().expect("unreachable");
		let x = interval.lower();
		let tuning_frequency = 4000000 * x + y_ref;
		println!("{tuning_frequency}");
		found = true;
	}

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_15: (usize, fn() -> Result<()>) = (15, solution);
