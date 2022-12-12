use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str::FromStr;

use color_eyre::eyre::Result;
use itertools::Itertools;
use linkme::distributed_slice;
use ndarray::{prelude::*, ErrorKind::IncompatibleShape, ShapeError};

mod solutions {
	automod::dir!("src/solutions");
}

#[distributed_slice]
static SOLUTIONS: [(usize, fn() -> Result<()>)] = [..];

fn try_for_each_line_in_file<P, F>(file_name: P, function: F) -> Result<()>
where
	P: AsRef<Path>,
	F: FnMut(&str) -> Result<()>,
{
	fn try_for_each_line_in_file_impl<F>(file_name: &Path, mut function: F) -> Result<()>
	where
		F: FnMut(&str) -> Result<()>,
	{
		let file = File::open(file_name)?;
		let mut file = BufReader::new(file);
		let mut line = String::new();

		while file.read_line(&mut line)? != 0 {
			function(&line)?;
			line.clear();
		}

		Ok(())
	}

	try_for_each_line_in_file_impl(file_name.as_ref(), function)
}

fn read_segmented<T: FromStr, P: AsRef<Path>>(file_name: P) -> Result<Vec<Vec<T>>>
where
	<T as FromStr>::Err: 'static + Error + Send + Sync,
{
	let mut result = Vec::new();
	let mut new_segment = true;
	try_for_each_line_in_file(file_name, |line| {
		let line = line.trim();
		if line.is_empty() {
			new_segment = true;
			return Ok(());
		}
		if new_segment {
			new_segment = false;
			result.push(Vec::new());
		}
		let segment = result.last_mut().expect("unreachable");
		segment.push(line.parse()?);
		Ok(())
	})?;
	Ok(result)
}

fn read_digit_field(file_name: &str) -> Result<Array2<u8>> {
	let mut bytes = Vec::new();
	File::open(file_name)?.read_to_end(&mut bytes)?;
	let mut lines = 0usize;
	let mut columns = None;
	let values = bytes
		.split(|&b| b == b'\n')
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
			Ok(line)
		})
		.flatten_ok()
		.map_ok(|&b| b.checked_sub(b'0').unwrap())
		.collect::<Result<Vec<_>>>()?;
	Ok(Array2::from_shape_vec((lines, columns.unwrap()), values)?)
}

fn main() -> Result<()> {
	color_eyre::install()?;

	let day = match std::env::args().nth(1) {
		Some(day) => day.parse::<usize>()?,
		None => {
			eprintln!("please pass the day");
			std::process::exit(1);
		}
	};

	SOLUTIONS
		.iter()
		.find(|(i, _)| *i == day)
		.unwrap_or_else(|| todo!("day {day} not implemented!"))
		.1()
}
