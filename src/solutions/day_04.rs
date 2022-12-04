use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

fn solution() -> Result<()> {
	let mut full_overlap_count = 0usize;
	let mut any_overlap_count = 0usize;
	try_for_each_line_in_file("inputs/day-04", |line| {
		let line = line.trim();
		let (l, r) = line
			.split_once(',')
			.ok_or_else(|| eyre!("invalid entry format {line}"))?;
		let [l, r] = [l, r].map(|range| -> Result<_> {
			let (s, e) = range
				.split_once('-')
				.ok_or_else(|| eyre!("invalid range format {range}"))?;
			let [s, e] = [s, e].map(|i| i.parse::<usize>());
			let (s, e) = (s?, e?);
			Ok(s..=e)
		});
		let (l, r) = (l?, r?);
		let full_overlap = l.contains(r.start()) && l.contains(r.end())
			|| r.contains(l.start()) && r.contains(l.end());
		full_overlap_count += full_overlap as usize;
		let any_overlap = l.contains(r.start())
			|| l.contains(r.end())
			|| r.contains(l.start())
			|| r.contains(l.end());
		any_overlap_count += any_overlap as usize;
		Ok(())
	})?;
	println!("{full_overlap_count}");
	println!("{any_overlap_count}");
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_04: (usize, fn() -> Result<()>) = (4, solution);
