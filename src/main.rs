use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use color_eyre::eyre::Result;
use linkme::distributed_slice;

mod solutions {
	automod::dir!("src/solutions");
}

#[distributed_slice]
static SOLUTIONS: [(usize, fn() -> Result<()>)] = [..];

fn read_segmented<T: FromStr, P: AsRef<Path>>(file_name: P) -> Result<Vec<Vec<T>>>
where
	<T as FromStr>::Err: 'static + Error + Send + Sync,
{
	fn read_segmented_impl<T: FromStr>(file_name: &Path) -> Result<Vec<Vec<T>>>
	where
		<T as FromStr>::Err: 'static + Error + Send + Sync,
	{
		let file = File::open(file_name)?;
		let mut file = BufReader::new(file);
		let mut result = Vec::new();
		let mut line = String::new();
		let mut new_segment = true;
		while file.read_line(&mut line)? != 0 {
			let trimmed = line.trim();
			if trimmed.is_empty() {
				new_segment = true;
				continue;
			}
			if new_segment {
				new_segment = false;
				result.push(Vec::new());
			}
			let segment = result.last_mut().expect("unreachable");
			segment.push(trimmed.parse()?);
			line.clear();
		}
		Ok(result)
	}

	read_segmented_impl(file_name.as_ref())
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
