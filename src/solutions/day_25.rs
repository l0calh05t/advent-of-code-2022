use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;

trait Snafu: Copy {
	fn from_snafu(snafu: &str) -> Result<Self>;
	fn to_snafu(self) -> String;
}

impl Snafu for u64 {
	fn from_snafu(snafu: &str) -> Result<Self> {
		let mut result: u64 = 0;
		for &byte in snafu.as_bytes() {
			result = result
				.checked_mul(5)
				.ok_or_else(|| eyre!("overflow while parsing snafu"))?;
			result = result
				.checked_add_signed(match byte {
					b'2' => 2,
					b'1' => 1,
					b'0' => 0,
					b'-' => -1,
					b'=' => -2,
					_ => return Err(eyre!("invalid snafu number")),
				})
				.ok_or_else(|| eyre!("overflow while parsing snafu"))?;
		}
		Ok(result)
	}

	fn to_snafu(mut self) -> String {
		let mut result = vec![];
		while self != 0 {
			let remainder = self % 5;
			self /= 5;
			let digit = match remainder {
				0 => b'0',
				1 => b'1',
				2 => b'2',
				3 => {
					self += 1;
					b'='
				}
				4 => {
					self += 1;
					b'-'
				}
				_ => unreachable!(),
			};
			result.push(digit);
		}
		result.reverse();
		String::from_utf8(result).unwrap()
	}
}

fn solution() -> Result<()> {
	let mut sum = 0;
	try_for_each_line_in_file("inputs/day-25", |line| {
		let line = line.trim();
		let converted: u64 = Snafu::from_snafu(line)?;
		sum += converted;
		Ok(())
	})?;
	let snafu_sum = sum.to_snafu();
	println!("{snafu_sum}");
	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_25: (usize, fn() -> Result<()>) = (25, solution);
