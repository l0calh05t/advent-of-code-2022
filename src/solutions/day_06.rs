use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use linkme::distributed_slice;

fn find_marker(data: &[u8], size: usize) -> Option<usize> {
	let mut temp_storage = Vec::with_capacity(size);
	data.windows(size)
		.find_position(|&w| {
			temp_storage.clear();
			temp_storage.extend_from_slice(w);
			temp_storage.sort();
			temp_storage.windows(2).all(|w| w[0] != w[1])
		})
		.map(|(wi, _)| wi + size)
}

fn solution() -> Result<()> {
	let data = include_bytes!("../../inputs/day-06");
	for size in [4, 14] {
		let solution = find_marker(data, size).ok_or_else(|| eyre!("no marker found"))?;
		println!("{solution}");
	}
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_06: (usize, fn() -> Result<()>) = (6, solution);
