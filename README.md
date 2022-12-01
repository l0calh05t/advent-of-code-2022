# Advent of Code 2022

Much like [last year](https://github.com/l0calh05t/advent-of-code-2021), this repository collects my Rust solutions and thoughts regarding Advent of Code 2022.
I am *not* attempting to compete for any leaderboards, just doing these for fun and to try out crates I haven't gotten around to using (enough).
So far these include:

- [automod](https://github.com/dtolnay/automod)
- [linkme](https://github.com/dtolnay/linkme)

## Day 1

As usual, Day 1 is pretty straightforward.
The only (minor) optimization here, is to use `select_nth_unstable_by` instead of sorting the array in its entirety.
Or rather it is an optimization as long as the input isn't a pathological case, see [rust-lang/rust#102451](https://github.com/rust-lang/rust/issues/102451).

Instead, I focused on using [automod](https://github.com/dtolnay/automod) and [linkme](https://github.com/dtolnay/linkme) to create a setup that should require a little less boilerplate per day than last year's workspace approach.
