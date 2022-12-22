use std::collections::HashSet;

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use linkme::distributed_slice;
use ndarray::prelude::*;

fn solution() -> Result<()> {
	let mut surfaces: [_; 3] = array_init::array_init(|_| HashSet::new());
	let mut voxels = Vec::new();
	try_for_each_line_in_file("inputs/day-18", |line| {
		let line = line.trim();
		let pos: [_; 3] = array_init::from_iter(line.split(','))
			.ok_or_else(|| eyre!("incorrect number of commas"))?;
		let pos: [u8; 3] = array_init::try_array_init(|i| pos[i].parse())?;
		voxels.push(pos);
		for axis in 0..3 {
			let surfaces = &mut surfaces[axis];
			let mut side = pos;
			if !surfaces.insert(side) {
				surfaces.remove(&side);
			}
			side[axis] += 1;
			if !surfaces.insert(side) {
				surfaces.remove(&side);
			}
		}
		Ok(())
	})?;
	let surface_area = surfaces.iter().map(HashSet::len).sum::<usize>();
	println!("{surface_area}");

	let shape = [0, 1, 2].map(|i| (voxels.iter().map(|p| p[i]).max().unwrap() + 3) as usize);
	let mut map = Array3::from_elem(shape, 0i8);
	voxels.into_iter().for_each(|voxel| {
		let voxel = voxel.map(|i| (i + 1) as usize);
		map[voxel] = 1;
	});

	let mut queue = vec![[0usize; 3]];
	while let Some(voxel) = queue.pop() {
		map[voxel] = -1;
		(0..3)
			.cartesian_product([-1, 1])
			.for_each(|(axis, offset)| {
				let mut voxel = voxel;
				match voxel[axis].checked_add_signed(offset) {
					Some(i) if i < map.raw_dim()[axis] => voxel[axis] = i,
					_ => return,
				};

				if map[voxel] == 0 {
					queue.push(voxel);
				}
			});
	}

	let interior_surface_area = map
		.indexed_iter()
		.map(|(voxel, value)| match value {
			0 => (0..3)
				.cartesian_product([-1, 1])
				.filter(|&(axis, offset)| {
					let mut voxel = [voxel.0, voxel.1, voxel.2];
					voxel[axis] = voxel[axis].checked_add_signed(offset).unwrap();
					map[voxel] != 0
				})
				.count(),
			_ => 0,
		})
		.sum::<usize>();
	let exterior_surface_area = surface_area - interior_surface_area;
	println!("{exterior_surface_area}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_18: (usize, fn() -> Result<()>) = (18, solution);
