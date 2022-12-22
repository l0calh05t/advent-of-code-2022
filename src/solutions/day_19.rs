use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;
use rayon::prelude::*;
use regex::Regex;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(align(8))]
struct Blueprint {
	ore_robot_cost: u8,
	clay_robot_cost: u8,
	obsidian_robot_cost: (u8, u8),
	geode_robot_cost: (u8, u8),
}

impl Blueprint {
	fn map_costs(&self, index: usize) -> [u8; 3] {
		match index {
			0 => [self.ore_robot_cost, 0, 0],
			1 => [self.clay_robot_cost, 0, 0],
			2 => [self.obsidian_robot_cost.0, self.obsidian_robot_cost.1, 0],
			3 => [self.geode_robot_cost.0, 0, self.geode_robot_cost.1],
			_ => panic!("index out of range"),
		}
	}

	fn max_costs(&self) -> [u8; 3] {
		let mut max_costs = [0; 3];
		for i in 0..4 {
			let costs = self.map_costs(i);
			max_costs = array_init::array_init(|j| max_costs[j].max(costs[j]));
		}
		max_costs
	}
}

fn read_blueprints() -> Result<Vec<Blueprint>> {
	let pattern = Regex::new(
		r"Blueprint (\d+): Each ore robot costs (\d+) ore. Each clay robot costs (\d+) ore. Each obsidian robot costs (\d+) ore and (\d+) clay. Each geode robot costs (\d+) ore and (\d+) obsidian.",
	)?;
	let mut blueprints = vec![];
	try_for_each_line_in_file("inputs/day-19", |line| {
		let line = line.trim();
		let captures = pattern
			.captures(line)
			.ok_or_else(|| eyre!("unexpected input '{line}'"))?;
		let blueprint_index = captures.get(1).unwrap().as_str().parse::<usize>()?;
		if blueprint_index != blueprints.len() + 1 {
			return Err(eyre!("input blueprints incorrectly sorted"));
		}
		blueprints.push(Blueprint {
			ore_robot_cost: captures.get(2).unwrap().as_str().parse()?,
			clay_robot_cost: captures.get(3).unwrap().as_str().parse()?,
			obsidian_robot_cost: (
				captures.get(4).unwrap().as_str().parse()?,
				captures.get(5).unwrap().as_str().parse()?,
			),
			geode_robot_cost: (
				captures.get(6).unwrap().as_str().parse()?,
				captures.get(7).unwrap().as_str().parse()?,
			),
		});
		Ok(())
	})?;
	Ok(blueprints)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
	time: u8,
	robots: [u8; 4],
	minerals: [u8; 4],
}

impl State {
	fn new(time: u8) -> State {
		State {
			time,
			robots: [1, 0, 0, 0],
			minerals: [0; 4],
		}
	}
}

fn maximum_geodes(blueprint: &Blueprint, time: u8) -> u8 {
	let costs: [_; 4] = array_init::array_init(|i| blueprint.map_costs(i));
	let max_costs = blueprint.max_costs();
	let mut best = 0;
	let mut queue = vec![State::new(time)];
	while let Some(state) = queue.pop() {
		let State {
			time,
			robots,
			minerals,
		} = state;

		// upper bound can exceed u8 range
		let upper_bound = {
			let mut added_robots = [0; 4];
			let mut minerals = minerals.map(|m| m as u16);
			for _ in 0..time {
				minerals =
					array_init::array_init(|i| minerals[i] + robots[i] as u16 + added_robots[i]);
				added_robots =
					array_init::array_init(|i| {
						added_robots[i]
							+ costs[i].iter().copied().zip(minerals.iter().copied()).all(
								|(cost, mineral)| mineral >= cost as u16 * (added_robots[i] + 1),
							) as u16
					});
			}
			minerals[3]
		};
		if upper_bound < best as u16 {
			continue;
		}
		let lower_bound = minerals[3] + time * robots[3];
		best = best.max(lower_bound);

		for robot in 0..4 {
			if robot < 3 && max_costs[robot] <= robots[robot] {
				continue;
			}

			let time_to_build = costs[robot]
				.iter()
				.copied()
				.zip(minerals.iter().copied())
				.enumerate()
				.map(|(i, (cost, mineral))| {
					let demand = cost.saturating_sub(mineral);
					let supply = robots[i];
					match (demand, supply) {
						(0, _) => 0,
						(_, 0) => u8::MAX - 1,
						_ => demand / supply + (demand % supply != 0) as u8,
					}
				})
				.max()
				.unwrap_or(0) + 1;
			if time_to_build >= time {
				continue;
			}

			let mut minerals = array_init::array_init(|i| minerals[i] + time_to_build * robots[i]);
			minerals
				.iter_mut()
				.zip(costs[robot].iter().copied())
				.for_each(|(mineral, cost)| *mineral -= cost);
			let robots = array_init::array_init(|i| robots[i] + (i == robot) as u8);
			queue.push(State {
				time: time - time_to_build,
				robots,
				minerals,
			});
		}
	}
	best
}

fn solution() -> Result<()> {
	let blueprints = read_blueprints()?;

	let part_1 = (&blueprints)
		.into_par_iter()
		.enumerate()
		.map(|(index, blueprint)| (index + 1) * maximum_geodes(blueprint, 24) as usize)
		.sum::<usize>();
	println!("{part_1}");

	let part_2 = (&blueprints)
		.into_par_iter()
		.take(3)
		.map(|blueprint| maximum_geodes(blueprint, 32) as usize)
		.product::<usize>();
	println!("{part_2}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_19: (usize, fn() -> Result<()>) = (19, solution);
