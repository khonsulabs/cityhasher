# cityhasher

[![crate version](https://img.shields.io/crates/v/cityhasher.svg)](https://crates.io/crates/cityhasher)
[![Live Build Status](https://img.shields.io/github/actions/workflow/status/khonsulabs/cityhasher/rust.yml?branch=main)](https://github.com/khonsulabs/cityhasher/actions?query=workflow:Tests)
[![Documentation for `main`](https://img.shields.io/badge/docs-main-informational)](https://khonsulabs.github.io/cityhasher/main/cityhasher/)

A pure Rust implementation of [CityHash](https://github.com/google/cityhash), a
hashing algorithm developed at Google by Geoff Pike and Jyrki Alakuijala that
has impressive performance and hashing quality.

This crate implements the current version of CityHash (v1.1) for 32- and 64-bit
outputs. If any of the remaining functionality is desired, please [open an
issue](https://github.com/khonsulabs/cityhasher).

This crate was implemented by directly porting the C++ reference implementation.
The test suite from the C++ codebase was also ported, and this implementation
passes the test suite. The goal of this crate is to provide a pure Rust
implementation that produces idential output as the C++ reference
implementation.

## Why use CityHash?

Before considering why to use CityHash, the primary reason to avoid CityHash is
if you need a hashing algorithm that is suitable for cryptography. Otherwise,
from [Google's announcement][ann]:

> We were greatly inspired by previous work on hashing, especially Austin
> Appleby’s MurmurHash. The key advantage of our approach is that most steps
> contain at least two independent mathematical operations. Modern CPUs tend to
> perform best with this type of code.
>
> The disadvantage of our approach is that the code is more complicated than
> most popular alternatives. We decided to optimize for speed rather than
> simplicity and even included special cases for short inputs.

[ann]: https://opensource.googleblog.com/2011/04/introducing-cityhash.html

## no_std support

This crate supports `no_std` environments when the `std` feature is not enabled.
Because this feature is enabled by default, add the crate without default
features to ensure no_std compatibility:

```toml
cityhasher = { version = "*", default-features = false }
```

## Feature Flags

- `std`: Enables type aliases for `HashMap` and `HashSet`. Enabled by default.
- `disable-bounds-checking`: When this flag is enabled, the crate utilizes
  unsafe code to access the data being hashed without bounds checking. This
  crate utilizes no other unsafe code. Enabled by default.

## Using HashMap/HashSet with this crate

This crate exports type aliases making it easy to use the standard library
collection types:

```rust
let mut map = cityhasher::HashMap::new();
map.insert(1, "hello");
map.insert(2, "world");
assert_eq!(map.get(&1), Some(&"hello"));

let mut set = cityhasher::HashSet::new();
set.insert(1);
set.insert(2);
assert!(set.contains(&1));
```

These type aliases are included if the `std` feature is enabled.

## Hashing bytes with this crate

This crate provides two functions to hash data: [`hash`] and [`hash_with_seed`].
Both functions use a generic parameter to control the hashing algorithm. If you
need to be compatible with other implementations of CityHash, ensure that you
are using the same size unsigned type in Rust as the desired output size.

The original CityHash library does not provide a [`hash_with_seed`] compatible
implementation for 32-bit hashes.

```rust
let hash32: u32 = cityhasher::hash("hello");
let hash64: u64 = cityhasher::hash("hello");let bytes =
assert_ne!(hash32 as u64, hash64);

let hash64_seeded: u64 = cityhasher::hash_with_seed("hello", 1);
assert_ne!(hash64_seeded, hash64);
```

## Benchmarks

This crate performs nearly identically as the original C++ implementation when
benchmarked on the developers machine using:

```sh
RUSTFLAGS="-C target-cpu=native" cargo +nightly bench -p benchmarks --features unsafe,nightly
```

This crate does not require nightly, but `cityhash-sys` currently does (v1.0.5).

### 32-bit hashing

This benchmark measures how long it takes to produce 32-bit hash values of
inputs of varying byte lengths. The other crates compared against are
[cityhash-sys](https://github.com/HUD-Software/cityhash-sys) and
[crc32fast](https://github.com/srijs/rust-crc32fast):

```text
32bit/cityhasher/2      time:   [4.1483 ns 4.1721 ns 4.1993 ns]
32bit/cityhash-sys/2    time:   [3.6513 ns 3.6723 ns 3.6947 ns]
32bit/crc32fast/2       time:   [2.4503 ns 2.4559 ns 2.4617 ns]

32bit/cityhasher/4      time:   [5.3657 ns 5.4042 ns 5.4434 ns]
32bit/cityhash-sys/4    time:   [4.2997 ns 4.3064 ns 4.3149 ns]
32bit/crc32fast/4       time:   [3.8654 ns 3.9192 ns 3.9758 ns]

32bit/cityhasher/8      time:   [4.2414 ns 4.2749 ns 4.3351 ns]
32bit/cityhash-sys/8    time:   [3.5043 ns 3.5184 ns 3.5350 ns]
32bit/crc32fast/8       time:   [8.0307 ns 8.0413 ns 8.0532 ns]

32bit/cityhasher/16     time:   [5.5017 ns 5.5311 ns 5.5709 ns]
32bit/cityhash-sys/16   time:   [5.6581 ns 5.6704 ns 5.6841 ns]
32bit/crc32fast/16      time:   [18.174 ns 18.221 ns 18.274 ns]

32bit/cityhasher/32     time:   [9.0291 ns 9.0965 ns 9.1629 ns]
32bit/cityhash-sys/32   time:   [8.9324 ns 8.9682 ns 9.0069 ns]
32bit/crc32fast/32      time:   [39.367 ns 39.514 ns 39.721 ns]

32bit/cityhasher/64     time:   [15.288 ns 15.325 ns 15.362 ns]
32bit/cityhash-sys/64   time:   [14.604 ns 14.618 ns 14.634 ns]
32bit/crc32fast/64      time:   [20.384 ns 20.460 ns 20.559 ns]

32bit/cityhasher/96     time:   [17.496 ns 17.617 ns 17.744 ns]
32bit/cityhash-sys/96   time:   [17.076 ns 17.195 ns 17.354 ns]
32bit/crc32fast/96      time:   [76.879 ns 76.966 ns 77.074 ns]

32bit/cityhasher/128    time:   [22.088 ns 22.120 ns 22.163 ns]
32bit/cityhash-sys/128  time:   [22.174 ns 22.343 ns 22.518 ns]
32bit/crc32fast/128     time:   [9.0384 ns 9.0418 ns 9.0454 ns]

32bit/cityhasher/256    time:   [38.263 ns 38.442 ns 38.665 ns]
32bit/cityhash-sys/256  time:   [38.025 ns 38.295 ns 38.714 ns]
32bit/crc32fast/256     time:   [16.311 ns 16.370 ns 16.472 ns]

32bit/cityhasher/1024   time:   [139.20 ns 139.45 ns 139.75 ns]
32bit/cityhash-sys/1024 time:   [137.39 ns 137.58 ns 137.77 ns]
32bit/crc32fast/1024    time:   [62.221 ns 62.513 ns 63.034 ns]
```

### 64-bit hashing

This benchmark measures how long it takes to produce 64-bit hash values of
inputs of varying byte lengths. The other crates compared against are
[cityhash-sys](https://github.com/HUD-Software/cityhash-sys) and
[fnv](https://github.com/servo/rust-fnv):

```text
64bit/cityhasher/2      time:   [1.9630 ns 1.9703 ns 1.9782 ns]
64bit/cityhash-sys/2    time:   [2.6516 ns 2.6580 ns 2.6664 ns]
64bit/fnv/2             time:   [1.2426 ns 1.2501 ns 1.2592 ns]

64bit/cityhasher/4      time:   [1.6810 ns 1.6823 ns 1.6841 ns]
64bit/cityhash-sys/4    time:   [2.4811 ns 2.4863 ns 2.4932 ns]
64bit/fnv/4             time:   [1.8129 ns 1.8194 ns 1.8279 ns]

64bit/cityhasher/8      time:   [2.0259 ns 2.0272 ns 2.0294 ns]
64bit/cityhash-sys/8    time:   [2.7576 ns 2.7611 ns 2.7652 ns]
64bit/fnv/8             time:   [3.4148 ns 3.4985 ns 3.5753 ns]

64bit/cityhasher/16     time:   [2.0576 ns 2.0651 ns 2.0725 ns]
64bit/cityhash-sys/16   time:   [2.9018 ns 2.9384 ns 2.9808 ns]
64bit/fnv/16            time:   [7.8188 ns 7.8976 ns 7.9746 ns]

64bit/cityhasher/32     time:   [2.3330 ns 2.3428 ns 2.3557 ns]
64bit/cityhash-sys/32   time:   [2.9538 ns 2.9643 ns 2.9756 ns]
64bit/fnv/32            time:   [19.301 ns 19.387 ns 19.513 ns]

64bit/cityhasher/64     time:   [4.4110 ns 4.4556 ns 4.4919 ns]
64bit/cityhash-sys/64   time:   [4.5723 ns 4.5807 ns 4.5907 ns]
64bit/fnv/64            time:   [48.732 ns 48.889 ns 49.102 ns]

64bit/cityhasher/96     time:   [12.336 ns 12.410 ns 12.504 ns]
64bit/cityhash-sys/96   time:   [11.522 ns 11.548 ns 11.578 ns]
64bit/fnv/96            time:   [84.950 ns 85.080 ns 85.232 ns]

64bit/cityhasher/128    time:   [12.197 ns 12.224 ns 12.259 ns]
64bit/cityhash-sys/128  time:   [11.168 ns 11.205 ns 11.252 ns]
64bit/fnv/128           time:   [113.30 ns 113.33 ns 113.35 ns]

64bit/cityhasher/256    time:   [19.206 ns 19.284 ns 19.372 ns]
64bit/cityhash-sys/256  time:   [17.944 ns 17.997 ns 18.064 ns]
64bit/fnv/256           time:   [235.58 ns 236.00 ns 236.53 ns]

64bit/cityhasher/1024   time:   [58.193 ns 58.214 ns 58.236 ns]
64bit/cityhash-sys/1024 time:   [57.869 ns 57.896 ns 57.928 ns]
64bit/fnv/1024          time:   [962.78 ns 963.07 ns 963.42 ns]
```
