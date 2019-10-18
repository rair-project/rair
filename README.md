# RAIR  




|      |    Badge     |
|------|--------------|
|**License**|[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)|
|**Travis CI**|  [![Build Status](https://travis-ci.org/Rair-Project/rair-core.svg?branch=master)](https://travis-ci.org/Rair-Project/rair-core)|
|**Code Coverage**|[![codecov](https://codecov.io/gh/Rair-Project/rair-core/branch/master/graph/badge.svg)](https://codecov.io/gh/Rair-Project/rair-core)|
|**API**| [![API](https://img.shields.io/badge/api-master-purple.svg)](https://rair-project.github.io/rair-core/)|
|**Chat**| ![Discord](https://img.shields.io/discord/634133396439695382?style=plastic)|


RAIR is a work in progress rewrite of [radare2](https://github.com/radare/radare2) in rust with these goals:
- Native speed.
- Extremely flexible and modern analysis.
- Simpler building system.
- Worry less about undefined behavior.
- Smooth translation to multithreading.
- Extremely stable and well documented API.
- Focusing on correctiness more than focusing on features.
- Avoid legacy systems compatiability.

It was hard to decide how to start this project and there was mainly two approaches:

A) start rair completely from scratch.

B) replacing parts from radare till it all becomes written in rust.

Ufortunately that didn't work out as expected for many reasons. Idiomatic rust is slightly different from Idiomatic C, where in rust you need to respect a very strict ownership/lifetime model that is almost non existent in C. Also radare2 is moving target and keeping compatiability with such huge system is very hard task on its own.
## Get started

``` bash
$ git clone https://github.com/oddcoder/rair.git
$ cd rair
$ cargo install
```
## Current status
|   Module  	|   Rust Porting   |                                            Documentation                                           |
|:-------------:|:----------------:|:--------------------------------------------------------------------------------------------------:|
|   rtrees      |:heavy_check_mark:|[![API](https://img.shields.io/badge/api-master-purple.svg)](https://rair-project.github.io/rair-core/rtrees)|
|   rio       	|:heavy_check_mark:|[![API](https://img.shields.io/badge/api-master-purple.svg)](https://rair-project.github.io/rair-core/rio/index.html)|
|    Rair       |        WIP       |NA|
|  R2agent  	|        NA        |NA|
|    R2pm   	|        NA        |NA|
|   Rabin   	|        NA        |NA|
|    Ragg   	|        NA        |NA|
|   Rahash  	|        NA        |NA|
|   Rarun   	|        NA        |NA|
|  Ragg-cc  	|        NA        |NA|
|    Rasm   	|        NA        |NA|
|    Rax    	|        NA        |NA|
|   Radiff  	|        NA        |NA|
|   Rafind  	|        NA        |NA|
|   r_anal  	|        NA        |NA|
|   r_asm   	|        NA        |NA|
|   r_bin   	|        NA        |NA|
|    r_bp   	|        NA        |NA|
|  r_config 	|        NA        |NA|
|   r_cons  	|        NA        |NA|
|   r_core  	|        NA        |NA|
|  r_crypto 	|        NA        |NA|
|  r_debug  	|        NA        |NA|
|   r_egg   	|        NA        |NA|
|   r_flag  	|        NA        |NA|
|    r_fs   	|        NA        |NA|
|   r_hash  	|        NA        |NA|
|   r_lang  	|        NA        |NA|
|  r_magic  	|        NA        |NA|
|  r_parse  	|        NA        |NA|
|   r_reg   	|        NA        |NA|
|  r_search 	|        NA        |NA|
|  r_socket 	|        NA        |NA|
| r_syscall 	|        NA        |NA|
|   r_util  	|        NA        |NA|

## Get involved

Regardless to how skilled/unskilled you are, there will always be something for you to do! I always try to keep the code base clean.
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
