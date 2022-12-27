use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;
use ndarray::{concatenate, prelude::*};
use num::Integer;

#[derive(Clone, Copy, Debug)]
enum Instruction {
	Forward(usize),
	Clockwise,
	Counterclockwise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Tile {
	Empty = b'.',
	Wall = b'#',
	Warp = b' ',
}

#[derive(Clone, Copy, Debug)]
struct Turtle {
	position: [usize; 2],
	facing: [i8; 2],
}

impl Turtle {
	fn new(map: ArrayView2<'_, Tile>) -> Result<Turtle> {
		let start = map
			.rows()
			.into_iter()
			.next()
			.ok_or_else(|| eyre!("empty map!"))?
			.into_iter()
			.position(|t| matches!(t, Tile::Empty))
			.ok_or_else(|| eyre!("no empty tiles in topmost row!"))?;

		let turtle = Turtle {
			position: [0, start],
			facing: [0, 1],
		};

		Ok(turtle)
	}

	fn process_instruction(&mut self, instruction: Instruction, map: ArrayView2<'_, Tile>) {
		match instruction {
			Instruction::Forward(n) => {
				for _ in 0..n {
					let mut next_tile = self.position;
					while {
						next_tile
							.iter_mut()
							.zip(self.facing)
							.zip(map.shape())
							.for_each(|((x, v), d)| {
								*x = (*x as isize + v as isize).rem_euclid(*d as isize) as usize;
							});
						map[next_tile] == Tile::Warp
					} {}
					if map[next_tile] != Tile::Empty {
						break;
					}
					self.position = next_tile;
				}
			}
			Instruction::Clockwise => {
				self.facing = [self.facing[1], -self.facing[0]];
			}
			Instruction::Counterclockwise => {
				self.facing = [-self.facing[1], self.facing[0]];
			}
		}
	}

	fn process_instruction_on_cube(
		&mut self,
		instruction: Instruction,
		map: ArrayView2<'_, Tile>,
		cube_map: [[FaceWithFrame; 2]; 3],
	) {
		match instruction {
			Instruction::Forward(n) => {
				for _ in 0..n {
					let mut next_tile = self.position.map(|i| i as isize);
					next_tile.iter_mut().zip(self.facing).for_each(|(x, v)| {
						*x += v as isize;
					});
					let mut next_facing = self.facing;

					if next_tile
						.iter()
						.zip(map.shape())
						.any(|(&x, &d)| x < 0 || x as usize >= d)
						|| map[next_tile.map(|i| i as usize)] == Tile::Warp
					{
						let current_face = self.position.map(|i| i / FACE_SIZE);
						let (_, current_frame) = cube_map
							.iter()
							.copied()
							.flatten()
							.find(|f| f.0 == current_face)
							.unwrap();
						let mut expected_next_frame = match self.facing {
							[0, 1] => current_frame.next_j(),
							[0, -1] => current_frame.prev_j(),
							[1, 0] => current_frame.next_i(),
							[-1, 0] => current_frame.prev_i(),
							_ => unreachable!(),
						};
						let (next_axis, next_negated) = expected_next_frame.to_cube_indices();
						let (next_face, next_frame) = cube_map[next_axis][next_negated];
						next_tile = next_tile.map(|e| e.rem_euclid(FACE_SIZE as isize));
						if expected_next_frame.0[0].iter().position(|e| *e != 0)
							!= next_frame.0[0].iter().position(|e| *e != 0)
						{
							expected_next_frame.0 = [
								expected_next_frame.0[1],
								expected_next_frame.0[0],
								expected_next_frame.0[2],
							];
							next_tile = [next_tile[1], next_tile[0]];
							next_facing = [next_facing[1], next_facing[0]];
						}
						for i in 0..2 {
							if expected_next_frame.0[i] == next_frame.0[i] {
								continue;
							}
							expected_next_frame.0[i] = expected_next_frame.0[i].map(|e| -e);
							next_tile[i] = FACE_SIZE as isize - next_tile[i] - 1;
							next_facing[i] = -next_facing[i];
						}
						assert_eq!(expected_next_frame, next_frame);
						next_tile.iter_mut().zip(next_face).for_each(|(t, f)| {
							*t += (FACE_SIZE * f) as isize;
						});
					}

					let next_tile = next_tile.map(|i| i as usize);
					assert_ne!(map[next_tile], Tile::Warp);
					if map[next_tile] != Tile::Empty {
						break;
					}
					self.position = next_tile;
					self.facing = next_facing;
				}
			}
			_ => self.process_instruction(instruction, map),
		}
	}

	fn value(&self) -> usize {
		1000 * (self.position[0] + 1)
			+ 4 * (self.position[1] + 1)
			+ match self.facing {
				[0, 1] => 0,
				[1, 0] => 1,
				[0, -1] => 2,
				[-1, 0] => 3,
				_ => unreachable!(),
			}
	}
}

const FACE_SIZE: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Frame([[i8; 3]; 3]);

impl Frame {
	fn new() -> Frame {
		Frame([[1, 0, 0], [0, 1, 0], [0, 0, 1]])
	}

	fn next_i(self) -> Frame {
		let map_i = self.0[2].map(|e| -e);
		let map_j = self.0[1];
		let normal = self.0[0];
		Frame([map_i, map_j, normal])
	}

	fn prev_i(self) -> Frame {
		let map_i = self.0[2];
		let map_j = self.0[1];
		let normal = self.0[0].map(|e| -e);
		Frame([map_i, map_j, normal])
	}

	fn next_j(self) -> Frame {
		let map_i = self.0[0];
		let map_j = self.0[2].map(|e| -e);
		let normal = self.0[1];
		Frame([map_i, map_j, normal])
	}

	fn prev_j(self) -> Frame {
		let map_i = self.0[0];
		let map_j = self.0[2];
		let normal = self.0[1].map(|e| -e);
		Frame([map_i, map_j, normal])
	}

	fn to_cube_indices(self) -> (usize, usize) {
		let axis = self.0[2].iter().position(|&e| e != 0).unwrap();
		let negative = (self.0[2][axis] < 0) as usize;
		(axis, negative)
	}
}

type FaceWithFrame = ([usize; 2], Frame);

fn fold_cube(map: &ArrayView2<'_, Tile>) -> Result<[[FaceWithFrame; 2]; 3]> {
	if !map.shape().iter().all(|d| d.is_multiple_of(&FACE_SIZE)) {
		return Err(eyre!(
			"map does not consist of {FACE_SIZE}×{FACE_SIZE} tiles!"
		));
	}

	let ni = map.shape()[0] / FACE_SIZE;
	let nj = map.shape()[1] / FACE_SIZE;

	fn is_face([i, j]: [usize; 2], map: &ArrayView2<'_, Tile>) -> Result<bool> {
		let i_first = FACE_SIZE * i;
		let i_last = FACE_SIZE * (i + 1);
		let j_first = FACE_SIZE * j;
		let j_last = FACE_SIZE * (j + 1);

		let face = map.slice(s!(i_first..i_last, j_first..j_last));
		if face.iter().all(|&t| t == Tile::Warp) {
			return Ok(false);
		} else if face.iter().any(|&t| t == Tile::Warp) {
			return Err(eyre!("unexpected warp tile on face"));
		}

		Ok(true)
	}

	let j0 = (0..nj)
		.position(|j| matches!(is_face([0, j], map), Ok(true)))
		.ok_or_else(|| eyre!("no valid face in first row"))?;

	let mut cube_map = [[None; 2]; 3];
	let mut visited = Array2::from_elem((ni, nj), false);

	fn fold_recursion<'a>(
		ij: [usize; 2],
		cube_map: &mut [[Option<FaceWithFrame>; 2]; 3],
		map: &'a ArrayView2<'_, Tile>,
		visited: &mut Array2<bool>,
		frame: Frame,
	) -> Result<()> {
		if visited[ij] {
			return Ok(());
		}
		visited[ij] = true;

		if !is_face(ij, map)? {
			return Ok(());
		}

		let (axis, negative) = frame.to_cube_indices();
		if cube_map[axis][negative].is_some() {
			return Err(eyre!("invalid folding pattern"));
		}

		cube_map[axis][negative] = Some((ij, frame));

		let [i, j] = ij;
		let (ni, nj) = visited.dim();
		if i > 0 {
			fold_recursion([i - 1, j], cube_map, map, visited, frame.prev_i())?;
		}
		if i + 1 < ni {
			fold_recursion([i + 1, j], cube_map, map, visited, frame.next_i())?;
		}
		if j > 0 {
			fold_recursion([i, j - 1], cube_map, map, visited, frame.prev_j())?;
		}
		if j + 1 < nj {
			fold_recursion([i, j + 1], cube_map, map, visited, frame.next_j())?;
		}

		Ok(())
	}
	fold_recursion([0, j0], &mut cube_map, map, &mut visited, Frame::new())?;

	let result = array_init::try_array_init(|axis| {
		array_init::try_array_init(|negative| {
			cube_map[axis][negative]
				.ok_or_else(|| eyre!("folded map does not cover all cube faces"))
		})
	})?;
	Ok(result)
}

fn read_input() -> Result<(Array2<Tile>, Vec<Instruction>)> {
	let mut map = Array2::from_elem((0, 0), Tile::Warp);
	let mut active_rows = 0;
	let mut in_map = true;
	let mut instructions = None;
	try_for_each_line_in_file("inputs/day-22", |line| {
		let line = line.trim_end();

		// separator
		if line.is_empty() {
			in_map = false;
			return Ok(());
		}

		// instructions
		if !in_map {
			if instructions.is_some() {
				return Err(eyre!("only one set of instructions expected"));
			}

			let instructions = instructions.get_or_insert(vec![]);
			for part in line.split_inclusive(&['R', 'L']) {
				let (movement, turn) = if let Some(part) = part.strip_suffix('R') {
					(part, Some(Instruction::Clockwise))
				} else if let Some(part) = part.strip_suffix('L') {
					(part, Some(Instruction::Counterclockwise))
				} else {
					(part, None)
				};
				instructions.push(Instruction::Forward(movement.parse()?));
				if let Some(turn) = turn {
					instructions.push(turn);
				}
			}

			return Ok(());
		}

		// map
		let line = line.as_bytes();

		if active_rows >= map.raw_dim()[0] {
			let mut dim = map.raw_dim();
			dim[0] = 1.max(dim[0]);
			map = concatenate(
				Axis(0),
				&[map.view(), Array2::from_elem(dim, Tile::Warp).view()],
			)?;
		}

		if line.len() > map.raw_dim()[1] {
			let mut dim = map.raw_dim();
			dim[1] = line.len() - dim[1];
			map = concatenate(
				Axis(1),
				&[map.view(), Array2::from_elem(dim, Tile::Warp).view()],
			)?;
		}

		for column in 0..line.len() {
			let tile = match line[column] {
				b'.' => Tile::Empty,
				b'#' => Tile::Wall,
				b' ' => Tile::Warp,
				_ => return Err(eyre!("invalid map tile {}", line[column])),
			};
			map[(active_rows, column)] = tile;
		}

		active_rows += 1;

		Ok(())
	})?;

	let instructions = instructions.ok_or_else(|| eyre!("no instructions in input"))?;
	let map = map.slice(s!(0..active_rows, ..));

	Ok((map.to_owned(), instructions))
}

fn solution() -> Result<()> {
	let (map, instructions) = read_input()?;
	let map = map.view();

	let mut turtle = Turtle::new(map)?;
	for instruction in &instructions {
		turtle.process_instruction(*instruction, map);
	}
	println!("{}", turtle.value());

	let cube_map = fold_cube(&map)?;
	let mut turtle = Turtle::new(map)?;
	for instruction in instructions {
		turtle.process_instruction_on_cube(instruction, map, cube_map);
	}
	println!("{}", turtle.value());

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_22: (usize, fn() -> Result<()>) = (22, solution);
