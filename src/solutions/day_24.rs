use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;
use ndarray::{concatenate, prelude::*};
use pathfinding::directed::astar::astar;

const WIND_SYMBOLS: [u8; 4] = [b'^', b'<', b'v', b'>'];
const WIND_DIRECTIONS: [[isize; 2]; 4] = [[-1isize, 0], [0, -1], [1, 0], [0, 1]];

fn read_input() -> Result<Array3<bool>> {
	let mut blizzards = Array3::from_elem((4, 0, 0), false);
	let mut valid_lines = 0;
	let mut done = false;
	try_for_each_line_in_file("inputs/day-24", |line| {
		let line = line.trim();

		if done {
			return Err(eyre!("unexpected line after input: {line}"));
		}

		if blizzards.dim().1 == 0 {
			let line = line
				.strip_prefix("#.")
				.ok_or_else(|| eyre!("start not at expected position"))?;
			if !line.as_bytes().iter().all(|&b| b == b'#') {
				return Err(eyre!("missing northern wall"));
			}
			blizzards = Array3::from_elem((4, 1, line.len()), false);
			return Ok(());
		} else if line
			.strip_suffix(".#")
			.map(|line| line.as_bytes().iter().all(|&b| b == b'#'))
			== Some(true)
		{
			// in the future, the above condition could be simplified using is_some_and
			done = true;
			return Ok(());
		}

		let line = line
			.strip_prefix('#')
			.and_then(|line| line.strip_suffix('#'))
			.ok_or_else(|| eyre!("missing side walls"))?
			.as_bytes();

		if line.len() != blizzards.dim().2 {
			return Err(eyre!("mismatched line width"));
		}

		if line.iter().any(|b| *b != b'.' && !WIND_SYMBOLS.contains(b)) {
			return Err(eyre!("invalid input"));
		}

		if valid_lines >= blizzards.dim().1 {
			let dim = blizzards.raw_dim();
			blizzards = concatenate(
				Axis(1),
				&[blizzards.view(), Array3::from_elem(dim, false).view()],
			)?;
		}

		blizzards
			.axis_iter_mut(Axis(0))
			.zip(WIND_SYMBOLS)
			.for_each(|(mut blizzard, marker)| {
				blizzard
					.slice_mut(s!(valid_lines, ..))
					.iter_mut()
					.zip(line)
					.for_each(|(e, &v)| {
						*e = v == marker;
					})
			});

		valid_lines += 1;

		Ok(())
	})?;

	if !done {
		return Err(eyre!("missing southern wall"));
	}

	let blizzards = blizzards.slice(s!(.., 0..valid_lines, ..)).to_owned();

	if blizzards.slice(s!(0, .., 0)).iter().any(|&b| b)
		|| blizzards.slice(s!(0, .., -1)).iter().any(|&b| b)
		|| blizzards.slice(s!(2, .., 0)).iter().any(|&b| b)
		|| blizzards.slice(s!(2, .., -1)).iter().any(|&b| b)
	{
		return Err(eyre!("unexpected blizzards at sides"));
	}

	Ok(blizzards)
}

fn blizzards_at_point_and_time(
	[row, col]: [usize; 2],
	blizzards: &Array3<bool>,
	time: usize,
) -> [bool; 4] {
	let (directions, rows, cols) = blizzards.dim();
	assert_eq!(directions, 4);
	array_init::array_init(|i| {
		let [v_row, v_col] = WIND_DIRECTIONS[i];
		let row = (row as isize - time as isize * v_row).rem_euclid(rows as isize) as usize;
		let col = (col as isize - time as isize * v_col).rem_euclid(cols as isize) as usize;
		blizzards[(i, row, col)]
	})
}

fn any_blizzards_at_point_and_time(
	position: [usize; 2],
	blizzards: &Array3<bool>,
	time: usize,
) -> bool {
	blizzards_at_point_and_time(position, blizzards, time)
		.into_iter()
		.any(|b| b)
}

fn _print_at_time(blizzards: &Array3<bool>, time: usize) {
	let (_, rows, cols) = blizzards.dim();
	for row in 0..rows {
		for col in 0..cols {
			let blizzards = blizzards_at_point_and_time([row, col], blizzards, time);
			let n = blizzards.iter().filter(|&&b| b).count();
			match n {
				0 => print!("."),
				1 => blizzards.iter().zip(WIND_SYMBOLS).for_each(|(&b, s)| {
					if b {
						print!("{}", s as char)
					}
				}),
				_ => print!("{n}"),
			}
		}
		println!();
	}
	println!();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Position {
	Start,
	End,
	Interior([usize; 2]),
}

impl Position {
	fn interior(self) -> Option<[usize; 2]> {
		match self {
			Position::Interior(p) => Some(p),
			_ => None,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
	position: Position,
	time: usize,
}

impl State {
	fn new() -> State {
		State {
			position: Position::Start,
			time: 0,
		}
	}

	fn successors_interior<'a>(
		&self,
		blizzards: &'a Array3<bool>,
	) -> impl IntoIterator<Item = State> + 'a {
		let State { position, time } = *self;
		let time = time + 1;
		position.interior().into_iter().flat_map(move |p| {
			[[-1isize, 0], [0, -1], [1, 0], [0, 1], [0, 0]]
				.into_iter()
				.filter_map(move |[vr, vc]| {
					let [r, c] = p;
					let r = r.checked_add_signed(vr)?;
					let c = c.checked_add_signed(vc)?;
					if r >= blizzards.dim().1
						|| c >= blizzards.dim().2
						|| any_blizzards_at_point_and_time([r, c], blizzards, time)
					{
						return None;
					}
					Some(State {
						position: Position::Interior([r, c]),
						time,
					})
				})
		})
	}

	fn successors<'a>(
		&self,
		blizzards: &'a Array3<bool>,
	) -> impl IntoIterator<Item = (State, usize)> + 'a {
		let State { position, time } = *self;
		let time = time + 1;
		let start = (position == Position::Start)
			.then(|| {
				[
					(!any_blizzards_at_point_and_time([0, 0], blizzards, time)).then_some(State {
						position: Position::Interior([0, 0]),
						time,
					}),
					Some(State { position, time }),
				]
				.into_iter()
				.flatten()
			})
			.into_iter()
			.flatten();
		let end = (position == Position::Interior([blizzards.dim().1 - 1, blizzards.dim().2 - 1]))
			.then_some(State {
				position: Position::End,
				time,
			});
		start
			.chain(end)
			.chain(self.successors_interior(blizzards))
			.map(|state| (state, 1))
	}

	fn successors_rev<'a>(
		&self,
		blizzards: &'a Array3<bool>,
	) -> impl IntoIterator<Item = (State, usize)> + 'a {
		let State { position, time } = *self;
		let time = time + 1;
		let start = (position == Position::End)
			.then(|| {
				let start = [blizzards.dim().1 - 1, blizzards.dim().2 - 1];
				[
					(!any_blizzards_at_point_and_time(start, blizzards, time)).then_some(State {
						position: Position::Interior(start),
						time,
					}),
					Some(State { position, time }),
				]
				.into_iter()
				.flatten()
			})
			.into_iter()
			.flatten();
		let end = (position == Position::Interior([0, 0])).then_some(State {
			position: Position::Start,
			time,
		});
		start
			.chain(end)
			.chain(self.successors_interior(blizzards))
			.map(|state| (state, 1))
	}

	fn heuristic(&self, blizzards: &Array3<bool>) -> usize {
		match self.position {
			Position::Start => blizzards.dim().1 + blizzards.dim().2,
			Position::End => 0,
			Position::Interior([r, c]) => blizzards.dim().1 - r + blizzards.dim().2 - c - 1,
		}
	}

	fn heuristic_rev(&self, blizzards: &Array3<bool>) -> usize {
		match self.position {
			Position::Start => 0,
			Position::End => blizzards.dim().1 + blizzards.dim().2,
			Position::Interior([r, c]) => r + c + 1,
		}
	}

	fn success(&self) -> bool {
		self.position == Position::End
	}

	fn success_rev(&self) -> bool {
		self.position == Position::Start
	}
}

fn solution() -> Result<()> {
	let blizzards = read_input()?;

	let (path, time_1) = astar(
		&State::new(),
		|state| state.successors(&blizzards),
		|state| state.heuristic(&blizzards),
		State::success,
	)
	.ok_or_else(|| eyre!("no path to exit"))?;

	println!("{time_1}");

	let (path, time_2) = astar(
		path.last().unwrap(),
		|state| state.successors_rev(&blizzards),
		|state| state.heuristic_rev(&blizzards),
		State::success_rev,
	)
	.ok_or_else(|| eyre!("no path to entrance"))?;

	let (_, time_3) = astar(
		path.last().unwrap(),
		|state| state.successors(&blizzards),
		|state| state.heuristic(&blizzards),
		State::success,
	)
	.ok_or_else(|| eyre!("no path to entrance"))?;

	let time_with_return = time_1 + time_2 + time_3;
	println!("{time_with_return}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_24: (usize, fn() -> Result<()>) = (24, solution);
