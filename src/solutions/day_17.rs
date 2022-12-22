use std::cmp::Ordering;

use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;
use ndarray::{concatenate, prelude::*};
use once_cell::sync::Lazy;

static ROCKS: Lazy<[Array2<u8>; 5]> = Lazy::new(|| {
	[
		arr2(&[[1, 1, 1, 1]]),
		arr2(&[[0, 1, 0], [1, 1, 1], [0, 1, 0]]),
		arr2(&[[1, 1, 1], [0, 0, 1], [0, 0, 1]]), // upside-down, since it's easier to reason about
		arr2(&[[1], [1], [1], [1]]),
		arr2(&[[1, 1], [1, 1]]),
	]
});

static GUSTS: Lazy<&[u8]> = Lazy::new(|| include_str!("../../inputs/day-17").trim().as_bytes());

struct Simulator {
	chamber: Array2<u8>,
	y_max: usize,
	rock_index: usize,
	rock: Option<(&'static Array2<u8>, (usize, usize))>,
	gust_index: usize,
}

impl Simulator {
	fn new() -> Self {
		Simulator {
			chamber: Array2::<u8>::zeros((4, 7)),
			y_max: 0,
			rock_index: 0,
			rock: None,
			gust_index: 0,
		}
	}

	fn reset(&mut self) {
		self.chamber.fill(0);
		self.y_max = 0;
		self.rock_index = 0;
		self.rock = None;
		self.gust_index = 0;
	}

	fn step(&mut self) -> Result<()> {
		let Simulator {
			chamber,
			y_max,
			rock_index,
			rock,
			gust_index,
		} = self;

		let the_rock = match rock.as_mut() {
			Some(rock) => rock,
			None => {
				let rock_selection = &ROCKS[*rock_index % ROCKS.len()];
				let rock_position = (*y_max + 3, 2);
				if rock_selection.dim().0 + rock_position.0 > chamber.dim().0 {
					*chamber = concatenate(
						Axis(0),
						&[chamber.view(), Array2::zeros(chamber.raw_dim()).view()],
					)?;
				}
				rock.get_or_insert((rock_selection, rock_position))
			}
		};

		let bottom = the_rock.1 .0;
		let top = bottom + the_rock.0.dim().0;
		match GUSTS[*gust_index % GUSTS.len()] {
			b'<' => {
				if the_rock.1 .1 > 0 && {
					let left = the_rock.1 .1 - 1;
					let right = left + the_rock.0.dim().1;
					ndarray::Zip::from(chamber.slice(s![bottom..top, left..right]))
						.and(the_rock.0)
						.all(|a, b| *a == 0 || *b == 0)
				} {
					the_rock.1 .1 -= 1;
				}
			}
			b'>' => {
				if the_rock.1 .1 + the_rock.0.dim().1 < chamber.dim().1 && {
					let left = the_rock.1 .1 + 1;
					let right = left + the_rock.0.dim().1;
					ndarray::Zip::from(chamber.slice(s![bottom..top, left..right]))
						.and(the_rock.0)
						.all(|a, b| *a == 0 || *b == 0)
				} {
					the_rock.1 .1 += 1;
				}
			}
			_ => {
				return Err(eyre!("invalid gust"));
			}
		};

		let left = the_rock.1 .1;
		let right = left + the_rock.0.dim().1;
		if the_rock.1 .0 > 0 && {
			let bottom = the_rock.1 .0 - 1;
			let top = bottom + the_rock.0.dim().0;
			ndarray::Zip::from(chamber.slice(s![bottom..top, left..right]))
				.and(the_rock.0)
				.all(|a, b| *a == 0 || *b == 0)
		} {
			the_rock.1 .0 -= 1;
		} else {
			let bottom = the_rock.1 .0;
			let top = bottom + the_rock.0.dim().0;
			ndarray::Zip::from(chamber.slice_mut(s![bottom..top, left..right]))
				.and(the_rock.0)
				.for_each(|c, &r| {
					*c |= r;
				});
			*y_max = (*y_max).max(top);
			*rock = None;
			*rock_index += 1;
		}

		*gust_index += 1;

		Ok(())
	}

	fn simulate_n_rocks(&mut self, n: usize) -> Result<usize> {
		self.reset();
		while self.rock_index < n {
			self.step()?;
		}
		Ok(self.y_max)
	}

	fn simulate_n_rocks_periodic(&mut self, n: usize) -> Result<usize> {
		self.reset();
		let period = GUSTS.len() * ROCKS.len();

		let y_0;
		let r_0;
		{
			let yrs = Some(Ok((0, 0)))
				.into_iter()
				.chain((0..period).map(|_| -> Result<_> {
					self.step()?;
					Ok((self.y_max, self.rock_index))
				}))
				.collect::<Result<Vec<_>>>()?;

			y_0 = self.y_max;
			r_0 = self.rock_index;

			match n.cmp(&r_0) {
				Ordering::Equal => return Ok(y_0),
				Ordering::Less => {
					return Ok(yrs[yrs.binary_search_by_key(&n, |(_, r)| *r).unwrap()].0)
				}
				_ => {}
			}
		}

		let yrs = Some(Ok((0, 0)))
			.into_iter()
			.chain((0..period).map(|_| -> Result<_> {
				self.step()?;
				Ok((self.y_max - y_0, self.rock_index - r_0))
			}))
			.collect::<Result<Vec<_>>>()?;

		#[cfg(debug_assertions)]
		{
			let (y_1, r_1) = yrs.last().map(|(y, r)| (y + y_0, r + r_0)).unwrap();

			let yrs_2 = Some(Ok((0, 0)))
				.into_iter()
				.chain((0..period).map(|_| -> Result<_> {
					self.step()?;
					Ok((self.y_max - y_1, self.rock_index - r_1))
				}))
				.collect::<Result<Vec<_>>>()?;

			debug_assert_eq!(yrs, yrs_2, "not periodic as expected");
		}

		let (y_p, r_p) = yrs.last().copied().unwrap();

		let n = n - r_0;
		let y = y_0
			+ y_p * (n / r_p)
			+ yrs[yrs.binary_search_by_key(&(n % r_p), |(_, r)| *r).unwrap()].0;

		Ok(y)
	}
}

fn solution() -> Result<()> {
	let mut simulator = Simulator::new();

	let y_max = simulator.simulate_n_rocks(2022)?;
	println!("{y_max}");

	let y_max_large = simulator.simulate_n_rocks_periodic(1_000_000_000_000)?;
	println!("{y_max_large}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_17: (usize, fn() -> Result<()>) = (17, solution);
