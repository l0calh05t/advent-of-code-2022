use crate::read_segmented;
use crate::SOLUTIONS;

use color_eyre::eyre::Result;
use linkme::distributed_slice;

fn solution() -> Result<()> {
	let entries = read_segmented::<u32, _>("inputs/day-01")?;

	let mut sums: Vec<u32> = entries
		.into_iter()
		.map(|segment| segment.into_iter().sum())
		.collect();

	let (top_two, third, _) = sums
		.as_mut_slice()
		.select_nth_unstable_by(2, |a, b| b.cmp(a));

	println!("{}", top_two.iter().max().expect("unreachable"));
	println!("{}", top_two.iter().sum::<u32>() + *third);

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_01: (usize, fn() -> Result<()>) = (1, solution);
