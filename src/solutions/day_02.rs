use std::{
	fs::File,
	io::{BufRead, BufReader},
};

use crate::SOLUTIONS;

use color_eyre::eyre::Result;
use linkme::distributed_slice;

#[derive(Clone, Copy, Debug)]
#[repr(i8)]
enum RochambeauPlay {
	Rock = 0,
	Paper,
	Scissors,
}

use RochambeauPlay::*;

impl RochambeauPlay {
	fn score(self) -> u64 {
		self as u64 + 1
	}

	fn outcome(self, response: Self) -> RochambeauOutcome {
		match (response as i8 - self as i8 + 4) % 3 - 1 {
			-1 => Loss,
			0 => Draw,
			1 => Win,
			_ => unreachable!(),
		}
	}
}

#[derive(Clone, Copy, Debug)]
#[repr(i8)]
enum RochambeauOutcome {
	Loss = -1,
	Draw,
	Win,
}

use RochambeauOutcome::*;

impl RochambeauOutcome {
	fn score(self) -> u64 {
		(self as i8 + 1) as u64 * 3
	}

	fn response(self, call: RochambeauPlay) -> RochambeauPlay {
		match (self as i8 + call as i8 + 3) % 3 {
			0 => Rock,
			1 => Paper,
			2 => Scissors,
			_ => unreachable!(),
		}
	}
}

fn read_games() -> Result<Vec<(RochambeauPlay, RochambeauPlay)>> {
	let file = File::open("inputs/day-02")?;
	let mut file = BufReader::new(file);
	let mut result = Vec::new();
	let mut line = String::new();
	while file.read_line(&mut line)? != 0 {
		if let Some((call, response)) = line.trim().split_once(' ') {
			let call = match call {
				"A" => Rock,
				"B" => Paper,
				"C" => Scissors,
				_ => panic!("unexpected Rochambeau call {call}"),
			};
			let response = match response {
				"X" => Rock,
				"Y" => Paper,
				"Z" => Scissors,
				_ => panic!("unexpected Rochambeau response {response}"),
			};
			result.push((call, response));
		}
		line.clear();
	}
	Ok(result)
}

fn score(call: RochambeauPlay, response: RochambeauPlay) -> u64 {
	response.score() + call.outcome(response).score()
}

fn score_outcome(call: RochambeauPlay, outcome: RochambeauOutcome) -> u64 {
	outcome.response(call).score() + outcome.score()
}

fn solution() -> Result<()> {
	let games = read_games()?;
	let solution_1 = games
		.iter()
		.copied()
		.map(|(call, response)| score(call, response))
		.sum::<u64>();
	println!("{solution_1}");

	let solution_2 = games
		.into_iter()
		.map(|(call, response)| {
			let outcome = match response {
				Rock => Loss,
				Paper => Draw,
				Scissors => Win,
			};
			score_outcome(call, outcome)
		})
		.sum::<u64>();
	println!("{solution_2}");
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_02: (usize, fn() -> Result<()>) = (2, solution);
