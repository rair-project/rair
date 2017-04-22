# RAIR
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

RAIR is a work in progress rewrite of [radare2](http://github.com/radare/radare2) in rust with these goals:
- native speed.
- simpler building system.
- zero cost memory safety.
- smooth translation to multithreading.
- extremely stable and well documented API.

It was hard to decide how to start this project and there was mainly two approaches

A) start rair completely from scratch.

B) replacing parts from radare till it all becomes written in rust.

I prefered the later approach because it will make rair usable right from the begining and it will be testable with [radare2-regressions](https://github.com/radare/radare2-regressions).

## Get started

`TODO`

## Dependencies
Currently rair is incomplete project and under heavy development it depends on libr completely plus those rust libraraies:
- libc
- getopts
- rust-serialize

## Get involved

Regardless to how skilled you are, there will always be something for you to do! I always try to keep the code base clean.
This is list of what you can do from easy to the more challenging:

- Improve this readme.
- Add support for code coverage / travis / and other CI systems
- Document already exisiting functions.
- fix one of `cargo clippy` warnings.
- Refactor the current codebase, there are many long functions and breaking them down is really usefull.
- Write unit tests/ fuzz rair
- imlement one or more of these:
	* rabin2
	* radiff2
	* ragg2
	* rasm2
	* radare2
	* rafind2
	* ragg2-cc
	* rarun2
	* rax2
