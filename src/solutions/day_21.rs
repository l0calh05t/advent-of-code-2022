use std::{
	collections::{hash_map::Entry, HashMap},
	fmt::{Debug, Display, Write},
	str::from_utf8,
};

use crate::try_for_each_line_in_file;
use crate::SOLUTIONS;

use color_eyre::eyre::{eyre, Result};
use linkme::distributed_slice;
use regex::{Captures, Regex};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MonkeyName([u8; 4]);

impl Debug for MonkeyName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("MonkeyName({})", from_utf8(&self.0).unwrap()))
	}
}

impl Display for MonkeyName {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(from_utf8(&self.0).unwrap())
	}
}

#[derive(Clone, Copy, Debug)]
enum MonkeyBusiness<T> {
	Yell(T),
	Add(MonkeyName, MonkeyName),
	Subtract(MonkeyName, MonkeyName),
	Multiply(MonkeyName, MonkeyName),
	Divide(MonkeyName, MonkeyName),
}

#[derive(Clone, Copy, Debug)]
struct Monkey<T> {
	name: MonkeyName,
	business: MonkeyBusiness<T>,
}

fn read_input() -> Result<Vec<Monkey<i64>>> {
	let [p0, p1] = [
		r"^([a-z]{4}): (\d+)$",
		r"^([a-z]{4}): ([a-z]{4}) ([+\-*/]) ([a-z]{4})$",
	]
	.map(Regex::new);
	let patterns = [p0?, p1?];

	let name_from_captures = |captures: &Captures, i: usize| {
		MonkeyName(
			array_init::from_iter(captures.get(i).unwrap().as_str().as_bytes().iter().copied())
				.unwrap(),
		)
	};

	let mut monkeys = vec![];
	try_for_each_line_in_file("inputs/day-21", |line| {
		let line = line.trim();
		let monkey;
		if let Some(captures) = patterns[0].captures(line) {
			let name = name_from_captures(&captures, 1);
			let value = captures.get(2).unwrap().as_str().parse()?;
			let business = MonkeyBusiness::Yell(value);
			monkey = Monkey { name, business }
		} else if let Some(captures) = patterns[1].captures(line) {
			let name = name_from_captures(&captures, 1);
			let left = name_from_captures(&captures, 2);
			let right = name_from_captures(&captures, 4);
			let business_type = match captures.get(3).unwrap().as_str() {
				"+" => MonkeyBusiness::Add,
				"-" => MonkeyBusiness::Subtract,
				"*" => MonkeyBusiness::Multiply,
				"/" => MonkeyBusiness::Divide,
				_ => unreachable!(),
			};
			let business = business_type(left, right);
			monkey = Monkey { name, business };
		} else {
			return Err(eyre!("invalid input {line}"));
		}
		monkeys.push(monkey);
		Ok(())
	})?;

	Ok(monkeys)
}

fn perform_monkey_business<T>(
	name: MonkeyName,
	monkeys: &mut HashMap<MonkeyName, MonkeyBusiness<T>>,
) -> Result<T>
where
	T: Clone
		+ std::ops::Add<T, Output = T>
		+ std::ops::Sub<T, Output = T>
		+ std::ops::Mul<T, Output = T>
		+ std::ops::Div<T, Output = T>,
{
	let business_entry = match monkeys.entry(name) {
		Entry::Occupied(business) => business,
		_ => return Err(eyre!("invalid monkey {name} or cycle!")),
	};

	let (left, right) = match business_entry.get() {
		MonkeyBusiness::Yell(value) => return Ok(value.clone()),
		MonkeyBusiness::Add(left, right)
		| MonkeyBusiness::Subtract(left, right)
		| MonkeyBusiness::Multiply(left, right)
		| MonkeyBusiness::Divide(left, right) => (*left, *right),
	};

	let business = business_entry.remove();
	let left = perform_monkey_business(left, monkeys)?;
	let right = perform_monkey_business(right, monkeys)?;

	let result = match business {
		MonkeyBusiness::Add(_, _) => left + right,
		MonkeyBusiness::Subtract(_, _) => left - right,
		MonkeyBusiness::Multiply(_, _) => left * right,
		MonkeyBusiness::Divide(_, _) => left / right,
		_ => unreachable!(),
	};

	monkeys.insert(name, MonkeyBusiness::Yell(result.clone()));

	Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Symbolic {
	Unknown,
	Constant(i64),
	Add(Box<Symbolic>, Box<Symbolic>),
	Subtract(Box<Symbolic>, Box<Symbolic>),
	Multiply(Box<Symbolic>, Box<Symbolic>),
	Divide(Box<Symbolic>, Box<Symbolic>),
}

impl Symbolic {
	fn is_zero(&self) -> bool {
		matches!(self, Symbolic::Constant(0))
	}

	fn is_one(&self) -> bool {
		matches!(self, Symbolic::Constant(1))
	}

	fn constant(&self) -> Option<i64> {
		match self {
			Symbolic::Constant(constant) => Some(*constant),
			_ => None,
		}
	}

	fn simplify(self) -> Symbolic {
		use Symbolic::*;
		match self {
			Add(left, right) => {
				let left = left.simplify();
				let right = right.simplify();
				if left.is_zero() {
					return right;
				} else if right.is_zero() {
					return left;
				}
				match (left, right) {
					(Unknown, Unknown) => unreachable!(),
					(Unknown, other) | (other, Unknown) => Add(Box::new(other), Box::new(Unknown)),
					(Constant(left), Constant(right)) => Constant(left + right),
					(Constant(constant), Add(left, right))
					| (Add(left, right), Constant(constant)) => {
						if let Some(other) = left.constant() {
							let constant = constant + other;
							if constant == 0 {
								*right
							} else {
								Add(Box::new(Constant(constant)), right)
							}
						} else {
							Add(Box::new(Constant(constant)), Box::new(Add(left, right)))
						}
					}
					(Constant(constant), other) | (other, Constant(constant)) => {
						Add(Box::new(Constant(constant)), Box::new(other))
					}
					(Subtract(a, b), Subtract(c, d)) => {
						Subtract(Box::new(Add(a, c)), Box::new(Add(b, d)))
					}
					(left, right) => Add(Box::new(left), Box::new(right)),
				}
			}
			Subtract(left, right) => {
				let left = left.simplify();
				let right = right.simplify();
				if right.is_zero() {
					return left;
				}
				match (left, right) {
					(Unknown, Unknown) => unreachable!(),
					(Constant(left), Constant(right)) => Constant(left - right),
					(left, right) => Subtract(Box::new(left), Box::new(right)),
				}
			}
			Multiply(left, right) => {
				let left = left.simplify();
				let right = right.simplify();
				if left.is_one() {
					return right;
				} else if right.is_one() {
					return left;
				}
				match (left, right) {
					(Unknown, Unknown) => unreachable!(),
					(Unknown, other) | (other, Unknown) => {
						Multiply(Box::new(other), Box::new(Unknown))
					}
					(Constant(left), Constant(right)) => Constant(left * right),
					(Constant(constant), Multiply(left, right))
					| (Multiply(left, right), Constant(constant)) => {
						if let Some(other) = left.constant() {
							let constant = constant * other;
							if constant == 1 {
								*right
							} else {
								Multiply(Box::new(Constant(constant)), right)
							}
						} else {
							Multiply(
								Box::new(Constant(constant)),
								Box::new(Multiply(left, right)),
							)
						}
					}
					(Constant(constant), other) | (other, Constant(constant)) => {
						Multiply(Box::new(Constant(constant)), Box::new(other))
					}
					(Divide(a, b), Divide(c, d)) => {
						Divide(Box::new(Multiply(a, c)), Box::new(Multiply(b, d)))
					}
					(left, right) => Multiply(Box::new(left), Box::new(right)),
				}
			}
			Divide(left, right) => {
				let left = left.simplify();
				let right = right.simplify();
				if right.is_one() {
					return left;
				}
				match (left, right) {
					(Unknown, Unknown) => unreachable!(),
					(Constant(left), Constant(right)) => {
						assert!(left.rem_euclid(right) == 0);
						Constant(left / right)
					}
					(left, right) => Divide(Box::new(left), Box::new(right)),
				}
			}
			simple => simple,
		}
	}

	fn simplify_full(self) -> Symbolic {
		let mut current = self;
		let mut prev;
		while {
			prev = current.clone();
			current = current.simplify();
			prev != current
		} {}
		current
	}
}

impl Display for Symbolic {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use Symbolic::*;
		match self {
			Unknown => f.write_char('x'),
			Constant(constant) => f.write_fmt(format_args!("{constant}")),
			Add(left, right) => f.write_fmt(format_args!("({left} + {right})")),
			Subtract(left, right) => f.write_fmt(format_args!("({left} - {right})")),
			Multiply(left, right) => f.write_fmt(format_args!("({left} * {right})")),
			Divide(left, right) => f.write_fmt(format_args!("({left} / {right})")),
		}
	}
}

impl std::ops::Add<Symbolic> for Symbolic {
	type Output = Self;

	fn add(self, rhs: Symbolic) -> Self::Output {
		Symbolic::Add(Box::new(self), Box::new(rhs))
	}
}

impl std::ops::Sub<Symbolic> for Symbolic {
	type Output = Self;

	fn sub(self, rhs: Symbolic) -> Self::Output {
		Symbolic::Subtract(Box::new(self), Box::new(rhs))
	}
}

impl std::ops::Mul<Symbolic> for Symbolic {
	type Output = Self;

	fn mul(self, rhs: Symbolic) -> Self::Output {
		Symbolic::Multiply(Box::new(self), Box::new(rhs))
	}
}

impl std::ops::Div<Symbolic> for Symbolic {
	type Output = Self;

	fn div(self, rhs: Symbolic) -> Self::Output {
		Symbolic::Divide(Box::new(self), Box::new(rhs))
	}
}

fn solution() -> Result<()> {
	let monkeys = read_input()?;

	let mut memoized_monkeys =
		HashMap::<_, _>::from_iter(monkeys.iter().copied().map(|m| (m.name, m.business)));

	let mut symbolic_monkeys = HashMap::<_, _>::from_iter(memoized_monkeys.iter().map(|(k, v)| {
		let k = *k;
		let v = if k == MonkeyName(*b"humn") {
			MonkeyBusiness::Yell(Symbolic::Unknown)
		} else {
			match *v {
				MonkeyBusiness::Yell(v) => MonkeyBusiness::Yell(Symbolic::Constant(v)),
				MonkeyBusiness::Add(left, right) => MonkeyBusiness::Add(left, right),
				MonkeyBusiness::Subtract(left, right) => MonkeyBusiness::Subtract(left, right),
				MonkeyBusiness::Multiply(left, right) => MonkeyBusiness::Multiply(left, right),
				MonkeyBusiness::Divide(left, right) => MonkeyBusiness::Divide(left, right),
			}
		};
		(k, v)
	}));

	let result = perform_monkey_business(MonkeyName(*b"root"), &mut memoized_monkeys)?;
	println!("{result}");

	let root = symbolic_monkeys.remove(&MonkeyName(*b"root")).unwrap();
	let (left, right) = match root {
		MonkeyBusiness::Yell(_) => return Err(eyre!("root doesn't have dependents!")),
		MonkeyBusiness::Add(left, right)
		| MonkeyBusiness::Subtract(left, right)
		| MonkeyBusiness::Multiply(left, right)
		| MonkeyBusiness::Divide(left, right) => (left, right),
	};

	let left = perform_monkey_business(left, &mut symbolic_monkeys)?.simplify_full();
	let right = perform_monkey_business(right, &mut symbolic_monkeys)?.simplify_full();

	let (target, mut expression) = match (left, right) {
		(Symbolic::Constant(constant), right) => (constant, right),
		(left, Symbolic::Constant(constant)) => (constant, left),
		_ => return Err(eyre!("expected one side to be a constant")),
	};

	let mut target = num::Rational64::from_integer(target);
	while !matches!(expression, Symbolic::Unknown) {
		match expression {
			Symbolic::Unknown | Symbolic::Constant(_) => unreachable!(),
			Symbolic::Add(constant, other) => {
				let constant = constant.constant().expect("not simplified properly?");
				target -= constant;
				expression = *other;
			}
			Symbolic::Subtract(left, right) => match (left.as_ref(), right.as_ref()) {
				(Symbolic::Constant(constant), _) => {
					target = num::Rational64::from_integer(*constant) - target;
					expression = *right;
				}
				(_, Symbolic::Constant(constant)) => {
					target += *constant;
					expression = *left;
				}
				_ => unreachable!("not simplified properly?"),
			},
			Symbolic::Multiply(constant, other) => {
				let constant = constant.constant().expect("not simplified properly?");
				target /= constant;
				expression = *other;
			}
			Symbolic::Divide(left, right) => match (left.as_ref(), right.as_ref()) {
				(Symbolic::Constant(constant), _) => {
					target = num::Rational64::from_integer(*constant) / target;
					expression = *right;
				}
				(_, Symbolic::Constant(constant)) => {
					target *= *constant;
					expression = *left;
				}
				_ => unreachable!("not simplified properly?"),
			},
		}
	}

	println!("{target}");

	Ok(())
}

#[distributed_slice(SOLUTIONS)]
static SOLUTION_DAY_21: (usize, fn() -> Result<()>) = (21, solution);
