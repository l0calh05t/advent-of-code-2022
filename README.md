# Advent of Code 2022

Much like [last year](https://github.com/l0calh05t/advent-of-code-2021), this repository collects my Rust solutions and thoughts regarding Advent of Code 2022.
I am *not* attempting to compete for any leaderboards, just doing these for fun and to try out crates I haven’t gotten around to using (enough).
So far these include:

- [automod](https://github.com/dtolnay/automod)
- [derive_more](https://github.com/JelteF/derive_more)
- [linkme](https://github.com/dtolnay/linkme)
- [nom](https://github.com/Geal/nom)

## Day 1

As usual, Day 1 is pretty straightforward.
The only (minor) optimization here, is to use `select_nth_unstable_by` instead of sorting the array in its entirety.
Or rather it is an optimization as long as the input isn’t a pathological case, see [rust-lang/rust#102451](https://github.com/rust-lang/rust/issues/102451).

Instead, I focused on using [automod](https://github.com/dtolnay/automod) and [linkme](https://github.com/dtolnay/linkme) to create a setup that should require a little less boilerplate per day than last year’s workspace approach.

## Day 2

Nothing to see here.
Only took the time to use integer-`repr` enums and compute the outcomes instead of using large, multi-case matches.

## Day 3

Pretty basic stuff, especially if you are using sets and working with normal `for`-loops (the combination of `Result` and iterators tends to get ugly real quick).
In principle, the allocations from the sets could be removed by sorting the byte arrays in place and deduplicating/intersecting in place (`line` and `items` are already re-used between iterations).
However, [`partition_dedup`](https://doc.rust-lang.org/std/primitive.slice.html#method.partition_dedup) isn’t in stable Rust yet.
Alternatively, [`itertools::Itertools::dedup`](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.dedup) could be used.
While sorted intersection is fairly easy to implement (and available [as a crate](https://docs.rs/sorted_intersection/latest/sorted_intersection/)), I decided to stick with the conceptually straightforward `HashSet`-approach since there is really no need for speed.
(I may change my mind in the future, and do it just because why not.)

### Update

While taking a walk I came to the conclusion that none of that is necessary.
Since there are only 52 possible items, the `HashSet` can just be replaced with a `u64` and intersections with `&` etc.
To fold the results I went with [`itertools::Itertools::fold_ok`](https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.fold_ok).
One of my favorite crates around.

## Day 4

Yet another easy one, so I refactored my iterative line-by-line processing (everyone else seems to do iterators-only this year and I already did that last year for a majority of tasks) into a common `try_for_each_line_in_file`-function.

## Day 5

Text formats should be illegal (especially fixed-width formats).

That said most of the effort in this task went into the parsing.
The rest is pretty much busywork.
In the second part, you are likely to run into a spot where Rust’s borrow checker may seem to get in the way if you try to mutably access two indices of a `Vec` at the same time (“seem” because it’s doing its job as intended—see below).
You can avoid this either by using a temporary (the result of `split_off` or a reused `mut Vec`) or by applying some `split_at_mut` trickery, which is what I did.

Yes, of course you could also use `unsafe` 😉, but in that case you might accidentally miss the potential `from == to` case and summon [nasal demons 👃🏻👿](http://catb.org/jargon/html/N/nasal-demons.html).

## Day 6

No file reading this time, since existing functions to create sliding windows over a `Result` iterator aren't great, especially if the error type isn't `Clone`.

## Day 7

Again, all the effort lies in the parsing.
Since I used [chumsky](https://github.com/zesterer/chumsky/) last year, I wanted to try [nom](https://github.com/Geal/nom) this year.
However, I was not able to get it to parse the last `ls` output.
In combination with [nom-bufreader](https://github.com/rust-bakery/nom-bufreader), the result was even worse with an infinite loop while parsing the last `ls`.
Since it's getting late, I hacked together an ad-hoc parser instead 🫤

## Day 8

Several days late because I was on brief vacation to London to see The Nightmare Before Christmas in Concert (totally worth it!! 🎃🎅🏿👻❄️🦇🎄).
Finally, some re-use from last year, at least the IO using `read_digit_field`.
Avoiding code duplication between directions using iterators was not trivial, but I think it turned out ok.

## Day 9

I'm currently reading the file twice, so that's not ideal, but the solution is the same for both due to the use of const generics!
I still wish we had full const generics, though.
Non-type template parameters and variadic templates are the two C++ features I regularly wish we had in Rust.

## Day 10

Another fairly easy one, no special crates either.
Nothing like last years 'assembly', but that was Day 24.

## Day 11

Another attempt at using [nom](https://github.com/Geal/nom) with [nom-bufreader](https://github.com/rust-bakery/nom-bufreader), again without success.
Even without num-bufreader, I couldn't get nom's streaming combinators to work as expected.
Reading the entire file up front and using complete combinators worked without other changes though.
Not sure PEG parsers and I will ever be friends.
The task itself easy enough, if you know some math.
I used `lcm` to combine the factors, but later noticed all factors are prime, so I could have just multiplied them.
I reused the Day 1 optimization here.
Like Day 5 with two indices, we ideally want mutable references to items at three indices here.
Doing it without `unsafe` is definitely possible, but extremely unwieldy, so I implemented a generic `pick_disjoint_mut` that performs the necessary checks (index validity and disjointedness), then just uses a line of `unsafe` code.

## Day 12

Pathfinding forward and reverse, easily solve with good old Dijkstra.
No point in re-implementing it, so I used the very nice [pathfinding](https://github.com/samueltardieu/pathfinding) crate again, like last year for Day 23.
I really like not having to create an explicit graph and the simple interface.
Didn't notice too much of a difference between the new 4.*x* version and 3.*x* version I used last year.

## Day 13

All the work is in the parsing again (using nom once more, but with `&str` instead of `&[u8]` this time).
The rest of the task pretty much writes itself.
Quite literally in Rust 🦀 due to `#[derive(...)]`.

## Day 14

Not particularly challenging, but a fun one.
Used [derive_more](https://github.com/JelteF/derive_more) on this one for `IntoIter`.
Also added a custom `impl Display` for my simulation state, but graphical output would be even neater.
I may have to add some form of graphical output at some point.
