use std::cmp::Ordering::*;

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

struct Instruction {
	count: usize,
	from: usize,
	to: usize,
}

fn strip_crlf(line: &str) -> &str {
	match line.strip_suffix('\n') {
		Some(line) => line.strip_suffix('\r').unwrap_or(line),
		None => line,
	}
}

fn read_input() -> Result<(Vec<Vec<u8>>, Vec<Instruction>)> {
	let [p0, p1, p2, p3] = [
		r"^(    |\[[A-Z]\] )*(   |\[[A-Z]\])$",
		r"^( \d  )+( \d )$",
		r"^move (\d+) from (\d+) to (\d+)$",
		r"^$",
	]
	.map(regex::Regex::new);
	let patterns = [p0?, p1?, p2?, p3?];

	let mut stacks: Vec<Vec<u8>> = Vec::new();
	let mut instructions = Vec::new();
	try_for_each_line_in_file("inputs/day-05", |line| {
		let line = strip_crlf(line);
		for (index, pattern) in patterns.iter().enumerate() {
			if let Some(captures) = pattern.captures(line) {
				match index {
					0 => {
						if stacks.is_empty() {
							stacks.resize_with(line.as_bytes().chunks(4).count(), Vec::new);
						}
						line.as_bytes().chunks(4).enumerate().for_each(|(i, c)| {
							if c[0] == b'[' {
								stacks[i].push(c[1]);
							}
						});
					}
					2 => {
						instructions.push(Instruction {
							count: captures.get(1).unwrap().as_str().parse::<usize>()?,
							from: captures.get(2).unwrap().as_str().parse::<usize>()? - 1,
							to: captures.get(3).unwrap().as_str().parse::<usize>()? - 1,
						});
					}
					1 | 3 => {}
					_ => unreachable!(),
				}
				return Ok(());
			}
		}

		Err(eyre!("invalid input"))
	})?;

	stacks.iter_mut().for_each(|stack| stack.reverse());
	Ok((stacks, instructions))
}

fn print_stack_tops(stacks: &Vec<Vec<u8>>) {
	for stack in stacks {
		if let Some(item) = stack.last() {
			let item = *item as char;
			print!("{item}")
		} else {
			print!(" ")
		}
	}
	println!();
}

fn solution() -> Result<()> {
	let (mut stacks, instructions) = read_input()?;
	let stacks_cloned = stacks.clone();

	for Instruction { count, from, to } in instructions.iter() {
		(0..*count).try_for_each(|_| -> Result<()> {
			let item = stacks[*from]
				.pop()
				.ok_or_else(|| eyre!("too many items removed from stack"))?;
			stacks[*to].push(item);
			Ok(())
		})?;
	}

	print_stack_tops(&stacks);

	let mut stacks = stacks_cloned;
	for Instruction { count, from, to } in instructions.iter() {
		let (from, to) = match from.cmp(to) {
			Less => {
				let (f, t) = stacks.split_at_mut(*to);
				(&mut f[*from], &mut t[0])
			}
			Equal => {
				continue;
			}
			Greater => {
				let (t, f) = stacks.split_at_mut(*from);
				(&mut f[0], &mut t[*to])
			}
		};

		let e = from.len();
		if e < *count {
			return Err(eyre!("too many items removed from stack"));
		}
		to.extend(from.drain(e - *count..e));
	}

	print_stack_tops(&stacks);

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_05: (usize, fn() -> Result<()>) = (5, solution);
