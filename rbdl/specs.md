# RBDL (Rair Binary Descriptor Language) Specs Draft

## Summary

[summary]: #summary

RBDL is Declarative Domain Specific Language to formally describe binary file formats. Its objective is to
completely decouple the parsing executable files (as well as any
file-based binary structures) from the remaining process of
reverse engineering. RBDL should be able to handle binary file
formats with variable header size, optional components, as well
as partially corrupted data. RBDL should provide means to
specify how to gracefully recover from partially corrupted
binary file when graceful recovery is possible, all without the need of
incorporating real code.

RBDL should come with RBDL transpiler that generates pure rust
code or nom based rust code. RDBL transpiler must be easily
integrated with cargo building ecosystem, ideally via procedural
macro interface.

RBDL itself should not depend on any language but it is mainly
targeting rust, thus the syntax would be inspired by rust code.
This way rust implementation can make use of the readily
available rust crates for constructing AST whenever possible.

## Motivation

[motivation]: #motivation

Parsing binary file formats is a very complex process, let alone
handling cases where binary files may be tampered with to make
the parsing process itself harder. RBDL aims to simplify the
process of parsing complex binary file formats by delegating the
process of writing parsers to transpiler that accepts
declarative syntax, and generating an imperative parser for the given
format.

## Prior art and Literature review.

[prior-art]: #prior-art

[DFDL](https://www.ogf.org/documents/GFD.207.pdf) and [DADL](http://ops4j.github.io/dadl/0.1.0/) are both W3C XML schema-based languages that serve similar purpose to RBDL. However, They have non-intuitive syntax with lots of syntactic overhead. Moreover, they do not support either algebraic enums or loosely defined structures (structures with `unreliable` members)

[Kaitai](http://ops4j.github.io/dadl/0.1.0/) are YAML based DSL that acts similarly to DFDL and DADL, and it suffers from the same problems.

[pest](https://pest.rs/) is PEG parser but it is only meant for text-based data.


[nom](https://docs.rs/nom/5.0.1/nom/) is an imperative parser combinator library with a focus on safe parsing. Nom is perfectly suitable for writing binary file parsers. RBDL acts as only a declarative layer laid on top of nom.


## General syntax

``` rust
#[attr1, attr2=val]
 name: type {
 // type content here
 }

```

## Comments

RBDL supports both single-line Rust style comments and multiline
rust style comments

``` rust
// This is comment

/*
 * This is a comment
 * as well.*/
```

## Types Reference

[reference-level-explanation]: #reference-level-explanation

### Atomic types with fixed size

* `u8` : Unsigned 8 bits integer.
* `i8` : Signed 8 bits integer.
* `u16` : Unsigned 16 bit integers.
* `i16` : Signed 16 bit integers.
* `u32` : Unsigned 32 bit integers.
* `i32` : Signed 32 bit integers.
* `u64` : Unsigned 64 bit integers.
* `i64` : Signed 64 bit integers.
* `u128` Unsigned 128 bit integers.
* `i128` Signed 128 bit integers.
* `f32` : Single-precision float.
* `f64` : Double-precision float.
* `oct` : Octal digit(s).
* `hex` : hexadecimal digit(s).
* `dec` : decimal digits(s).
* `bin` : binary digit(s).

### Types with variable size

All types with variable size must either have a `size` property or `delimiter` Attribute.

 - `vec<T>` : Zero or more occurences of `T`
 - `String` : Stream or characters.

### Combinatoral types

*Combinatorial* types are used to combine other types.

* `struct`: Allows combining types via concatenating them.

Example:

``` rust
Mystruct: struct {
 #[count=100, encoding="ascii"]
 name: String,
 type: u64,
 #[delimiter=b"\x00", encoding="ascii"]
 owner: String
}
```

* `enum`: enums allow combining types via selecting one or
the other. All members of a given enum must be separable from
each other. Enums are only valid when all members pairs inside the enum are separable. Two members of the same enum are said to be separable
IFF there exists `n` where the data stored at given nth offset
from the start of both members are both static and different.

Example:

``` rust
X: Enum {
 #[static=5]
 V: u8,
 #[static=7)]
 V2: U8,
}
```

Example 2:

``` rust
ExoticFileV1 :struct {
 #[static="EXOFILE"]
 magic: String,
 data1: u64,
 data2: u64,
 #[static="1"]
 Version: char,
 //Rest of file definition goes here
}
ExoticFileV2 :struct {
 #[static="EXOFILE"]
 magic: String,
 data1: u32,
 data2: u64,
 data3: u32,
 #[static="2"]
 Version: char,
 //Rest of file definition goes here
}

Enum {
 v1: ExoticFileV1
 v2: ExoticFileV2
}
```

* `Option\<T\>`: Wrapper around types that may be or may be not
part of the file format. `Option<T>` is

### Attributes

#### `endianness`

Used to set endianness for numeric values that acquire more than one byte.
`endianness` attribute is compatible with `u16`, `s16`, `u32`, `u32`, `u64`, `s64`, `u128`, `s128`.

Possible values:
- `le`: little endian (Default value).
- `be`: big endian.

Example:
```rust
MyStruct: struct {
 #[endianness=be]
 a: u64
}
```

#### `alignment`

Align the start address.

Example:
```rust
MyStruct: struct {
 a: u8
 #[alignment=16]
 b: u8
}
```

#### `padding`

Align the end address.

Example:

```rust
#[padding=512]
TarHeader: struct {
 //stuff here
}
```

#### `discard`
Do not include getter function for members marked with this attribute.

Example:

```rust
Mystruct : struct{
 #[discard]
 my_size: u64,
 #[count=my_size]
 my_vec: Vec<u8>,
}
```

#### `unreliable`

Used on members that may fail to parse correctly. In which case these members will be ignored and parsing will continue past them. Any type referencing an unreliable member must be unreliable as well.

Example
```rust
Mystruct: struct {
 #[count=5, unreliable]
 x: oct
}
```

#### `static`

Used on members with already known values.

Example:

```rust
MyStruct : struct {
 #[static=0b101]
 x: u8,
 #[static=-5, endianness=be]
 y: i64,
 #[static = "Hello", delimiter=0, discard]
 signature: String,
 #[static=[0x00, 0x80, 0xff]]
 data: Vec<u8>,
 #[static='x']
 my_char: char,
 #[count=2, static=0o17]
 my_oct: oct
}
```
#### `encoding`
Allows setting encoding for `String` type. Available Encodings:
- ascii
- utf8
- utf16

#### `delimiter`

Sets the ending delimiter for both vectors and strings.
A delimiter can be a `u8` or `Vec<u8>`.
Note: In case of existence of both `delimiter` and `count`, both are valid whichever comes first.

#### `count`

Sets the number of elements in either `String` or vector.

### Example

#### Tar file parser

``` rust
FileType: enum {
 #[static='0']
 Regular: char,
 #[static='1']
 HardLink: char,
 #[static='2']
 SymbolicLink: char,
 #[static='3']
 CharacterDevice: char,
 #[static='4']
 BlockDevice: char,
 #[static='5']
 Directory: char,
 #[static='6']
 FIFONode: char,
}
#[padding=512]
TarHeader: struct {
 #[size=100, encoding="ascii"]
 file_name: String,
 #[size=8, encoding="ascii"]
 mode: oct,
 #[size=8, encoding="ascii"]
 uid: oct,
 #[size=8, encoding="ascii"]
 gid: oct,
 #[size=12, encoding="ascii"]
 size: oct,
 #[size=12, encoding="ascii"]
 mtime: oct,
 #[size=6, encoding="ascii"]
 checksum: oct,
 #[static=[0, 32], hidden]
 checksum_deimiter: Vec<u8>,
 typeflag: FileType,
 #[size=100, encoding="ascii", delimiter=0]
 linkname: String,
 #[static="ustar", delimiter=0, encoding="ascii", hidden]
 magic: String,
 #[size=2]
 version: String,
 #[size=32, encoding="ascii", delimiter=0]
 user: String,
 #[size=32, encoding="ascii", delimiter=0]
 group: String,
 #[size=8, encoding="ascii"]
 devmajor: oct,
 #[size=8, encoding="ascii"]
 devminor: oct,
 #[size=155, encoding="ascii", delimiter=0]
 prefix: String,
}
```



## Unresolved questions

[unresolved-questions]: #unresolved-questions

* Are these primitives enough to parse all binary file formats?

As pointed to me by @thestr4ng3r, No there is at least a case where using RBDL at its current state will not practical. Consider the following scenario: We have two independent version numbers followed by two pieces of data each depends on its respective version number as shown below:

```rust
FileA0B0: struct {
 #[static=0]
 version_a: u8,
 #[static=0]
 version_b: u8,
 data1: u8,
 data2: u8
}

FileA0B1: struct {
 #[static=0]
 version_a: u8,
 #[static=1]
 version_b: u8,
 data1: u8,
 data2: u16
}

FileA1B0: struct {
 #[static=1]
 version_a: u8,
 #[static=0]
 version_b: u8,
 data1: u16,
 data2: u8
}

FileA1B0: struct {
 #[static=1]
 version_a: u8,
 #[static=0]
 version_b: u8,
 data1: u16,
 data2: u8
}

FullFile: Enum {
 f00: FileA0B0,
 f01: FileA0B1,
 f10: FileA1B0,
 f11: FileA1B1,
}
```

We would need a way to convert `data1` and `data2` into independent enums each with a
separate selector. Ideally, that selector should be flexible enough to describe arbitrary
logic.

Kaitai approach is providing a set of predefined logic selectors such as `switch-on` and `FourCC`. While
these logic selectors cover mosts of the use cases found in the real world. They are not flexible, and they
can't model arbitrary logic without facing the same exponential explosion.

A suggested Approach is using the target language as expression language to provide arbitrary logic support.
Revisiting our original `enum` definition:

    Enums are only valid when all members pairs inside the enum are separable.

Since enums will be able to use that would decide which variant of the enum will be selected. It may not be necessary in that case for every 2 member pairs of the enum to be separable.

Example:

grammar.rbdl

```rust
Version: enum {
 #[static=0]
 V0: u8
 #[static=1]
 V1: u8
}
#[shadow=DataShadow]
Data : enum {
 D8: u8,
 D16: u16,
}

FullFile: struct {
 version_a: Version,
 version_b: Version,
 #[selector=version_sel, selector_args=[version_a]]
 data_a: Data,
 #[selector=version_sel, selector_args=[version_b]]
 data_b: Data,
}
```

grammar.rs
```rust
rbdl_include!("grammar.rdbl")

fn version_sel(version: &Version) -> DataShadow {
 match version {
 Version::V0 => DataShadow::D8,
 Version::V1 => DataShadow::D16
 }
}
```
The code snippet above introduces 3 attributes, the first of which is `shadow`. `shadow` attribute will only be available for rbdl enums and it will translate in the target language to `enum` or its nearest equivalent with the same fields as the attribute users, but no data. It will be used with selector attribute to signal which variation of the rbdl enum to be used. The other 2 attributes are `selector` and `selector_args`. These are used to identify the name of the selector function and which members of the parent struct to send to them.
