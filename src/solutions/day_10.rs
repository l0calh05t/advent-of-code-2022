use std::str::from_utf8;

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

fn solution() -> Result<()> {
	let mut cycle = 0i32;
	let mut x = 1;
	let mut signal_strength = 0;
	let mut pixels = [b'.'; 6 * 40];
	let mut update = |add_x: Option<i32>| {
		if (x - cycle % 40).abs() <= 1 {
			pixels[cycle as usize % pixels.len()] = b'#';
		}
		cycle += 1;
		if (cycle + 20) % 40 == 0 {
			signal_strength += cycle * x;
		}
		if let Some(add_x) = add_x {
			x += add_x;
		}
	};
	try_for_each_line_in_file("inputs/day-10", |line| {
		let line = line.trim();
		if let Some(d) = line.strip_prefix("addx ") {
			let d = d.parse::<i32>()?;
			update(None);
			update(Some(d));
		} else if line == "noop" {
			update(None);
		} else {
			return Err(eyre!("unknown instruction {line}"));
		}
		Ok(())
	})?;
	println!("{signal_strength}");
	for scan_line in pixels.chunks(40) {
		let scan_line = from_utf8(scan_line).expect("unreachable");
		println!("{scan_line}");
	}
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_10: (usize, fn() -> Result<()>) = (10, solution);
