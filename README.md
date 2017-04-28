# RAIR [![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

**Travis CI:**  [![Build Status](https://travis-ci.org/oddcoder/rair.svg?branch=master)](https://travis-ci.org/oddcoder/rair)

**AppVeyor:** [![Build status](https://ci.appveyor.com/api/projects/status/sn9d2w6vcctn7mvt/branch/master?svg=true)](https://ci.appveyor.com/project/oddcoder/rair/branch/master)

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

Currently rair depends almost completely on libr so the simplest way to have libr installed is to install radare2 fully first then installing rair.

``` bash
$ git clone https://github.com/radare/radare2.git
$ git clone https://github.com/oddcoder/rair.git
$ cd radare2
$ sys /install.sh
$ cd ../rair
$ cargo install
```
## Current status
|   Module  	| Rust Porting 	| Documentation 	| Testing 	|
|:---------:	|:------------:	|:-------------:	|:-------:	|
|  R2agent  	|      NA      	|       NA      	|    NA   	|
|    R2pm   	|      NA      	|       NA      	|    NA   	|
|   Rabin   	|      NA      	|       NA      	|    NA   	|
|    Ragg   	|      NA      	|       NA      	|    NA   	|
|   Rahash  	|     Done     	|       -h      	|   Done     	|
|   Rarun   	|      NA      	|       NA      	|    NA   	|
|    Rair  	|      NA      	|       NA      	|    NA   	|
|  Ragg-cc  	|      NA      	|       NA      	|    NA   	|
|    Rasm   	|      NA      	|       NA      	|    NA   	|
|    Rax    	|      NA      	|       NA      	|    NA   	|
|   Radiff  	|      NA      	|       NA      	|    NA   	|
|   Rafind  	|   Partial    	|       -h      	|   Done   	|
|   r_anal  	|      NA      	|       NA      	|    NA   	|
|   r_asm   	|      NA      	|       NA      	|    NA   	|
|   r_bin   	|      NA      	|       NA      	|    NA   	|
|    r_bp   	|      NA      	|       NA      	|    NA   	|
|  r_config 	|      NA      	|       NA      	|    NA   	|
|   r_cons  	|  Partial FFI 	|       NA      	|    NA   	|
|   r_core  	|      NA      	|       NA      	|    NA   	|
|  r_crypto 	|  Partial FFI 	|       NA      	|    NA   	|
|  r_debug  	|      NA      	|       NA      	|    NA   	|
|   r_egg   	|      NA      	|       NA      	|    NA   	|
|   r_flag  	|      NA      	|       NA      	|    NA   	|
|    r_fs   	|      NA      	|       NA      	|    NA   	|
|   r_hash  	|  Partial FFI 	|       NA      	|    NA   	|
|    r_io   	|  Partial FFI 	|       NA      	|    NA   	|
|   r_lang  	|      NA      	|       NA      	|    NA   	|
|  r_magic  	|      NA      	|       NA      	|    NA   	|
|  r_parse  	|      NA      	|       NA      	|    NA   	|
|   r_reg   	|      NA      	|       NA      	|    NA   	|
|  r_search 	|   Partial    	|       NA      	|    NA   	|
|  r_socket 	|      NA      	|       NA      	|    NA   	|
| r_syscall 	|      NA      	|       NA      	|    NA   	|
|   r_util  	|  Partial FFI 	|       NA      	|    NA   	|

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
	* ragg2-cc
	* rarun2
	* rax2
