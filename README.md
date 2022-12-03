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

## Day 2

Nothing to see here.
Only took the time to use integer-`repr` enums and compute the outcomes instead of using large, multi-case matches.

## Day 3

Pretty basic stuff, especially if you are using sets and working with normal `for`-loops (the combination of `Result` and iterators tends to get ugly real quick).
In principle, the allocations from the sets could be removed by sorting the byte arrays in place and deduplicating/intersecting in place (`line` and `items` are already re-used between iterations).
However, [`partition_dedup`](https://doc.rust-lang.org/std/primitive.slice.html#method.partition_dedup) isn't in stable Rust yet.
Alternatively, [`itertools::Itertools::dedup`](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.dedup) could be used.
While sorted intersection is fairly easy to implement (and available [as a crate](https://docs.rs/sorted_intersection/latest/sorted_intersection/)), I decided to stick with the conceptually straightforward `HashSet`-approach since there is really no need for speed.
(I may change my mind in the future, and do it just because why not.)

### Update

While taking a walk I came to the conclusion that none of that is necessary.
Since there are only 52 possible items, the `HashSet` can just be replaced with and `u64` and intersections with `&` etc.
To fold the results I went with [`itertools::Itertools::fold_ok`](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.fold_ok).
One of my favorite crates around.
