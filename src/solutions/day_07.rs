use std::collections::hash_map::Entry::*;
use std::collections::HashMap;

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

#[derive(Debug, Clone)]
enum Node {
	File(usize),
	Directory(HashMap<String, Node>),
}

use Node::*;

impl Node {
	fn size(&self) -> usize {
		match self {
			File(s) => *s,
			Directory(ns) => ns.values().map(Node::size).sum(),
		}
	}
}

fn part_1(node: &Node) -> usize {
	if let Directory(entries) = node {
		let size = node.size();
		(if size <= 100_000 { size } else { 0 })
			+ entries.iter().map(|(_, node)| part_1(node)).sum::<usize>()
	} else {
		0
	}
}

fn part_2(node: &Node) -> usize {
	let total_size = node.size();
	let total_available = 70_000_000;
	let required = 30_000_000;
	let to_free = total_size + required - total_available;
	let mut smallest_sufficient = total_size;

	fn recursion(to_free: usize, smallest_sufficient: &mut usize, node: &Node) {
		if let Directory(entries) = node {
			let size = node.size();
			if size >= to_free && size < *smallest_sufficient {
				*smallest_sufficient = size;
			}
			entries
				.values()
				.for_each(|e| recursion(to_free, smallest_sufficient, e));
		}
	}
	recursion(to_free, &mut smallest_sufficient, node);

	smallest_sufficient
}

fn solution() -> Result<()> {
	let mut cwd: Option<Vec<String>> = None;
	let mut in_ls = false;
	let mut root = Directory(HashMap::new());
	try_for_each_line_in_file("inputs/day-07", |line| {
		let line = line.trim();
		if let Some(parameters) = line.strip_prefix("$ cd ") {
			in_ls = false;
			match parameters {
				"/" => {
					cwd = Some(Vec::new());
				}
				".." => {
					cwd.as_mut()
						.ok_or_else(|| eyre!("can't leave an unknown directory"))?
						.pop()
						.ok_or_else(|| eyre!("can't leave the root directory"))?;
				}
				path => {
					cwd.as_mut()
						.ok_or_else(|| eyre!("can't access the children of an unknown directory"))?
						.push(path.to_string());
				}
			}
		} else if line == "$ ls" {
			in_ls = true;
			cwd.as_ref()
				.ok_or_else(|| eyre!("can't list an unknown directory"))?;
		} else {
			if !in_ls {
				return Err(eyre!("unexpected output in log file"));
			}

			let mut cur = match &mut root {
				Directory(root) => root,
				_ => unreachable!(),
			};
			for dir in cwd.as_ref().expect("unreachable") {
				cur = match cur
					.get_mut(dir)
					.ok_or_else(|| eyre!("listing a non-existent directory"))?
				{
					Directory(target) => target,
					_ => return Err(eyre!("listing a file instead of a directory")),
				};
			}

			let (front, back) = line
				.split_once(' ')
				.ok_or_else(|| eyre!("missing separator in ls output"))?;
			if front == "dir" {
				cur.entry(back.to_string())
					.or_insert_with(|| Directory(HashMap::new()));
			} else {
				let size = front.parse::<usize>()?;
				match cur.entry(back.to_string()) {
					Occupied(e) => {
						if let File(size_existing) = e.get() {
							if *size_existing != size {
								return Err(eyre!("file size changed!"));
							}
						} else {
							return Err(eyre!("directory is now a file!"));
						}
					}
					Vacant(e) => {
						e.insert(File(size));
					}
				}
			}
		}
		Ok(())
	})?;

	println!("{}", part_1(&root));
	println!("{}", part_2(&root));

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_07: (usize, fn() -> Result<()>) = (7, solution);
