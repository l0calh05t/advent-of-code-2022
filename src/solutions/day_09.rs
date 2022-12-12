use std::collections::HashSet;

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

fn solve<const K: usize>() -> Result<()> {
	let mut knots = [[0i64; 2]; K];
	let mut tail_positions = HashSet::<_>::from_iter([[0i64; 2]]);

	try_for_each_line_in_file("inputs/day-09", |line| {
		let line = line.trim();
		let (direction, steps) = line
			.split_once(' ')
			.ok_or_else(|| eyre!("invalid entry format {line}"))?;
		let steps = steps.parse::<i64>()?;

		let (sign, axis) = match direction {
			"R" => (1, 0),
			"L" => (-1, 0),
			"U" => (1, 1),
			"D" => (-1, 1),
			_ => return Err(eyre!("invalid entry format {line}")),
		};

		for _ in 0..steps {
			knots[0][axis] += sign;
			for k in 1..K {
				let delta = [0, 1].map(|i| knots[k - 1][i] - knots[k][i]);
				if delta.iter().map(|d| d.abs()).max().expect("unreachable") <= 1 {
					break;
				}
				let update = delta.map(|d| d.signum());
				knots[k] = [0, 1].map(|i| knots[k][i] + update[i]);
				if k == K - 1 {
					tail_positions.insert(knots[k]);
				}
			}
		}

		Ok(())
	})?;

	let visited = tail_positions.len();
	println!("{visited}");

	Ok(())
}

fn solution() -> Result<()> {
	solve::<2>()?;
	solve::<10>()
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_09: (usize, fn() -> Result<()>) = (9, solution);
