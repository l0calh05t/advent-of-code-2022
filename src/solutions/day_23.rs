use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

fn read_input() -> Result<HashSet<[isize; 2]>> {
	let mut result = HashSet::default();
	let mut row = 0;
	try_for_each_line_in_file("inputs/day-23", |line| {
		let line = line.trim();
		for (column, value) in line.as_bytes().iter().copied().enumerate() {
			match value {
				b'#' => {
					result.insert([row, column as isize]);
				}
				b'.' => {}
				_ => return Err(eyre!("invalid input {value}")),
			};
		}
		row += 1;
		Ok(())
	})?;
	Ok(result)
}

fn plan_move(
	elves: &HashSet<[isize; 2]>,
	round: usize,
	plan: &mut HashMap<[isize; 2], Option<[isize; 2]>>,
) {
	plan.clear();
	let mut directions = [
		0usize, // N
		4,      // S
		2,      // W
		6,      // E
	];
	directions.rotate_left(round % 4);

	'outer: for &elf in elves {
		let neighbor_positions = [
			[elf[0] - 1, elf[1]],     // N
			[elf[0] - 1, elf[1] - 1], // NW
			[elf[0], elf[1] - 1],     // W
			[elf[0] + 1, elf[1] - 1], // SW
			[elf[0] + 1, elf[1]],     // S
			[elf[0] + 1, elf[1] + 1], // SE
			[elf[0], elf[1] + 1],     // E
			[elf[0] - 1, elf[1] + 1], // NE
		];

		let neighbors = neighbor_positions.map(|elf| elves.contains(&elf));
		if neighbors.iter().all(|&n| !n) {
			continue;
		}

		for direction in directions {
			let offset = direction.checked_sub(1).unwrap_or(7);
			if neighbors.iter().cycle().skip(offset).take(3).all(|&n| !n) {
				plan.entry(neighbor_positions[direction])
					.and_modify(|e| *e = None)
					.or_insert(Some(elf));
				continue 'outer;
			}
		}
	}
}

fn run_plan(elves: &mut HashSet<[isize; 2]>, plan: &HashMap<[isize; 2], Option<[isize; 2]>>) {
	for (&k, v) in plan {
		if let Some(v) = v {
			elves.remove(v);
			elves.insert(k);
		}
	}
}

fn bounds(elves: &HashSet<[isize; 2]>) -> [[isize; 2]; 2] {
	let mut lo = [isize::MAX; 2];
	let mut hi = [isize::MIN; 2];
	for elf in elves {
		lo.iter_mut().zip(elf).for_each(|(b, v)| *b = (*b).min(*v));
		hi.iter_mut().zip(elf).for_each(|(b, v)| *b = (*b).max(*v));
	}
	[lo, hi]
}

fn empty_ground(elves: &HashSet<[isize; 2]>) -> usize {
	let [lo, hi] = bounds(elves);
	lo.into_iter()
		.zip(hi)
		.map(|(lo, hi)| (hi - lo + 1) as usize)
		.product::<usize>()
		- elves.len()
}

fn solution() -> Result<()> {
	let mut elves = read_input()?;
	let mut plan = HashMap::default();

	for round in 0.. {
		plan_move(&elves, round, &mut plan);
		if plan.values().all(Option::is_none) {
			println!("{}", round + 1);
			break;
		}
		run_plan(&mut elves, &plan);
		if round == 9 {
			println!("{}", empty_ground(&elves));
		}
	}

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_23: (usize, fn() -> Result<()>) = (23, solution);
