use std::{fs::File, io::Read};

use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use linkme::distributed_slice;
use ndarray::{prelude::*, ErrorKind::IncompatibleShape, ShapeError};
use pathfinding::directed::dijkstra::dijkstra;

type Position = (usize, usize);

fn read_map(file_name: &str) -> Result<(Array2<u8>, Position, Position)> {
	let mut bytes = Vec::new();
	File::open(file_name)?.read_to_end(&mut bytes)?;
	let mut lines = 0usize;
	let mut columns = None;
	let mut start_position = None;
	let mut end_position = None;
	let values = bytes
		.split_mut(|&b| b == b'\n')
		.map(|line| {
			if line.is_empty() {
				return Ok(line);
			}
			lines += 1;
			if let Some(columns) = columns {
				if columns != line.len() {
					return Err(ShapeError::from_kind(IncompatibleShape).into());
				}
			}
			columns = Some(line.len());
			line.iter_mut().enumerate().for_each(|(col, ch)| match ch {
				b'S' => {
					*ch = b'a';
					start_position = Some((lines - 1, col));
				}
				b'E' => {
					*ch = b'z';
					end_position = Some((lines - 1, col));
				}
				_ => {}
			});
			Ok(line)
		})
		.flatten_ok()
		.map_ok(|&mut b| b.checked_sub(b'a').ok_or_else(|| eyre!("invalid height")))
		.flatten_ok()
		.collect::<Result<Vec<_>>>()?;
	Ok((
		Array2::from_shape_vec((lines, columns.unwrap_or(0)), values)?,
		start_position.ok_or_else(|| eyre!("no start position"))?,
		end_position.ok_or_else(|| eyre!("no end position"))?,
	))
}

fn solution() -> Result<()> {
	let (map, start, end) = read_map("inputs/day-12")?;

	let neighbors = |(r, c): (usize, usize)| {
		[
			r.checked_sub(1).map(|r| (r, c)),
			c.checked_sub(1).map(|c| (r, c)),
			(r + 1 < map.dim().0).then_some((r + 1, c)),
			(c + 1 < map.dim().1).then_some((r, c + 1)),
		]
	};

	let (_, cost) = dijkstra(
		&start,
		|&p| {
			neighbors(p).into_iter().filter_map({
				let hp1 = map[p] + 1;
				let map = &map;
				move |p: Option<(usize, usize)>| p.and_then(|p| (map[p] <= hp1).then_some((p, 1)))
			})
		},
		|p| p == &end,
	)
	.ok_or_else(|| eyre!("no path to end"))?;
	println!("{cost}");

	let (_, cost) = dijkstra(
		&end,
		|&p| {
			neighbors(p).into_iter().filter_map({
				let h = map[p];
				let map = &map;
				move |p: Option<(usize, usize)>| p.and_then(|p| (h <= map[p] + 1).then_some((p, 1)))
			})
		},
		|&p| map[p] == 0,
	)
	.ok_or_else(|| eyre!("no path to start"))?;
	println!("{cost}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_12: (usize, fn() -> Result<()>) = (12, solution);
