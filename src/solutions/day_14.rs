use std::{
	cmp::Ordering,
	fmt::{Display, Write},
};

use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use derive_more::IntoIterator;
use itertools::Itertools;
use linkme::distributed_slice;
use ndarray::prelude::*;
use nom::{
	bytes::complete::tag,
	character::complete::{char, digit1, line_ending},
	combinator::{eof, map, map_res, opt},
	multi::{separated_list0, separated_list1},
	sequence::{pair, separated_pair, terminated},
	IResult,
};

#[derive(Debug, Clone, Copy)]
struct Position([isize; 2]);

impl Position {
	fn parse(i: &str) -> IResult<&str, Self, ()> {
		map(
			separated_pair(
				map_res(digit1, |s: &str| s.parse()),
				char(','),
				map_res(digit1, |s: &str| s.parse()),
			),
			|t| Self([t.0, t.1]),
		)(i)
	}
}

#[derive(Debug, Clone, IntoIterator)]
struct LineStrip(#[into_iterator(owned, ref)] Vec<Position>);

impl LineStrip {
	fn parse(i: &str) -> IResult<&str, Self, ()> {
		map(separated_list1(tag(" -> "), Position::parse), Self)(i)
	}
}

fn parse_input(i: &str) -> IResult<&str, Vec<LineStrip>, ()> {
	terminated(
		separated_list0(line_ending, LineStrip::parse),
		pair(opt(line_ending), eof),
	)(i)
}

#[derive(Debug, Clone, Copy)]
struct Extent {
	min: [isize; 2],
	max: [isize; 2],
}

impl Extent {
	fn new() -> Self {
		Self {
			min: [500, 0],
			max: [500, 0],
		}
	}

	fn extend(&mut self, p: &Position) {
		// loop since array_zip is unstable
		for i in 0..2 {
			self.min[i] = self.min[i].min(p.0[i]);
			self.max[i] = self.max[i].max(p.0[i]);
		}
	}

	fn from_iter<'a, I: Iterator<Item = &'a Position>>(iter: I) -> Self {
		let mut result = Self::new();
		iter.for_each(|p| result.extend(p));
		result
	}

	fn size(&self) -> [usize; 2] {
		[
			(self.max[0] - self.min[0] + 1) as usize,
			(self.max[1] - self.min[1] + 1) as usize,
		]
	}

	fn adjust(&self, mut p: Position) -> [usize; 2] {
		// loop since array_zip is unstable
		for i in 0..2 {
			p.0[i] -= self.min[i];
		}
		p.0.map(|i| i as usize)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum Tile {
	Source = b'+',
	Empty = b'.',
	Wall = b'#',
	Sand = b'o',
}

#[derive(Clone, Debug)]
struct Map {
	tiles: Array2<Tile>,
	source: [usize; 2],
}

impl Map {
	fn from_walls(walls: &Vec<LineStrip>) -> Result<Self> {
		let extent = Extent::from_iter(walls.iter().flatten());
		let size = extent.size();
		let mut tiles = Array2::from_shape_simple_fn(size, || Tile::Empty);
		for strip in walls {
			if let Some(start) = strip.0.first() {
				let start = extent.adjust(*start);
				tiles[start] = Tile::Wall;
			}
			strip
				.0
				.iter()
				.tuple_windows()
				.try_for_each(|(start, end)| {
					let start = extent.adjust(*start);
					let end = extent.adjust(*end);
					match [start[0].cmp(&end[0]), start[1].cmp(&end[1])] {
						[Ordering::Less, Ordering::Equal] => {
							for i in (start[0] + 1)..=end[0] {
								tiles[[i, start[1]]] = Tile::Wall;
							}
						}
						[Ordering::Greater, Ordering::Equal] => {
							for i in end[0]..start[0] {
								tiles[[i, start[1]]] = Tile::Wall;
							}
						}
						[Ordering::Equal, Ordering::Less] => {
							for j in (start[1] + 1)..=end[1] {
								tiles[[start[0], j]] = Tile::Wall;
							}
						}
						[Ordering::Equal, Ordering::Greater] => {
							for j in end[1]..start[1] {
								tiles[[start[0], j]] = Tile::Wall;
							}
						}
						[Ordering::Equal, Ordering::Equal] => {}
						_ => return Err(eyre!("no such thing as a diagonal wall")),
					}
					Ok(())
				})?;
		}

		let source = extent.adjust(Position([500, 0]));
		tiles[source] = Tile::Source;

		Ok(Self { tiles, source })
	}
}

impl Display for Map {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for line in self.tiles.columns() {
			for tile in line.iter().copied() {
				f.write_char(tile as u8 as char)?;
			}
			f.write_char('\n')?;
		}
		Ok(())
	}
}

fn simulate(mut map: Map) {
	let mut path = vec![map.source];
	let mut grains = 0usize;
	let mut print_grains = 1usize;
	while let Some(pos) = path.pop() {
		if pos[1] + 1 >= map.tiles.raw_dim()[1] || matches!(map.tiles[pos], Tile::Wall | Tile::Sand)
		{
			path.clear();
			continue;
		}

		let below = map.tiles[[pos[0], pos[1] + 1]];
		let below_left = (pos[0] > 0).then(|| map.tiles[[pos[0] - 1, pos[1] + 1]]);
		let below_right =
			(pos[0] + 1 < map.tiles.raw_dim()[0]).then(|| map.tiles[[pos[0] + 1, pos[1] + 1]]);

		if below == Tile::Empty {
			path.push(pos);
			path.push([pos[0], pos[1] + 1]);
		} else if below_left == Some(Tile::Empty) {
			path.push(pos);
			path.push([pos[0] - 1, pos[1] + 1]);
		} else if below_left.is_none() {
			path.clear();
		} else if below_right == Some(Tile::Empty) {
			path.push(pos);
			path.push([pos[0] + 1, pos[1] + 1]);
		} else if below_right.is_none() {
			path.clear();
		} else {
			map.tiles[pos] = Tile::Sand;
			grains += 1;
			if grains == print_grains {
				// println!("{map}");
				print_grains <<= 1;
			}
		}
	}

	// if grains != print_grains {
	// 	println!("{map}");
	// }

	println!("{grains}");
}

fn solution() -> Result<()> {
	let input = std::fs::read_to_string("inputs/day-14")?;
	let (_, mut walls) = parse_input(&input)?;
	simulate(Map::from_walls(&walls)?);

	// compute a bottom wall that is guaranteed to be oversized (could optimize a bit)
	let mut extent = Extent::from_iter(walls.iter().flatten());
	extent.max[1] += 2;
	let l = extent.min[0] - extent.max[1];
	let r = extent.max[0] + extent.max[1];
	let b = extent.max[1];
	walls.push(LineStrip(vec![Position([l, b]), Position([r, b])]));

	simulate(Map::from_walls(&walls)?);

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_14: (usize, fn() -> Result<()>) = (14, solution);
