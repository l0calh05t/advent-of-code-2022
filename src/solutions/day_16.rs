use std::collections::HashMap;

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use linkme::distributed_slice;
use pathfinding::directed::dijkstra::dijkstra;
use regex::Regex;

type GraphEntry = (u64, Vec<(usize, u64)>);

fn read_graph() -> Result<Vec<GraphEntry>> {
	let mut nodes = HashMap::<_, _>::from_iter([("AA".to_string(), 0)]);
	let mut graph = Vec::new();
	let pattern = Regex::new(
		r"Valve ([A-Z]{2}) has flow rate=(\d+); tunnel(?:s?) lead(?:s?) to valve(?:s?) ([A-Z, ]+)",
	)?;
	try_for_each_line_in_file("inputs/day-16", |line| {
		let line = line.trim();
		let captures = pattern
			.captures(line)
			.ok_or_else(|| eyre!("unexpected input '{line}'"))?;
		let node = captures.get(1).unwrap().as_str();
		let num_nodes = nodes.len();
		let node = *nodes.entry(node.to_string()).or_insert(num_nodes);
		let flow = captures.get(2).unwrap().as_str().parse::<u64>()?;
		let neighbors: Vec<_> = captures
			.get(3)
			.unwrap()
			.as_str()
			.split(", ")
			.map(|neighbor| {
				let num_nodes = nodes.len();
				let neighbor = *nodes.entry(neighbor.to_string()).or_insert(num_nodes);
				(neighbor, 1u64)
			})
			.collect();
		graph.push((node, flow, neighbors));
		Ok(())
	})?;
	graph.sort_by_key(|(node, _, _)| *node);
	let graph = graph
		.into_iter()
		.map(|(_, flow, mut neighbors)| {
			neighbors.sort();
			(flow, neighbors)
		})
		.collect();
	Ok(graph)
}

fn compact_graph(
	graph: &[GraphEntry],
	mut condition: impl FnMut(usize, u64) -> bool,
) -> Vec<GraphEntry> {
	let nodes_to_fold: Vec<_> = graph
		.iter()
		.enumerate()
		.skip(1)
		.filter_map(|(node, (flow, _))| condition(node, *flow).then_some(node))
		.collect();
	let mut graph =
		HashMap::<_, _>::from_iter(graph.iter().enumerate().map(|(node, (flow, neighbors))| {
			let neighbors = HashMap::<_, _>::from_iter(neighbors.iter().copied());
			(node, (*flow, neighbors))
		}));
	for node in nodes_to_fold {
		let (_, neighbors) = graph.remove(&node).unwrap();
		for (neighbor, distance) in &neighbors {
			let current = &mut graph.get_mut(neighbor).unwrap().1;
			let _ = current.remove(&node);
			for (other_neighbor, other_distance) in &neighbors {
				if neighbor == other_neighbor {
					continue;
				}
				let total_distance = distance + other_distance;
				let distance = current.entry(*other_neighbor).or_insert(total_distance);
				*distance = (*distance).min(total_distance);
			}
		}
	}

	let mut nodes = HashMap::<_, _>::with_capacity(graph.len());
	nodes.insert(0usize, 0usize);
	let mut graph: Vec<_> = graph
		.into_iter()
		.map(|(node, (flow, neighbors))| {
			let num_nodes = nodes.len();
			let node = *nodes.entry(node).or_insert(num_nodes);
			let mut neighbors: Vec<_> = neighbors
				.into_iter()
				.map(|(neighbor, distance)| {
					let num_nodes = nodes.len();
					let neighbor = *nodes.entry(neighbor).or_insert(num_nodes);
					(neighbor, distance)
				})
				.collect();
			neighbors.sort_by_key(|(neighbor, _)| *neighbor);
			(node, flow, neighbors)
		})
		.collect();
	graph.sort_by_key(|(node, _, _)| *node);
	graph
		.into_iter()
		.map(|(_, flow, neighbors)| (flow, neighbors))
		.collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
struct State {
	node: usize,
	flow: u64,
	accumulated_flow: u64,
	open: u64,
	time: u64,
}

fn solve(graph: &[GraphEntry], t_max: u64) -> Result<u64> {
	let num_nodes = graph.len();
	let total_max_flow = graph.iter().map(|(flow, _)| *flow).sum::<u64>();

	if num_nodes == 0 {
		return Err(eyre!("graph unexpectedly empty"));
	}
	if num_nodes > 64 {
		return Err(eyre!("graph unexpectedly large"));
	}

	let state = State::default();
	let path = dijkstra(
		&state,
		|state| {
			let State {
				node,
				flow,
				accumulated_flow,
				open,
				time,
			} = *state;
			let node_mask = 1 << node;
			let open_if_closed = (node != 0 && (open & node_mask) == 0).then(|| {
				(
					State {
						node,
						flow: flow + graph[node].0,
						accumulated_flow: accumulated_flow + flow,
						open: open | node_mask,
						time: time + 1,
					},
					total_max_flow - flow,
				)
			});
			let do_nothing = (open == (1 << num_nodes) - 2
				|| graph[node]
					.1
					.iter()
					.all(|(_, distance)| time + distance > t_max))
			.then(|| {
				let dt = t_max - time;
				(
					State {
						node,
						flow,
						accumulated_flow: accumulated_flow + dt * flow,
						open,
						time: t_max,
					},
					dt * (total_max_flow - flow),
				)
			});
			graph[node]
				.1
				.iter()
				.copied()
				.filter_map(move |(neighbor, distance)| {
					(open != (1 << num_nodes) - 2 && time + distance <= t_max).then_some((
						State {
							node: neighbor,
							flow,
							accumulated_flow: accumulated_flow + distance * flow,
							open,
							time: time + distance,
						},
						distance * (total_max_flow - flow),
					))
				})
				.chain(open_if_closed)
				.chain(do_nothing)
		},
		|state| state.time == t_max,
	)
	.ok_or_else(|| eyre!("no solution found!"))?;
	Ok(path.0.last().unwrap().accumulated_flow)
}

fn solution() -> Result<()> {
	let graph = read_graph()?;
	let graph = compact_graph(&graph, |_, flow| flow == 0);

	let flow = solve(&graph, 30)?;
	println!("{flow}");

	let n = graph.len();
	let flow = (1..n)
		.combinations(n / 2)
		.map(|me| {
			let my_graph = compact_graph(&graph, |node, _| me.contains(&node));
			let elephants_graph = compact_graph(&graph, |node, _| !me.contains(&node));
			solve(&my_graph, 26).unwrap() + solve(&elephants_graph, 26).unwrap()
		})
		.max()
		.unwrap();
	println!("{flow}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_16: (usize, fn() -> Result<()>) = (16, solution);
