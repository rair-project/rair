# RAIR
RAIR is a work in progress rewrite of [radare2](github.com/radare/radare) in rust with these goals:
- native speed.
- simpler building system.
- zero cost memory safety.
- smooth translation to multithreading.

It was hard to decide how to start this project and there was mainly two approaches
A) start rair completely from scratch.
B) replacing parts from radare till it all becomes written in rust.

I prefered the later approach because it will make rair usable right from the begining and it will be testable with [radare2-regressions](https://github.com/radare/radare2-regressions).


## Dependencies
Currently rair is incomplete project and under heavy development it depends on libr completely plus those rust libraraies:
- libc
- getopts
- rust-serialize

## Current goal

Get rid of all the `TODO`s and `unsafe`
If you want to give a hand try
```
git grep unsafe
```
choose 1 and eliminate it!
