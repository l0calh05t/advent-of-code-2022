use crate::read_digit_field;
use crate::SOLUTIONS;

use color_eyre::eyre::Result;
use itertools::{
	FoldWhile::{Continue, Done},
	Itertools,
};
use linkme::distributed_slice;
use ndarray::prelude::*;

fn perform_fold<'a, 'b, I>(iter: I)
where
	I: Iterator<Item = (&'a u8, &'b mut u8)>,
{
	iter.fold(-1i16, |max_height, (input, visible)| {
		if *input as i16 > max_height {
			*visible = 1;
			*input as _
		} else {
			max_height
		}
	});
}

fn perform_fold_2<'a, I>(v: u8, mut iter: I) -> u64
where
	I: Iterator<Item = &'a u8>,
{
	iter.fold_while(0, |mut d, n| {
		d += 1;
		if *n < v {
			Continue(d)
		} else {
			Done(d)
		}
	})
	.into_inner()
}

fn solution() -> Result<()> {
	let input = read_digit_field("inputs/day-08")?;
	let mut visible = Array2::<u8>::zeros(input.raw_dim());
	input.axes().map(|ad| ad.axis).for_each(|axis| {
		input
			.lanes(axis)
			.into_iter()
			.zip_eq(visible.lanes_mut(axis))
			.for_each(|(input, mut visible)| {
				perform_fold(input.iter().zip_eq(visible.iter_mut()));
				// zip_eq doesn't support rev
				perform_fold(input.iter().zip(visible.iter_mut()).rev());
			});
	});

	let count = visible.map(|v| *v as u64).sum();
	println!("{count}");

	let best_scenic_score = input
		.indexed_iter()
		.map(|(i, v)| {
			let i = [i.0, i.1];
			(0..2)
				.map(|k| {
					let (before, after) =
						input.index_axis(Axis(k), i[k]).split_at(Axis(0), i[k ^ 1]);
					let (_, after) = after.split_at(Axis(0), 1);
					let before = perform_fold_2(*v, before.iter().rev());
					let after = perform_fold_2(*v, after.iter());
					before * after
				})
				.product::<u64>()
		})
		.max()
		.unwrap_or(0);
	println!("{best_scenic_score}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_08: (usize, fn() -> Result<()>) = (8, solution);
